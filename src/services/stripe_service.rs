// Service pour interagir avec l'API Stripe

use stripe::{
    Client, CreateCustomer, CreatePaymentIntent, CreatePrice, CreateProduct, 
    CreateSetupIntent, CreateSubscription, Currency, Customer, PaymentIntent, 
    PaymentIntentCaptureMethod, Price, Product, SetupIntent, Subscription,
    UpdateSubscription, PaymentMethod, StripeError, PaymentIntentOffSession,
};

/// Créer un PaymentIntent pour un paiement unique
pub async fn create_payment_intent(
    client: &Client,
    amount: i64,
    order_id: &str,
) -> Result<PaymentIntent, StripeError> {
    let mut params = CreatePaymentIntent::new(amount, Currency::EUR);
    params.capture_method = Some(PaymentIntentCaptureMethod::Automatic);
    params.metadata = Some(
        [("order_id".to_string(), order_id.to_string())]
            .iter()
            .cloned()
            .collect(),
    );
    
    PaymentIntent::create(client, params).await
}

/// Créer un client Stripe
pub async fn create_customer(
    client: &Client,
    email: &str,
    user_id: &str,
) -> Result<Customer, stripe::StripeError> {
    let mut params = CreateCustomer::new();
    params.email = Some(email);
    params.metadata = Some(
        [("user_id".to_string(), user_id.to_string())]
            .iter()
            .cloned()
            .collect(),
    );
    
    Customer::create(client, params).await
}

/// Créer un prix pour un abonnement
pub async fn create_subscription_price(
    client: &Client,
    product_name: &str,
    amount: i64,
) -> Result<String, StripeError> {
    // Créer le produit
    let mut product_params = CreateProduct::new(product_name);
    product_params.metadata = Some(
        [("type".to_string(), "subscription".to_string())]
            .iter()
            .cloned()
            .collect(),
    );
    
    let product = Product::create(client, product_params).await?;
    
    // Créer le prix avec recurring
    let mut price_params = CreatePrice::new(Currency::EUR);
    price_params.product = Some(stripe::IdOrCreate::Id(&product.id));
    price_params.unit_amount = Some(amount);
    
    // Configurer recurring pour un abonnement mensuel
    let mut recurring = stripe::CreatePriceRecurring::default();
    recurring.interval = stripe::CreatePriceRecurringInterval::Month;
    price_params.recurring = Some(recurring);
    
    let price = Price::create(client, price_params).await?;
    
    Ok(price.id.to_string())
}

/// Créer un abonnement
pub async fn create_subscription(
    client: &Client,
    customer_id: &str,
    price_id: &str,
    payment_method_id: Option<&str>,
) -> Result<Subscription, StripeError> {
    let customer_id: stripe::CustomerId = customer_id.parse().unwrap();
    let price_id: stripe::PriceId = price_id.parse().unwrap();
    
    let mut params = CreateSubscription::new(customer_id);
    
    // Ajouter le price via items
    let mut item = stripe::CreateSubscriptionItems::default();
    item.price = Some(price_id.to_string());
    params.items = Some(vec![item]);
    
    // Ajouter le payment method si fourni
    if let Some(pm_id) = payment_method_id {
        params.default_payment_method = Some(pm_id);
    }
    
    Subscription::create(client, params).await
}

/// Annuler un abonnement
pub async fn cancel_subscription(
    client: &Client,
    subscription_id: &str,
) -> Result<Subscription, StripeError> {
    let subscription_id = subscription_id.parse().unwrap();
    let mut params = UpdateSubscription::new();
    params.cancel_at_period_end = Some(true);
    
    Subscription::update(client, &subscription_id, params).await
}

/// Créer un SetupIntent pour enregistrer un moyen de paiement
pub async fn create_setup_intent(
    client: &Client,
    user_id: &str,
) -> Result<SetupIntent, StripeError> {
    let mut params = CreateSetupIntent::new();
    params.metadata = Some(
        [("user_id".to_string(), user_id.to_string())]
            .iter()
            .cloned()
            .collect(),
    );
    // usage field n'existe plus dans cette version
    
    SetupIntent::create(client, params).await
}

/// Détacher un moyen de paiement
pub async fn detach_payment_method(
    client: &Client,
    payment_method_id: &str,
) -> Result<PaymentMethod, stripe::StripeError> {
    let payment_method_id = payment_method_id.parse().unwrap();
    PaymentMethod::detach(client, &payment_method_id).await
}

/// Créer un paiement avec un moyen sauvegardé
pub async fn create_payment_with_saved_method(
    client: &Client,
    amount: i64,
    payment_method_id: &str,
    description: &str,
) -> Result<PaymentIntent, StripeError> {
    let payment_method_id = payment_method_id.parse().unwrap();
    
    let mut params = CreatePaymentIntent::new(amount, Currency::EUR);
    params.payment_method = Some(payment_method_id);
    params.confirm = Some(true);
    params.description = Some(description);
    params.off_session = Some(stripe::PaymentIntentOffSession::Exists(true));
    
    PaymentIntent::create(client, params).await
}
