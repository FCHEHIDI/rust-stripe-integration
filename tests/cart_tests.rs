// Tests d'intégration pour le panier

#[cfg(test)]
mod tests {
    use ruststripe::models::{Product, SubscriptionPlan};
    use ruststripe::state::AppState;
    use ruststripe::config::Config;
    
    fn create_test_state() -> AppState {
        let config = Config {
            stripe_secret_key: "sk_test_fake".to_string(),
            stripe_webhook_secret: "whsec_test".to_string(),
            base_url: "http://localhost:3000".to_string(),
        };
        let state = AppState::new(config);
        
        // Ajouter des produits de test
        state.products.insert("test_prod_1".to_string(), Product {
            id: "test_prod_1".to_string(),
            name: "Test Product".to_string(),
            price: 1000,
            stock: 10,
            description: "A test product".to_string(),
        });
        
        state.products.insert("test_prod_2".to_string(), Product {
            id: "test_prod_2".to_string(),
            name: "Out of Stock Product".to_string(),
            price: 2000,
            stock: 0,
            description: "No stock".to_string(),
        });
        
        // Ajouter des plans de test
        state.subscription_plans.insert("test_plan".to_string(), SubscriptionPlan {
            id: "test_plan".to_string(),
            name: "Test Plan".to_string(),
            price: 1500,
            description: "Test subscription".to_string(),
        });
        
        state
    }
    
    #[test]
    fn test_product_initialization() {
        let state = create_test_state();
        
        // Le state inclut les produits de test + les produits de démo (3)
        assert!(state.products.len() >= 2);
        assert!(state.products.contains_key("test_prod_1"));
        
        let product = state.products.get("test_prod_1").unwrap();
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.price, 1000);
        assert_eq!(product.stock, 10);
    }
    
    #[test]
    fn test_subscription_plan_initialization() {
        let state = create_test_state();
        
        // Le state inclut les plans de test + les plans de démo (3)
        assert!(state.subscription_plans.len() >= 1);
        
        let plan = state.subscription_plans.get("test_plan").unwrap();
        assert_eq!(plan.name, "Test Plan");
        assert_eq!(plan.price, 1500);
    }
    
    #[test]
    fn test_cart_operations() {
        let state = create_test_state();
        let user_id = "test_user";
        
        // Vérifier qu'il n'y a pas de panier initialement
        assert!(!state.carts.contains_key(user_id));
        
        // Le panier sera créé par les routes, ici on teste juste l'état
        assert_eq!(state.carts.len(), 0);
    }
    
    #[test]
    fn test_product_stock_check() {
        let state = create_test_state();
        
        let in_stock = state.products.get("test_prod_1").unwrap();
        assert!(in_stock.stock > 0);
        
        let out_of_stock = state.products.get("test_prod_2").unwrap();
        assert_eq!(out_of_stock.stock, 0);
    }
    
    #[test]
    fn test_price_calculations() {
        let state = create_test_state();
        let product = state.products.get("test_prod_1").unwrap();
        
        // Test calcul simple
        let quantity = 3;
        let total = product.price * quantity;
        assert_eq!(total, 3000);
        
        // Test avec 0 quantité
        assert_eq!(product.price * 0, 0);
    }
    
    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;
        
        let state = Arc::new(create_test_state());
        let mut handles = vec![];
        
        // Simuler plusieurs threads accédant au state
        for i in 0..5 {
            let state_clone = Arc::clone(&state);
            let handle = thread::spawn(move || {
                let product = state_clone.products.get("test_prod_1");
                assert!(product.is_some());
                format!("Thread {} OK", i)
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
    
    #[test]
    fn test_state_isolation() {
        let state1 = create_test_state();
        let state2 = create_test_state();
        
        // Modifier state1 ne devrait pas affecter state2
        state1.products.insert("new_product".to_string(), Product {
            id: "new_product".to_string(),
            name: "New".to_string(),
            price: 500,
            stock: 5,
            description: "Test".to_string(),
        });
        
        assert!(state1.products.contains_key("new_product"));
        assert!(!state2.products.contains_key("new_product"));
    }
}

