// Routes pour les webhooks Stripe

use axum::{
    body::Bytes,
    extract::State,
    http::{StatusCode, HeaderMap},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::models::*;
use crate::state::AppState;

/// Handler pour les webhooks Stripe
pub async fn stripe_webhook(
    State(state): State<AppState>,
    // headers: HeaderMap,  // Décommenter pour vérifier la signature
    body: Bytes,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let payload = String::from_utf8(body.to_vec())
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(ApiError { error: "Invalid payload".to_string() })
        ))?;
    
    // ========== VÉRIFICATION DE SIGNATURE (Production) ==========
    // ⚠️ IMPORTANT: En production, TOUJOURS vérifier la signature Stripe
    // pour empêcher des attaquants d'envoyer de faux webhooks
    
    /* IMPLÉMENTATION COMPLÈTE:
    
    // 1. Récupérer le header Stripe-Signature
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(ApiError { error: "Missing Stripe signature".to_string() })
        ))?;
    
    // 2. Vérifier la signature avec le secret webhook
    let webhook_secret = &state.config.stripe_webhook_secret;
    
    use stripe::Webhook;
    let event = Webhook::construct_event(
        &payload,
        signature,
        webhook_secret
    ).map_err(|e| {
        tracing::error!("❌ Signature webhook invalide: {}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiError { error: "Invalid webhook signature".to_string() })
        )
    })?;
    
    // event est maintenant un stripe::Event vérifié et typé
    tracing::info!("✅ Signature webhook vérifiée");
    
    */
    
    // VERSION SIMPLIFIÉE (pour démo uniquement - NON SÉCURISÉ)
    // On parse directement sans vérifier la signature
    let event: serde_json::Value = serde_json::from_str(&payload)
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(ApiError { error: "Invalid JSON".to_string() })
        ))?;
    
    // ============================================================
    
    let event_type = event["type"].as_str().unwrap_or("");
    
    tracing::info!("📨 Webhook reçu: {}", event_type);
    
    match event_type {
        // Paiement réussi
        "payment_intent.succeeded" => {
            handle_payment_success(&state, &event).await?;
        }
        
        // Paiement échoué
        "payment_intent.payment_failed" => {
            handle_payment_failed(&state, &event).await?;
        }
        
        // SetupIntent réussi (carte enregistrée)
        "setup_intent.succeeded" => {
            handle_setup_success(&state, &event).await?;
        }
        
        // Abonnement créé
        "customer.subscription.created" => {
            tracing::info!("✅ Abonnement créé");
        }
        
        // Paiement abonnement réussi
        "invoice.payment_succeeded" => {
            handle_invoice_paid(&state, &event).await?;
        }
        
        // Paiement abonnement échoué
        "invoice.payment_failed" => {
            handle_invoice_failed(&state, &event).await?;
        }
        
        // Carte expirée bientôt
        "customer.source.expiring" => {
            tracing::warn!("⚠️ Carte expire bientôt - notification à envoyer");
        }
        
        _ => {
            tracing::info!("ℹ️ Événement non géré: {}", event_type);
        }
    }
    
    Ok(StatusCode::OK)
}

async fn handle_payment_success(
    state: &AppState,
    event: &serde_json::Value,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let payment_intent_id = event["data"]["object"]["id"].as_str().unwrap_or("");
    let metadata = &event["data"]["object"]["metadata"];
    let order_id = metadata["order_id"].as_str();
    
    if let Some(order_id) = order_id {
        if let Some(mut order) = state.orders.get_mut(order_id) {
            order.status = OrderStatus::Completed;
            order.updated_at = Utc::now();
            
            let user_id = order.user_id.clone();
            
            // Décrémenter les stocks
            for item in &order.items {
                if let Some(mut product) = state.products.get_mut(&item.product_id) {
                    product.stock -= item.quantity;
                    tracing::info!("Stock mis à jour: {} - nouveau stock: {}", 
                                 product.name, product.stock);
                }
            }
            
            // Vider le panier MAINTENANT (paiement confirmé)
            state.carts.remove(&user_id);
            
            tracing::info!("Commande {} payée avec succès - PI: {}", order_id, payment_intent_id);
            println!("\n NOTIFICATION CLIENT: Votre commande {} a été confirmée!", order_id);
        }
    }
    
    Ok(())
}

async fn handle_payment_failed(
    state: &AppState,
    event: &serde_json::Value,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let payment_intent_id = event["data"]["object"]["id"].as_str().unwrap_or("");
    let metadata = &event["data"]["object"]["metadata"];
    let order_id = metadata["order_id"].as_str();
    
    if let Some(order_id) = order_id {
        if let Some(mut order) = state.orders.get_mut(order_id) {
            order.status = OrderStatus::Failed;
            order.updated_at = Utc::now();
            
            tracing::error!("Paiement échoué pour commande {} - PI: {}", order_id, payment_intent_id);
            println!("\n NOTIFICATION CLIENT: Le paiement pour votre commande {} a échoué", order_id);
        }
    }
    
    Ok(())
}

async fn handle_setup_success(
    state: &AppState,
    event: &serde_json::Value,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let payment_method_id = event["data"]["object"]["payment_method"].as_str().unwrap_or("");
    let metadata = &event["data"]["object"]["metadata"];
    let user_id = metadata["user_id"].as_str().unwrap_or("");
    
    // Récupérer les infos de la carte depuis Stripe (simulé)
    let saved_pm = SavedPaymentMethod {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        stripe_payment_method_id: payment_method_id.to_string(),
        card_last4: "4242".to_string(), // À récupérer depuis Stripe en réel
        card_brand: "visa".to_string(),
        exp_month: 12,
        exp_year: 2025,
        is_default: state.payment_methods.iter()
            .filter(|pm| pm.user_id == user_id)
            .count() == 0,
        created_at: Utc::now(),
    };
    
    state.payment_methods.insert(saved_pm.id.clone(), saved_pm.clone());
    
    tracing::info!("Carte enregistrée pour user {} - ****{}", user_id, saved_pm.card_last4);
    println!("\n NOTIFICATION CLIENT: Votre carte a été enregistrée avec succès!");
    
    Ok(())
}

async fn handle_invoice_paid(
    state: &AppState,
    event: &serde_json::Value,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let subscription_id = event["data"]["object"]["subscription"].as_str().unwrap_or("");
    let amount = event["data"]["object"]["amount_paid"].as_i64().unwrap_or(0);
    
    tracing::info!("Facture payée pour abonnement {} - Montant: {}€", 
                 subscription_id, amount as f64 / 100.0);
    println!("\n NOTIFICATION CLIENT: Votre abonnement a été renouvelé - Montant: {}€", 
            amount as f64 / 100.0);
    
    Ok(())
}

async fn handle_invoice_failed(
    state: &AppState,
    event: &serde_json::Value,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let subscription_id = event["data"]["object"]["subscription"].as_str().unwrap_or("");
    let attempt_count = event["data"]["object"]["attempt_count"].as_i64().unwrap_or(0);
    
    tracing::warn!("Échec paiement abonnement {} - Tentative {}/3", 
                  subscription_id, attempt_count);
    
    if attempt_count >= 3 {
        // Suspendre l'abonnement
        for mut sub in state.subscriptions.iter_mut() {
            if sub.stripe_subscription_id == subscription_id {
                sub.status = SubscriptionStatus::PastDue;
                tracing::error!("Abonnement {} suspendu après 3 échecs", subscription_id);
                println!("\n NOTIFICATION CLIENT: Votre abonnement a été suspendu suite à des échecs de paiement");
                break;
            }
        }
    } else {
        println!("\n NOTIFICATION CLIENT: Le paiement de votre abonnement a échoué. Nouvelle tentative dans {} jour(s)", 
                3 - attempt_count);
    }
    
    Ok(())
}
