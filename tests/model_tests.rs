// Tests unitaires pour les modèles

#[cfg(test)]
mod tests {
    use ruststripe::models::*;
    use chrono::Utc;
    
    #[test]
    fn test_product_creation() {
        let product = Product {
            id: "prod_123".to_string(),
            name: "Test Cap".to_string(),
            price: 2500,
            stock: 50,
            description: "A nice cap".to_string(),
        };
        
        assert_eq!(product.id, "prod_123");
        assert_eq!(product.price, 2500);
        assert!(product.stock > 0);
    }
    
    #[test]
    fn test_cart_item_serialization() {
        let item = CartItem {
            product_id: "prod_1".to_string(),
            quantity: 3,
        };
        
        assert_eq!(item.product_id, "prod_1");
        assert_eq!(item.quantity, 3);
    }
    
    #[test]
    fn test_order_item_total() {
        let order_item = OrderItem {
            product_id: "prod_1".to_string(),
            product_name: "Cap".to_string(),
            quantity: 2,
            price: 1500,
        };
        
        let total = order_item.quantity as i64 * order_item.price;
        assert_eq!(total, 3000);
    }
    
    #[test]
    fn test_subscription_status() {
        let status = SubscriptionStatus::Active;
        assert!(matches!(status, SubscriptionStatus::Active));
        
        let cancelled = SubscriptionStatus::Cancelled;
        assert!(matches!(cancelled, SubscriptionStatus::Cancelled));
    }
    
    #[test]
    fn test_order_status_transitions() {
        // Test de logique métier
        let initial = OrderStatus::Processing;
        assert!(matches!(initial, OrderStatus::Processing));
        
        // Normalement Processing -> Completed ou Failed
        let completed = OrderStatus::Completed;
        assert!(matches!(completed, OrderStatus::Completed));
        
        let failed = OrderStatus::Failed;
        assert!(matches!(failed, OrderStatus::Failed));
    }
    
    #[test]
    fn test_subscription_plan_validation() {
        let plan = SubscriptionPlan {
            id: "plan_basic".to_string(),
            name: "Basic".to_string(),
            price: 1000,
            description: "Basic plan".to_string(),
        };
        
        // Valider qu'un plan a un prix positif
        assert!(plan.price > 0);
        assert!(!plan.name.is_empty());
        assert!(!plan.id.is_empty());
    }
    
    #[test]
    fn test_cart_empty() {
        let cart = Cart {
            user_id: "user_1".to_string(),
            items: vec![],
            created_at: Utc::now(),
        };
        
        assert!(cart.items.is_empty());
        assert_eq!(cart.items.len(), 0);
    }
    
    #[test]
    fn test_cart_multiple_items() {
        let cart = Cart {
            user_id: "user_1".to_string(),
            items: vec![
                CartItem { product_id: "p1".to_string(), quantity: 2 },
                CartItem { product_id: "p2".to_string(), quantity: 1 },
                CartItem { product_id: "p3".to_string(), quantity: 5 },
            ],
            created_at: Utc::now(),
        };
        
        assert_eq!(cart.items.len(), 3);
        
        // Vérifier le total des quantités
        let total_quantity: i32 = cart.items.iter().map(|i| i.quantity).sum();
        assert_eq!(total_quantity, 8);
    }
    
    #[test]
    fn test_saved_payment_method() {
        let pm = SavedPaymentMethod {
            id: "pm_123".to_string(),
            user_id: "user_1".to_string(),
            stripe_payment_method_id: "pm_card_visa".to_string(),
            card_last4: "4242".to_string(),
            card_brand: "visa".to_string(),
            exp_month: 12,
            exp_year: 2026,
            is_default: false,
            created_at: Utc::now(),
        };
        
        assert_eq!(pm.card_last4.len(), 4);
        assert_eq!(pm.card_brand, "visa");
    }
    
    #[test]
    fn test_price_conversion() {
        // Stripe utilise les centimes
        let price_euros = 25; // 25€
        let price_cents = price_euros * 100;
        
        assert_eq!(price_cents, 2500);
        
        // Conversion inverse
        let cents = 4500;
        let euros = cents / 100;
        assert_eq!(euros, 45);
    }
}
