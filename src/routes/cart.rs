// EXERCICE 1: Routes pour la gestion du panier et paiement

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::models::*;
use crate::services::stripe_service;
use crate::state::AppState;

// Policy d'annulation: 24 heures max après création de commande
const CANCELLATION_WINDOW_HOURS: i64 = 24;

/// Ajouter un article au panier
pub async fn add_to_cart(
    State(state): State<AppState>,
    Json(req): Json<AddToCartRequest>,
) -> Result<Json<Cart>, (StatusCode, Json<ApiError>)> {
    // Vérifier que le produit existe et a du stock
    let product = state.products.get(&req.product_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Produit non trouvé".to_string() })
        ))?;
    
    if product.stock < req.quantity {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError { 
                error: format!("Stock insuffisant.")
            })
        ));
    }
    
    // Récupérer ou créer le panier et le modifier directement
    let mut cart_ref = state.carts.entry(req.user_id.clone())
        .or_insert_with(|| Cart {
            user_id: req.user_id.clone(),
            items: vec![],
            created_at: Utc::now(),
        });
    
    // Ajouter ou mettre à jour l'article
    if let Some(item) = cart_ref.items.iter_mut().find(|i| i.product_id == req.product_id) {
        item.quantity += req.quantity;
    } else {
        cart_ref.items.push(CartItem {
            product_id: req.product_id.clone(),
            quantity: req.quantity,
        });
    }
    
    tracing::info!("✅ Article ajouté au panier pour user {}", req.user_id);
    // Clone uniquement pour la réponse JSON
    Ok(Json(cart_ref.clone()))
}

// Fonction helper pour calculer le panier avec validation
struct CartCalculation {
    items: Vec<OrderItem>,
    total: i64,
    stock_warnings: Vec<String>,
}

fn calculate_cart(
    cart: &Cart,
    products: &dashmap::DashMap<String, Product>,
    validate_stock: bool,
) -> Result<CartCalculation, (StatusCode, Json<ApiError>)> {
    let mut items = vec![];
    let mut total = 0i64;
    let mut stock_warnings = vec![];
    
    for item in &cart.items {
        let product = products.get(&item.product_id)
            .ok_or_else(|| (
                StatusCode::NOT_FOUND,
                Json(ApiError { error: format!("Produit {} introuvable", item.product_id) })
            ))?;
        
        // Vérifier le stock
        if product.stock < item.quantity {
            let warning = format!("{}: demandé {}, disponible {}", 
                                product.name, item.quantity, product.stock);
            
            if validate_stock {
                // Mode strict (checkout): bloquer
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError { 
                        error: format!("Stock insuffisant pour {}. Disponible: {}", 
                                     product.name, product.stock)
                    })
                ));
            } else {
                // Mode lecture (view_cart): warning seulement
                stock_warnings.push(warning);
            }
        }
        
        let subtotal = product.price * item.quantity as i64;
        total += subtotal;
        
        items.push(OrderItem {
            product_id: product.id.clone(),
            product_name: product.name.clone(),
            quantity: item.quantity,
            price: product.price,
        });
    }
    
    Ok(CartCalculation { items, total, stock_warnings })
}

#[derive(Deserialize)]
pub struct ViewCartQuery {
    user_id: String,
}

/// Voir le contenu du panier
pub async fn view_cart(
    State(state): State<AppState>,
    Query(query): Query<ViewCartQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let cart = state.carts.get(&query.user_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Panier vide".to_string() })
        ))?;
    
    // Utiliser la fonction helper (sans validation stock pour le view)
    let calc = calculate_cart(&cart, &state.products, false)?;
    
    // Formater pour la réponse JSON
    let items_detail: Vec<_> = calc.items.iter().map(|item| {
        serde_json::json!({
            "product_id": item.product_id,
            "name": item.product_name,
            "price": item.price,
            "quantity": item.quantity,
            "subtotal": item.price * item.quantity as i64,
        })
    }).collect();
    
    Ok(Json(serde_json::json!({
        "user_id": cart.user_id,
        "items": items_detail,
        "total": calc.total,
        "created_at": cart.created_at,
        "stock_warnings": calc.stock_warnings,  // Alertes visibles!
    })))
}

/// Passer à la caisse (créer un PaymentIntent Stripe)
pub async fn checkout(
    State(state): State<AppState>,
    Json(req): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, (StatusCode, Json<ApiError>)> {
    // Récupérer le panier
    let cart = state.carts.get(&req.user_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Panier vide".to_string() })
        ))?;
    
    if cart.items.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError { error: "Le panier est vide".to_string() })
        ));
    }
    
    // Utiliser la fonction helper (avec validation stock pour le checkout)
    let calc = calculate_cart(&cart, &state.products, true)?;
    
    // Créer la commande
    let order_id = Uuid::new_v4().to_string();
    let order = Order {
        id: order_id.clone(),
        user_id: req.user_id.clone(),
        items: calc.items,
        total: calc.total,
        status: OrderStatus::Pending,
        payment_intent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Créer le PaymentIntent Stripe
    let payment_intent = stripe_service::create_payment_intent(
        &state.stripe_client,
        calc.total,
        &order_id,
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: format!("Erreur Stripe: {}", e) })
    ))?;
    
    // Mettre à jour la commande avec le PaymentIntent ID
    let mut order = order;
    order.payment_intent_id = Some(payment_intent.id.to_string());
    order.status = OrderStatus::Processing;
    
    state.orders.insert(order_id.clone(), order);
    
    tracing::info!("💳 Checkout créé pour user {} - Montant: {}€", 
                  req.user_id, calc.total as f64 / 100.0);
    
    Ok(Json(CheckoutResponse {
        order_id: order_id.clone(),
        checkout_url: format!("{}/checkout/{}", state.config.base_url, order_id),
        client_secret: payment_intent.client_secret.unwrap_or_default(),
    }))
}

/// Récupérer une commande
pub async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<Order>, (StatusCode, Json<ApiError>)> {
    let order = state.orders.get(&order_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Commande non trouvée".to_string() })
        ))?;
    
    Ok(Json(order.clone()))
}

#[derive(Deserialize)]
pub struct ListOrdersQuery {
    user_id: String,
}

/// Lister toutes les commandes d'un utilisateur
pub async fn list_orders(
    State(state): State<AppState>,
    Query(query): Query<ListOrdersQuery>,
) -> Result<Json<Vec<Order>>, (StatusCode, Json<ApiError>)> {
    let orders: Vec<Order> = state.orders
        .iter()
        .filter(|entry| entry.value().user_id == query.user_id)
        .map(|entry| entry.value().clone())
        .collect();
    
    Ok(Json(orders))
}

/// Annuler une commande (seulement si Pending ou Failed + dans les 24h)
pub async fn cancel_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let mut order = state.orders.get_mut(&order_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Commande non trouvée".to_string() })
        ))?;
    
    // Vérifier le délai d'annulation (24h max)
    let now = Utc::now();
    let elapsed_hours = (now - order.created_at).num_hours();
    
    if elapsed_hours > CANCELLATION_WINDOW_HOURS {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError { 
                error: format!(
                    "Délai d'annulation dépassé. Vous pouviez annuler dans les {} heures suivant la commande. Commande créée il y a {} heures.",
                    CANCELLATION_WINDOW_HOURS,
                    elapsed_hours
                )
            })
        ));
    }
    
    // On ne peut annuler que les commandes Pending ou Failed
    match order.status {
        OrderStatus::Completed => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError { error: "Commande déjà complétée, annulation impossible".to_string() })
            ));
        }
        OrderStatus::Processing => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError { error: "Paiement en cours, attendez la confirmation".to_string() })
            ));
        }
        OrderStatus::Cancelled => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError { error: "Commande déjà annulée".to_string() })
            ));
        }
        _ => {}
    }
    
    order.status = OrderStatus::Cancelled;
    order.updated_at = Utc::now();
    
    tracing::info!("Commande {} annulée par l'utilisateur ({}h après création)", 
                  order_id, elapsed_hours);
    
    Ok(Json(serde_json::json!({
        "message": "Commande annulée avec succès",
        "order_id": order_id,
        "cancelled_at": now,
        "hours_since_order": elapsed_hours,
    })))
}

/// Modifier une commande (jusqu'au dernier moment si stock dispo)
/// ⚠️ L'équipe logistique va vous détester pour cette feature
pub async fn update_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
    Json(req): Json<UpdateOrderRequest>,
) -> Result<Json<Order>, (StatusCode, Json<ApiError>)> {
    let mut order = state.orders.get_mut(&order_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Commande non trouvée".to_string() })
        ))?;
    
    // Vérifier le délai de modification (même fenêtre que l'annulation)
    let now = Utc::now();
    let elapsed_hours = (now - order.created_at).num_hours();
    
    if elapsed_hours > CANCELLATION_WINDOW_HOURS {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError { 
                error: format!(
                    "Délai de modification dépassé. Vous pouviez modifier dans les {} heures suivant la commande.",
                    CANCELLATION_WINDOW_HOURS
                )
            })
        ));
    }
    
    // On ne peut modifier que les commandes Pending ou Failed
    match order.status {
        OrderStatus::Completed => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError { error: "Commande déjà complétée, modification impossible".to_string() })
            ));
        }
        OrderStatus::Processing => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError { error: "Paiement en cours, modification impossible".to_string() })
            ));
        }
        OrderStatus::Cancelled => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError { error: "Commande annulée, modification impossible".to_string() })
            ));
        }
        _ => {}
    }
    
    // Créer un panier temporaire pour valider les nouveaux items
    let temp_cart = Cart {
        user_id: order.user_id.clone(),
        items: req.items.clone(),
        created_at: order.created_at,
    };
    
    // Valider le nouveau panier (stock, etc.) avec le helper
    let calc = calculate_cart(&temp_cart, &state.products, true)?;
    
    // Mettre à jour la commande
    order.items = calc.items;
    order.total = calc.total;
    order.updated_at = now;
    order.status = OrderStatus::Pending; // Réinitialiser si besoin d'un nouveau paiement
    
    tracing::warn!("⚠️ Commande {} modifiée par l'utilisateur ({}h après création) - Nouveau total: {}€", 
                  order_id, elapsed_hours, calc.total as f64 / 100.0);
    tracing::warn!("📢 ALERTE LOGISTIQUE: Commande {} modifiée!", order_id);
    
    Ok(Json(order.clone()))
}
