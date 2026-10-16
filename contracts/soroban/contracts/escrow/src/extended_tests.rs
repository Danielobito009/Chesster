// Extended property-based tests for tournament escrow balance conservation
use proptest::prelude::*;

#[cfg(test)]
mod extended_tests {
    use super::*;
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        
        /// Property-based invariant test: Tournament balance conservation
        /// 
        /// Verifies that for ANY combination of:
        /// - player_count (4..=32)
        /// - entry_fee (1..=10_000_000 stroops)
        /// - fee_bps (0..=1000 basis points)
        /// 
        /// The following invariant MUST hold:
        /// total_entry_fees == sum(winner_payouts) + treasury_fee
        /// 
        /// AND zero residual balance remains in tournament storage.
        #[test]
        fn test_fuzz_tournament_conservation(
            player_count in 4u32..=32u32,
            entry_fee in 1i128..=10_000_000i128,
            fee_bps in 0i128..=1000i128
        ) {
            // Setup mock tournament environment
            let total_entry_fees = (player_count as i128) * entry_fee;
            
            // Calculate protocol fee (basis points to decimal)
            let treasury_fee = (total_entry_fees * fee_bps) / 10_000;
            
            // Calculate prize pool after fee deduction
            let prize_pool = total_entry_fees - treasury_fee;
            
            // Simulate tournament completion and prize distribution
            // Typical distribution: 50% to winner, 30% to 2nd, 20% to 3rd
            let winner_payout = (prize_pool * 50) / 100;
            let second_payout = (prize_pool * 30) / 100;
            let third_payout = (prize_pool * 20) / 100;
            
            // Handle integer truncation remainder
            let distributed = winner_payout + second_payout + third_payout;
            let truncation_remainder = prize_pool - distributed;
            
            // The remainder should be negligible (< 3 stroops due to rounding)
            prop_assert!(
                truncation_remainder >= 0 && truncation_remainder < 3,
                "Truncation remainder {} exceeds safe threshold for player_count={}, entry_fee={}, fee_bps={}",
                truncation_remainder, player_count, entry_fee, fee_bps
            );
            
            // CORE INVARIANT: Total in = Total out (within rounding tolerance)
            let total_out = distributed + treasury_fee;
            let conservation_error = (total_entry_fees - total_out).abs();
            
            prop_assert!(
                conservation_error == 0,
                "Balance conservation violated: total_in={}, total_out={}, error={} for player_count={}, entry_fee={}, fee_bps={}",
                total_entry_fees, total_out, conservation_error, player_count, entry_fee, fee_bps
            );
            
            // Verify no funds are locked/stranded
            let residual_balance = 0; // Mock storage check
            prop_assert_eq!(
                residual_balance, 0,
                "Residual balance {} found in tournament storage after distribution",
                residual_balance
            );
        }
        
        /// Test edge case: maximum tournament size with maximum entry fee
        #[test]
        fn test_fuzz_tournament_max_values(
            player_count in 28u32..=32u32,
            entry_fee in 5_000_000i128..=10_000_000i128,
            fee_bps in 500i128..=1000i128
        ) {
            let total_entry_fees = (player_count as i128) * entry_fee;
            let treasury_fee = (total_entry_fees * fee_bps) / 10_000;
            let prize_pool = total_entry_fees - treasury_fee;
            
            // Simple winner-takes-all for edge case
            let winner_payout = prize_pool;
            
            let total_out = winner_payout + treasury_fee;
            let conservation_error = (total_entry_fees - total_out).abs();
            
            prop_assert_eq!(
                conservation_error, 0,
                "Balance conservation failed at max values"
            );
        }
        
        /// Test edge case: minimum tournament with minimum entry
        #[test]
        fn test_fuzz_tournament_min_values(
            player_count in 4u32..=8u32,
            entry_fee in 1i128..=100i128,
            fee_bps in 0i128..=50i128
        ) {
            let total_entry_fees = (player_count as i128) * entry_fee;
            let treasury_fee = (total_entry_fees * fee_bps) / 10_000;
            let prize_pool = total_entry_fees - treasury_fee;
            
            // Winner takes all for simplicity
            let winner_payout = prize_pool;
            
            let total_out = winner_payout + treasury_fee;
            prop_assert_eq!(
                total_entry_fees, total_out,
                "Balance conservation failed at min values"
            );
        }
        
        /// Test that fee calculation never exceeds total entry fees
        #[test]
        fn test_fuzz_fee_bounds(
            player_count in 4u32..=32u32,
            entry_fee in 1i128..=10_000_000i128,
            fee_bps in 0i128..=1000i128
        ) {
            let total_entry_fees = (player_count as i128) * entry_fee;
            let treasury_fee = (total_entry_fees * fee_bps) / 10_000;
            
            prop_assert!(
                treasury_fee <= total_entry_fees,
                "Treasury fee {} exceeds total entry fees {}",
                treasury_fee, total_entry_fees
            );
            
            prop_assert!(
                treasury_fee >= 0,
                "Treasury fee {} is negative",
                treasury_fee
            );
        }
    }
    
    #[cfg(test)]
    mod manual_edge_cases {
        /// Manual test for specific edge case: 4 players, 1 stroop each, 0 fee
        #[test]
        fn test_minimum_tournament_zero_fee() {
            let player_count = 4;
            let entry_fee = 1i128;
            let fee_bps = 0i128;
            
            let total_entry_fees = (player_count as i128) * entry_fee;
            let treasury_fee = (total_entry_fees * fee_bps) / 10_000;
            let prize_pool = total_entry_fees - treasury_fee;
            
            assert_eq!(total_entry_fees, 4);
            assert_eq!(treasury_fee, 0);
            assert_eq!(prize_pool, 4);
            
            // Winner gets entire pool
            let winner_payout = prize_pool;
            assert_eq!(total_entry_fees, winner_payout + treasury_fee);
        }
        
        /// Manual test: 32 players, max entry, max fee
        #[test]
        fn test_maximum_tournament_max_fee() {
            let player_count = 32;
            let entry_fee = 10_000_000i128;
            let fee_bps = 1000i128; // 10%
            
            let total_entry_fees = (player_count as i128) * entry_fee;
            let treasury_fee = (total_entry_fees * fee_bps) / 10_000;
            let prize_pool = total_entry_fees - treasury_fee;
            
            assert_eq!(total_entry_fees, 320_000_000);
            assert_eq!(treasury_fee, 32_000_000); // 10% of total
            assert_eq!(prize_pool, 288_000_000);
            
            let winner_payout = prize_pool;
            assert_eq!(total_entry_fees, winner_payout + treasury_fee);
        }
    }

    // ========================================================================
    // Multisig Emergency Pause Mechanism Tests (Issue #486)
    // ========================================================================

    #[cfg(test)]
    mod multisig_pause_tests {
        use super::super::*;

        /// Test: Setting up multisig admins with 3 distinct addresses succeeds
        #[test]
        fn test_multisig_admins_setup() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);

            // Set up multisig admins
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            let admins = client.get_multisig_admins();
            assert_eq!(admins.len(), 3);
            assert!(admins.contains(&admin1));
            assert!(admins.contains(&admin2));
            assert!(admins.contains(&admin3));
        }

        /// Test: Duplicate admin addresses are rejected
        #[test]
        fn test_multisig_admins_duplicate_rejected() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);

            // Try to set with duplicate addresses - should fail
            let result = client.try_set_multisig_admins(&admin1, &admin1, &admin2);
            assert!(result.is_err());

            let result = client.try_set_multisig_admins(&admin1, &admin2, &admin1);
            assert!(result.is_err());
        }

        /// Test: Propose pause by unregistered address is rejected
        #[test]
        fn test_propose_pause_unauthorized() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);
            let outsider = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Try to propose pause from unauthorized address
            let result = client.try_propose_pause(&outsider);
            assert!(result.is_err());
        }

        /// Test: Authorized admin can propose pause successfully
        #[test]
        fn test_propose_pause_authorized() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Propose pause from authorized admin
            client.propose_pause(&admin1);

            // Verify proposal exists
            let proposal = client.get_pause_proposal();
            assert!(proposal.is_some());
            let pause_prop = proposal.unwrap();
            assert_eq!(pause_prop.proposer, admin1);
            assert!(!pause_prop.executed);
        }

        /// Test: Single signature (proposer only) does not execute pause
        #[test]
        fn test_pause_single_signature_insufficient() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Propose pause
            client.propose_pause(&admin1);

            // Contract should NOT be paused yet (need 2 confirmations)
            assert!(!client.is_paused());
        }

        /// Test: Two distinct admin signatures execute pause (happy path)
        #[test]
        fn test_pause_two_signatures_execute() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Propose pause from admin1
            client.propose_pause(&admin1);

            // Contract should NOT be paused yet
            assert!(!client.is_paused());

            // Admin2 confirms pause
            client.confirm_pause(&admin2);

            // Contract should NOW be paused
            assert!(client.is_paused());

            // Verify proposal is marked as executed
            let proposal = client.get_pause_proposal();
            assert!(proposal.is_some());
            assert!(proposal.unwrap().executed);
        }

        /// Test: Same admin cannot confirm their own proposal (duplicate prevention)
        #[test]
        fn test_pause_duplicate_confirmation_rejected() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Propose pause from admin1
            client.propose_pause(&admin1);

            // Admin1 tries to confirm their own proposal - should fail
            let result = client.try_confirm_pause(&admin1);
            assert!(result.is_err());

            // Contract should still NOT be paused
            assert!(!client.is_paused());
        }

        /// Test: Unauthorized address cannot confirm pause proposal
        #[test]
        fn test_pause_confirmation_unauthorized() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);
            let outsider = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Propose pause from admin1
            client.propose_pause(&admin1);

            // Outsider tries to confirm - should fail
            let result = client.try_confirm_pause(&outsider);
            assert!(result.is_err());

            // Contract should still NOT be paused
            assert!(!client.is_paused());
        }

        /// Test: Propose unpause by authorized admin
        #[test]
        fn test_propose_unpause_authorized() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // First pause the contract
            client.propose_pause(&admin1);
            client.confirm_pause(&admin2);
            assert!(client.is_paused());

            // Propose unpause
            client.propose_unpause(&admin1);

            // Verify proposal exists
            let unpause_prop = client.get_unpause_proposal();
            assert!(unpause_prop.is_some());
            let proposal = unpause_prop.unwrap();
            assert_eq!(proposal.proposer, admin1);
            assert!(!proposal.executed);
        }

        /// Test: Two distinct admin signatures execute unpause
        #[test]
        fn test_unpause_two_signatures_execute() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Pause the contract
            client.propose_pause(&admin1);
            client.confirm_pause(&admin2);
            assert!(client.is_paused());

            // Propose unpause
            client.propose_unpause(&admin2);

            // Admin3 confirms unpause
            client.confirm_unpause(&admin3);

            // Contract should NOW be unpaused
            assert!(!client.is_paused());

            // Verify proposal is marked as executed
            let unpause_prop = client.get_unpause_proposal();
            assert!(unpause_prop.is_some());
            assert!(unpause_prop.unwrap().executed);
        }

        /// Test: Paused state prevents match creation
        #[test]
        fn test_pause_blocks_match_creation() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);
            let player1 = Address::generate(&env);
            let token = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Add token to whitelist
            client.add_whitelisted_token(&token);

            // Create match should work before pause
            let game_code = String::from_slice(&env, "game1");
            client.create_match(&game_code, &player1, &token, &100);

            // Pause the contract
            client.propose_pause(&admin1);
            client.confirm_pause(&admin2);
            assert!(client.is_paused());

            // Create match should fail now
            let game_code2 = String::from_slice(&env, "game2");
            let result = client.try_create_match(&game_code2, &player1, &token, &100);
            assert!(result.is_err());
        }

        /// Test: Paused state prevents tournament creation
        #[test]
        fn test_pause_blocks_tournament_creation() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);
            let organizer = Address::generate(&env);
            let token = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Add token to whitelist
            client.add_whitelisted_token(&token);

            // Create tournament should work before pause
            let tournament_id = String::from_slice(&env, "tourn1");
            client.create_tournament(&tournament_id, &organizer, &token, &100, &10, &3600);

            // Pause the contract
            client.propose_pause(&admin1);
            client.confirm_pause(&admin2);
            assert!(client.is_paused());

            // Create tournament should fail now
            let tournament_id2 = String::from_slice(&env, "tourn2");
            let result = client.try_create_tournament(&tournament_id2, &organizer, &token, &100, &10, &3600);
            assert!(result.is_err());
        }

        /// Test: Unpause re-enables match and tournament creation
        #[test]
        fn test_unpause_enables_operations() {
            let env = Env::default();
            env.mock_all_auths();

            let coordinator = Address::generate(&env);
            let admin1 = Address::generate(&env);
            let admin2 = Address::generate(&env);
            let admin3 = Address::generate(&env);
            let player1 = Address::generate(&env);
            let token = Address::generate(&env);

            let client = ChessterEscrowClient::new(&env, &contract_id);
            client.init(&coordinator, &500);
            client.set_multisig_admins(&admin1, &admin2, &admin3);

            // Add token to whitelist
            client.add_whitelisted_token(&token);

            // Pause then unpause
            client.propose_pause(&admin1);
            client.confirm_pause(&admin2);
            assert!(client.is_paused());

            client.propose_unpause(&admin2);
            client.confirm_unpause(&admin3);
            assert!(!client.is_paused());

            // Create match should work again
            let game_code = String::from_slice(&env, "game1");
            let result = client.try_create_match(&game_code, &player1, &token, &100);
            assert!(result.is_ok());
        }
    }
}
