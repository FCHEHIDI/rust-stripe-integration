// EXERCICE 3: Routes pour la gestion des moyens de paiement

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

/// Configurer un nouveau moyen de paiement
pub async fn setup_payment_method(
    State(state): State<AppState>,
    Json(req): Json<SetupPaymentMethodRequest>,
) -> Result<Json<SetupResponse>, (StatusCode, Json<ApiError>)> {
    tracing::info!("💳 Configuration moyen de paiement pour user {}", req.user_id);
    
    // Créer un SetupIntent Stripe
    let setup_intent = stripe_service::create_setup_intent(
        &state.stripe_client,
        &req.user_id,
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: format!("Erreur création SetupIntent: {}", e) })
    ))?;
    
    Ok(Json(SetupResponse {
        setup_intent_id: setup_intent.id.to_string(),
        client_secret: setup_intent.client_secret.unwrap_or_default(),
    }))
}

#[derive(Deserialize)]
pub struct ListPaymentMethodsQuery {
    user_id: String,
}

/// Lister les moyens de paiement sauvegardés
pub async fn list_payment_methods(
    State(state): State<AppState>,
    Query(query): Query<ListPaymentMethodsQuery>,
) -> Result<Json<Vec<SavedPaymentMethod>>, (StatusCode, Json<ApiError>)> {
    let payment_methods: Vec<SavedPaymentMethod> = state.payment_methods
        .iter()
        .filter(|entry| entry.value().user_id == query.user_id)
        .map(|entry| entry.value().clone())
        .collect();
    
    tracing::info!("📋 Liste moyens paiement user {}: {} cartes", 
                  query.user_id, payment_methods.len());
    
    Ok(Json(payment_methods))
}

/// Supprimer un moyen de paiement
pub async fn delete_payment_method(
    State(state): State<AppState>,
    Path(pm_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let payment_method = state.payment_methods.get(&pm_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Moyen de paiement non trouvé".to_string() })
        ))?;
    
    // Détacher sur Stripe
    stripe_service::detach_payment_method(
        &state.stripe_client,
        &payment_method.stripe_payment_method_id,
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: format!("Erreur suppression: {}", e) })
    ))?;
    
    state.payment_methods.remove(&pm_id);
    
    tracing::info!("🗑️ Moyen de paiement supprimé: {}", pm_id);
    
    Ok(Json(serde_json::json!({
        "message": "Moyen de paiement supprimé avec succès",
        "payment_method_id": pm_id,
    })))
}

/// Payer avec un moyen de paiement sauvegardé
pub async fn pay_with_saved_method(
    State(state): State<AppState>,
    Json(req): Json<PayWithSavedMethodRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    // Vérifier que le moyen de paiement existe
    let payment_method = state.payment_methods.get(&req.payment_method_id)
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ApiError { error: "Moyen de paiement non trouvé".to_string() })
        ))?;
    
    if payment_method.user_id != req.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiError { error: "Ce moyen de paiement ne vous appartient pas".to_string() })
        ));
    }
    
    tracing::info!("Paiement {} pour user {} avec carte ****{}", 
                  req.amount, req.user_id, payment_method.card_last4);
    
    // Créer un PaymentIntent avec le moyen de paiement sauvegardé
    let payment_intent = stripe_service::create_payment_with_saved_method(
        &state.stripe_client,
        req.amount,
        &payment_method.stripe_payment_method_id,
        &req.description,
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: format!("Erreur paiement: {}", e) })
    ))?;
    
    Ok(Json(serde_json::json!({
        "payment_intent_id": payment_intent.id.to_string(),
        "status": format!("{:?}", payment_intent.status),
        "amount": req.amount,
        "card_last4": payment_method.card_last4,
    })))
}
