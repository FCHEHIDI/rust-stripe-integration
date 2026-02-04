use crate::config::Config;
use crate::models::*;
use dashmap::DashMap;
use std::sync::Arc;
use stripe::Client as StripeClient;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub stripe_client: StripeClient,
    
    // Base de données en mémoire (pour démo)
    pub products: Arc<DashMap<String, Product>>,
    pub carts: Arc<DashMap<String, Cart>>,
    pub orders: Arc<DashMap<String, Order>>,
    pub subscriptions: Arc<DashMap<String, UserSubscription>>,
    pub payment_methods: Arc<DashMap<String, SavedPaymentMethod>>,
    pub subscription_plans: Arc<DashMap<String, SubscriptionPlan>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let stripe_client = StripeClient::new(&config.stripe_secret_key);
        
        let state = Self {
            config,
            stripe_client,
            products: Arc::new(DashMap::new()),
            carts: Arc::new(DashMap::new()),
            orders: Arc::new(DashMap::new()),
            subscriptions: Arc::new(DashMap::new()),
            payment_methods: Arc::new(DashMap::new()),
            subscription_plans: Arc::new(DashMap::new()),
        };
        
        // Initialiser les données de démo
        state.init_demo_data();
        
        state
    }
    
    fn init_demo_data(&self) {
        // Produits (casquettes)
        let products = vec![
            Product {
                id: "cap_001".to_string(),
                name: "Casquette Classic Rouge".to_string(),
                price: 2500, // 25€
                stock: 50,
                description: "Casquette classique rouge, ajustable".to_string(),
            },
            Product {
                id: "cap_002".to_string(),
                name: "Casquette Sport Noire".to_string(),
                price: 3000, // 30€
                stock: 30,
                description: "Casquette sport noire, respirante".to_string(),
            },
            Product {
                id: "cap_003".to_string(),
                name: "Casquette Premium Blanche".to_string(),
                price: 4500, // 45€
                stock: 20,
                description: "Casquette premium en coton bio".to_string(),
            },
        ];
        
        for product in products {
            self.products.insert(product.id.clone(), product);
        }
        
        // Plans d'abonnement
        let plans = vec![
            SubscriptionPlan {
                id: "plan_normal".to_string(),
                name: "Normal".to_string(),
                price: 1000, // 10€
                description: "Abonnement journal formule normale".to_string(),
            },
            SubscriptionPlan {
                id: "plan_supplement".to_string(),
                name: "Supplément".to_string(),
                price: 1500, // 15€
                description: "Abonnement journal avec suppléments".to_string(),
            },
            SubscriptionPlan {
                id: "plan_complet".to_string(),
                name: "Complet".to_string(),
                price: 2000, // 20€
                description: "Abonnement journal formule complète".to_string(),
            },
        ];
        
        for plan in plans {
            self.subscription_plans.insert(plan.id.clone(), plan);
        }
        
        tracing::info!("✅ Données de démo initialisées: {} produits, {} plans", 
                      self.products.len(), self.subscription_plans.len());
    }
}
