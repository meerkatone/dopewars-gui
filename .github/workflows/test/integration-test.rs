#[cfg(test)]
mod integration_tests {
    use std::collections::HashMap;
    
    // Import the core game structures and functions
    // This assumes you've made them public or that tests have access to them
    use crate::{Market, Player, SUBSTANCES, MAX_CARRYING_CAPACITY};
    
    #[test]
    fn test_player_initialization() {
        let player = Player::new();
        
        // Test initial values
        assert_eq!(player.cash, 2000);
        assert_eq!(player.debt, 5000);
        assert_eq!(player.day, 1);
        assert_eq!(player.health, 100);
        
        // Test inventory initialization
        for substance in SUBSTANCES.iter() {
            assert_eq!(player.inventory.get(&substance.to_string()), Some(&0));
        }
    }
    
    #[test]
    fn test_market_initialization() {
        let market = Market::new();
        
        // Test price history initialization
        for substance in SUBSTANCES.iter() {
            assert!(market.price_history.contains_key(&substance.to_string()));
            assert!(market.price_history.get(&substance.to_string()).unwrap().is_empty());
        }
        
        // Test events initialization
        assert!(market.events.is_empty());
    }
    
    #[test]
    fn test_market_price_generation() {
        let mut market = Market::new();
        market.generate_prices();
        
        // Test that prices are generated for all substances
        for substance in SUBSTANCES.iter() {
            assert!(market.prices.contains_key(&substance.to_string()));
            let price = market.prices.get(&substance.to_string()).unwrap();
            
            // Test that prices are within expected ranges
            match *substance {
                "Weed" => assert!(*price >= 2 && *price <= 500), // Allow for price events
                "Cocaine" => assert!(*price >= 20 && *price <= 5000),
                "Ludes" => assert!(*price >= 4 && *price <= 1000),
                "Acid" => assert!(*price >= 10 && *price <= 2000),
                "Heroin" => assert!(*price >= 30 && *price <= 7500),
                "Speed" => assert!(*price >= 10 && *price <= 3500),
                _ => panic!("Unexpected substance: {}", substance),
            }
        }
        
        // Test that price history is updated
        for substance in SUBSTANCES.iter() {
            let history = market.price_history.get(&substance.to_string()).unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0], *market.prices.get(&substance.to_string()).unwrap());
        }
    }
    
    #[test]
    fn test_player_inventory_functions() {
        let mut player = Player::new();
        
        // Start with empty inventory
        assert_eq!(player.total_items(), 0);
        assert_eq!(player.space_available(), MAX_CARRYING_CAPACITY);
        
        // Add items to inventory
        for (i, substance) in SUBSTANCES.iter().enumerate() {
            *player.inventory.entry(substance.to_string()).or_insert(0) += i as i32 + 1;
        }
        
        // Calculate expected total
        let expected_total: i32 = (1..=SUBSTANCES.len()).sum::<usize>() as i32;
        assert_eq!(player.total_items(), expected_total);
        assert_eq!(player.space_available(), MAX_CARRYING_CAPACITY - expected_total);
    }
    
    #[test]
    fn test_market_price_history() {
        let mut market = Market::new();
        
        // Generate prices multiple times to build history
        for _ in 0..15 {
            market.generate_prices();
        }
        
        // Check history length is capped at max_history
        for substance in SUBSTANCES.iter() {
            let history = market.price_history.get(&substance.to_string()).unwrap();
            assert_eq!(history.len(), market.max_history);
        }
    }
}
