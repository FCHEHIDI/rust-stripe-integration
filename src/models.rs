use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ========== EXERCICE 1: Gestion de panier ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: i64, // En centimes
    pub stock: i32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: String,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub user_id: String,
    pub items: Vec<CartItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total: i64,
    pub status: OrderStatus,
    pub payment_intent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub price: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

// ========== EXERCICE 2: Abonnements ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub price: i64, // En centimes par mois
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubscription {
    pub id: String,
    pub user_id: String,
    pub plan_id: String,
    pub stripe_subscription_id: String,
    pub status: SubscriptionStatus,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub created_at: DateTime<Utc>,
    pub payment_method_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Cancelled,
    Incomplete,
}

// ========== EXERCICE 3: Moyens de paiement ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPaymentMethod {
    pub id: String,
    pub user_id: String,
    pub stripe_payment_method_id: String,
    pub card_last4: String,
    pub card_brand: String,
    pub exp_month: i32,
    pub exp_year: i32,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

// ========== Requêtes API ==========

#[derive(Debug, Deserialize)]
pub struct AddToCartRequest {
    pub user_id: String,
    pub product_id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderRequest {
    pub items: Vec<CartItem>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub user_id: String,
    pub plan_id: String,
    pub email: String,
    pub payment_method: String,
}

#[derive(Debug, Deserialize)]
pub struct SetupPaymentMethodRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PayWithSavedMethodRequest {
    pub user_id: String,
    pub payment_method_id: String,
    pub amount: i64,
    pub description: String,
}

// ========== Réponses API ==========

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub order_id: String,
    pub checkout_url: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub subscription_id: String,
    pub client_secret: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SetupResponse {
    pub setup_intent_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}
