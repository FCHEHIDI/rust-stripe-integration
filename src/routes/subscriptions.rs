// EXERCICE 2: Routes pour les abonnements récurrents

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::models::*;
use crate::services::stripe_service;
use crate::state::AppState;

/// Créer un nouvel abonnement
pub async fn create_subscription(
    State(state): State<AppState>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<Json<SubscriptionResponse>, (StatusCode, Json<ApiError>)> {
    // Vérifier que le plan existe
    let plan = state.subscription_plans.get(&req.plan_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Plan d'abonnement non trouvé".to_string() })
        ))?;
    
    tracing::info!("Création abonnement {} pour user {}", plan.name, req.user_id);
    
    // Créer un client Stripe
    let customer = stripe_service::create_customer(
        &state.stripe_client,
        &req.email,
        &req.user_id,
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: format!("Erreur création client: {}", e) })
    ))?;
    
    // Créer un produit Stripe pour l'abonnement
    let price_id = stripe_service::create_subscription_price(
        &state.stripe_client,
        &plan.name,
        plan.price,
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: format!("Erreur création prix: {}", e) })
    ))?;
    
    // Créer l'abonnement Stripe
    let subscription = stripe_service::create_subscription(
        &state.stripe_client,
        &customer.id.to_string(),
        &price_id,
        Some(&req.payment_method),
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: format!("Erreur création abonnement: {}", e) })
    ))?;
    
    // Enregistrer l'abonnement
    let sub_id = Uuid::new_v4().to_string();
    let user_sub = UserSubscription {
        id: sub_id.clone(),
        user_id: req.user_id.clone(),
        plan_id: req.plan_id.clone(),
        stripe_subscription_id: subscription.id.to_string(),
        status: SubscriptionStatus::Active,
        current_period_end: Utc::now() + chrono::Duration::days(30),
        cancel_at_period_end: false,
        created_at: Utc::now(),
        payment_method_id: None,
    };
    
    state.subscriptions.insert(sub_id.clone(), user_sub);
    
    tracing::info!("Abonnement créé: {} - Plan: {} ({}€/mois)", 
                  sub_id, plan.name, plan.price as f64 / 100.0);
    
    // Récupérer le client secret pour confirmer le paiement
    let client_secret = subscription.latest_invoice
        .and_then(|invoice| {
            match invoice {
                stripe::Expandable::Id(_) => None,
                stripe::Expandable::Object(inv) => {
                    inv.payment_intent.and_then(|pi| {
                        match pi {
                            stripe::Expandable::Object(intent) => intent.client_secret,
                            _ => None,
                        }
                    })
                }
            }
        })
        .unwrap_or_default();
    
    Ok(Json(SubscriptionResponse {
        subscription_id: sub_id,
        client_secret,
        status: "active".to_string(),
    }))
}

/// Récupérer un abonnement
pub async fn get_subscription(
    State(state): State<AppState>,
    Path(sub_id): Path<String>,
) -> Result<Json<UserSubscription>, (StatusCode, Json<ApiError>)> {
    let subscription = state.subscriptions.get(&sub_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Abonnement non trouvé".to_string() })
        ))?;
    
    Ok(Json(subscription.clone()))
}

/// Annuler un abonnement
pub async fn cancel_subscription(
    State(state): State<AppState>,
    Path(sub_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let mut subscription = state.subscriptions.get_mut(&sub_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Abonnement non trouvé".to_string() })
        ))?;
    
    // Annuler sur Stripe
    stripe_service::cancel_subscription(
        &state.stripe_client,
        &subscription.stripe_subscription_id,
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: format!("Erreur annulation: {}", e) })
    ))?;
    
    subscription.status = SubscriptionStatus::Cancelled;
    subscription.cancel_at_period_end = true;
    
    tracing::info!("Abonnement annulé: {}", sub_id);
    
    Ok(Json(serde_json::json!({
        "message": "Abonnement annulé avec succès",
        "subscription_id": sub_id,
    })))
}
