use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod config;
mod models;
mod routes;
mod services;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Charger la configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    tracing::info!("Démarrage du serveur RustStripe...");
    tracing::info!("Clé Stripe: {}...", &config.stripe_secret_key[..20]);

    // Créer l'état partagé de l'application
    let state = AppState::new(config);

    // Créer le routeur
    let app = Router::new()
        // Routes de santé
        .route("/", get(|| async { "RustStripe API - Exercices Stripe" }))
        .route("/health", get(|| async { "OK" }))
        
        // EXERCICE 1: Gestion de panier et paiement
        .route("/api/cart/add", post(routes::cart::add_to_cart))
        .route("/api/cart/view", get(routes::cart::view_cart))
        .route("/api/cart/checkout", post(routes::cart::checkout))
        .route("/api/orders/:order_id", get(routes::cart::get_order))
        .route("/api/orders/:order_id/cancel", post(routes::cart::cancel_order))
        .route("/api/orders/:order_id/update", post(routes::cart::update_order))
        .route("/api/orders", get(routes::cart::list_orders))
        
        // EXERCICE 2: Abonnements récurrents
        .route("/api/subscriptions/create", post(routes::subscriptions::create_subscription))
        .route("/api/subscriptions/:sub_id", get(routes::subscriptions::get_subscription))
        .route("/api/subscriptions/:sub_id/cancel", post(routes::subscriptions::cancel_subscription))
        
        // EXERCICE 3: Moyens de paiement
        .route("/api/payment-methods/setup", post(routes::payment_methods::setup_payment_method))
        .route("/api/payment-methods/list", get(routes::payment_methods::list_payment_methods))
        .route("/api/payment-methods/:pm_id/delete", post(routes::payment_methods::delete_payment_method))
        .route("/api/payment-methods/pay", post(routes::payment_methods::pay_with_saved_method))
        
        // Webhooks Stripe
        .route("/webhooks/stripe", post(routes::webhooks::stripe_webhook))
        
        // CORS et état
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Démarrer le serveur
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Serveur démarré sur http://{}", addr);
    tracing::info!(" Documentation des endpoints:");
    tracing::info!("  - Panier: POST /api/cart/add, GET /api/cart/view, POST /api/cart/checkout");
    tracing::info!("  - Abonnements: POST /api/subscriptions/create");
    tracing::info!("  - Moyens de paiement: POST /api/payment-methods/setup");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
