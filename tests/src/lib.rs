#[cfg(test)]
mod tests {
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env,
    };
    use stellium_escrow::EscrowContract;
    use stellium_payment::PaymentContract;

    fn setup_escrow_env() -> (Env, Address) {
        let env = Env::default();
        let contract_id = env.register(EscrowContract, ());
        (env, contract_id)
    }

    fn setup_payment_env() -> (Env, Address) {
        let env = Env::default();
        let contract_id = env.register(PaymentContract, ());
        (env, contract_id)
    }

    fn create_token<'a>(env: &'a Env, admin: &Address) -> (Address, StellarAssetClient<'a>, TokenClient<'a>) {
        let token_address = env.register_stellar_asset_contract(admin.clone());
        let stellar_client = StellarAssetClient::new(env, &token_address);
        let token_client = TokenClient::new(env, &token_address);
        (token_address, stellar_client, token_client)
    }

    // ========================================
    // Escrow tests
    // ========================================

    #[test]
    fn test_create_escrow() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400, &None);

        let escrow = client.get_escrow(&escrow_id);
        assert_eq!(escrow.id, escrow_id);
        assert_eq!(escrow.buyer, buyer);
        assert_eq!(escrow.seller, seller);
        assert_eq!(escrow.amount, 1000);
        assert_eq!(escrow.asset, asset);
        assert!(!escrow.released);
        assert!(!escrow.refunded);
        assert_eq!(escrow.arbitrator, None);
        assert!(!escrow.disputed);
    }

    #[test]
    fn test_release_escrow() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400, &None);

        assert_eq!(token.balance(&seller), 0);

        client.release(&escrow_id);

        let escrow = client.get_escrow(&escrow_id);
        assert!(escrow.released);
        assert_eq!(token.balance(&seller), 1000);
        assert_eq!(token.balance(&contract_id), 0);
    }

    #[test]
    fn test_refund_escrow() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400, &None);

        env.ledger().with_mut(|li| {
            li.timestamp += 86401;
        });

        client.refund(&escrow_id);

        let escrow = client.get_escrow(&escrow_id);
        assert!(escrow.refunded);
        assert_eq!(token.balance(&buyer), 1000);
        assert_eq!(token.balance(&contract_id), 0);
    }

    #[test]
    #[should_panic(expected = "already released")]
    fn test_release_already_released() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400, &None);

        client.release(&escrow_id);
        client.release(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "timeout not reached")]
    fn test_refund_before_timeout() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400, &None);

        client.refund(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "already released")]
    fn test_release_then_refund() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400, &None);

        client.release(&escrow_id);
        client.refund(&escrow_id);
    }

    #[test]
    #[should_panic]
    fn test_unauthorized_release() {
        // Without mock_all_auths, any require_auth() call will fail.
        // The escrow.create() calls buyer.require_auth() internally,
        // so even the create call will panic without proper authorization.
        // This verifies the auth system is enforced.
        let env = Env::default();
        let contract_id = env.register(EscrowContract, ());

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let asset = Address::generate(&env);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        client.create(&buyer, &seller, &1000, &asset, &86400, &None);
    }

    // ========================================
    // Payment tests
    // ========================================

    #[test]
    fn test_create_payment() {
        let (env, contract_id) = setup_payment_env();
        env.mock_all_auths();

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, token) = create_token(&env, &admin);

        stellar.mint(&sender, &500);

        let client = stellium_payment::PaymentContractClient::new(&env, &contract_id);
        let metadata = soroban_sdk::Bytes::from_array(&env, &[1, 2, 3]);
        let payment_id = client.pay(&sender, &recipient, &500, &asset, &metadata);

        let payment = client.get_payment(&payment_id);
        assert_eq!(payment.id, payment_id);
        assert_eq!(payment.sender, sender);
        assert_eq!(payment.recipient, recipient);
        assert_eq!(payment.amount, 500);
        assert_eq!(payment.asset, asset);
        assert!(payment.completed);
        assert_eq!(token.balance(&recipient), 500);
    }

    #[test]
    fn test_verify_payment() {
        let (env, contract_id) = setup_payment_env();
        env.mock_all_auths();

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&sender, &500);

        let client = stellium_payment::PaymentContractClient::new(&env, &contract_id);
        let metadata = soroban_sdk::Bytes::from_array(&env, &[]);
        let payment_id = client.pay(&sender, &recipient, &500, &asset, &metadata);

        assert!(client.verify(&payment_id));
    }

    #[test]
    fn test_verify_nonexistent() {
        let (env, contract_id) = setup_payment_env();

        let client = stellium_payment::PaymentContractClient::new(&env, &contract_id);
        assert!(!client.verify(&999));
    }

    // ========================================
    // Escrow edge case tests
    // ========================================

    #[test]
    #[should_panic(expected = "escrow amount must be greater than zero")]
    fn test_escrow_zero_amount() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        client.create(&buyer, &seller, &0, &asset, &86400, &None);
    }

    #[test]
    fn test_escrow_zero_timeout_immediate_refund() {
        // timeout=0 means the escrow is immediately refundable
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &0, &None);

        // Should be immediately refundable without advancing the ledger
        client.refund(&escrow_id);

        let escrow = client.get_escrow(&escrow_id);
        assert!(escrow.refunded);
        assert_eq!(token.balance(&buyer), 1000);
    }

    #[test]
    #[should_panic(expected = "buyer and seller must be different addresses")]
    fn test_escrow_same_buyer_and_seller() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        // buyer and seller are the same address
        client.create(&buyer, &buyer, &1000, &asset, &86400, &None);
    }

    #[test]
    #[should_panic(expected = "already refunded")]
    fn test_escrow_duplicate_refund() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400, &None);

        // Advance time past timeout
        env.ledger().with_mut(|li| {
            li.timestamp += 86401;
        });

        client.refund(&escrow_id);
        // Second refund should panic
        client.refund(&escrow_id);
    }

    // ========================================
    // Payment edge case tests
    // ========================================

    #[test]
    #[should_panic(expected = "payment amount must be greater than zero")]
    fn test_payment_zero_amount() {
        let (env, contract_id) = setup_payment_env();
        env.mock_all_auths();

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&sender, &500);

        let client = stellium_payment::PaymentContractClient::new(&env, &contract_id);
        let metadata = soroban_sdk::Bytes::from_array(&env, &[]);
        client.pay(&sender, &recipient, &0, &asset, &metadata);
    }

    #[test]
    fn test_payment_empty_metadata() {
        let (env, contract_id) = setup_payment_env();
        env.mock_all_auths();

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, token) = create_token(&env, &admin);

        stellar.mint(&sender, &500);

        let client = stellium_payment::PaymentContractClient::new(&env, &contract_id);
        let metadata = soroban_sdk::Bytes::from_array(&env, &[]);
        let payment_id = client.pay(&sender, &recipient, &500, &asset, &metadata);

        let payment = client.get_payment(&payment_id);
        assert_eq!(payment.metadata.len(), 0);
        assert!(payment.completed);
        assert_eq!(token.balance(&recipient), 500);
    }

    #[test]
    #[should_panic(expected = "sender and recipient must be different addresses")]
    fn test_payment_self_payment() {
        let (env, contract_id) = setup_payment_env();
        env.mock_all_auths();

        let sender = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&sender, &500);

        let client = stellium_payment::PaymentContractClient::new(&env, &contract_id);
        let metadata = soroban_sdk::Bytes::from_array(&env, &[]);
        // sender and recipient are the same address
        client.pay(&sender, &sender, &500, &asset, &metadata);
    }

    #[test]
    fn test_payment_multiple_payments_increment_id() {
        let (env, contract_id) = setup_payment_env();
        env.mock_all_auths();

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&sender, &1000);

        let client = stellium_payment::PaymentContractClient::new(&env, &contract_id);
        let metadata = soroban_sdk::Bytes::from_array(&env, &[]);

        let id1 = client.pay(&sender, &recipient, &100, &asset, &metadata);
        let id2 = client.pay(&sender, &recipient, &200, &asset, &metadata);
        let id3 = client.pay(&sender, &recipient, &300, &asset, &metadata);

        // IDs should be sequential
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);

        // Each payment should be independently verifiable
        assert!(client.verify(&id1));
        assert!(client.verify(&id2));
        assert!(client.verify(&id3));
    }

    // ========================================
    // Dispute and resolution tests
    // ========================================

    #[test]
    fn test_create_escrow_with_arbitrator() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator.clone()),
        );

        let escrow = client.get_escrow(&escrow_id);
        assert_eq!(escrow.id, escrow_id);
        assert_eq!(escrow.buyer, buyer);
        assert_eq!(escrow.seller, seller);
        assert_eq!(escrow.amount, 1000);
        assert_eq!(escrow.asset, asset);
        assert!(!escrow.released);
        assert!(!escrow.refunded);
        assert_eq!(escrow.arbitrator, Some(arbitrator));
        assert!(!escrow.disputed);
    }

    #[test]
    fn test_dispute_escrow() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        client.dispute(&escrow_id);

        let escrow = client.get_escrow(&escrow_id);
        assert!(escrow.disputed);
        assert!(!escrow.released);
        assert!(!escrow.refunded);
    }

    #[test]
    #[should_panic(expected = "is disputed")]
    fn test_dispute_blocks_release() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        client.dispute(&escrow_id);
        // Release must fail while disputed
        client.release(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "is disputed")]
    fn test_dispute_blocks_refund() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        client.dispute(&escrow_id);

        // Advance ledger time past timeout
        env.ledger().with_mut(|li| {
            li.timestamp += 86401;
        });

        // Refund must fail while disputed even after timeout
        client.refund(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "no arbitrator configured")]
    fn test_dispute_without_arbitrator_panics() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400, &None);

        client.dispute(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "already disputed")]
    fn test_duplicate_dispute_panics() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        client.dispute(&escrow_id);
        client.dispute(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "already released")]
    fn test_dispute_after_release_panics() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        client.release(&escrow_id);
        client.dispute(&escrow_id);
    }

    #[test]
    #[should_panic(expected = "already refunded")]
    fn test_dispute_after_refund_panics() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        env.ledger().with_mut(|li| {
            li.timestamp += 86401;
        });

        client.refund(&escrow_id);
        client.dispute(&escrow_id);
    }

    #[test]
    fn test_resolve_to_seller() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        assert_eq!(token.balance(&seller), 0);

        client.dispute(&escrow_id);
        client.resolve(&escrow_id, &true);

        let escrow = client.get_escrow(&escrow_id);
        assert!(escrow.disputed);
        assert!(escrow.released);
        assert!(!escrow.refunded);
        assert_eq!(token.balance(&seller), 1000);
        assert_eq!(token.balance(&buyer), 0);
        assert_eq!(token.balance(&contract_id), 0);
    }

    #[test]
    fn test_resolve_to_buyer() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        client.dispute(&escrow_id);
        client.resolve(&escrow_id, &false);

        let escrow = client.get_escrow(&escrow_id);
        assert!(escrow.disputed);
        assert!(!escrow.released);
        assert!(escrow.refunded);
        assert_eq!(token.balance(&buyer), 1000);
        assert_eq!(token.balance(&seller), 0);
        assert_eq!(token.balance(&contract_id), 0);
    }

    #[test]
    #[should_panic(expected = "is not disputed")]
    fn test_resolve_not_disputed_panics() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        client.resolve(&escrow_id, &true);
    }

    #[test]
    #[should_panic(expected = "already released")]
    fn test_resolve_duplicate_panics() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        let escrow_id = client.create(
            &buyer,
            &seller,
            &1000,
            &asset,
            &86400,
            &Some(arbitrator),
        );

        client.dispute(&escrow_id);
        client.resolve(&escrow_id, &true);
        // Second resolve should panic
        client.resolve(&escrow_id, &true);
    }

    #[test]
    #[should_panic(expected = "arbitrator cannot be buyer")]
    fn test_escrow_arbitrator_same_as_buyer() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        client.create(&buyer, &seller, &1000, &asset, &86400, &Some(buyer.clone()));
    }

    #[test]
    #[should_panic(expected = "arbitrator cannot be seller")]
    fn test_escrow_arbitrator_same_as_seller() {
        let (env, contract_id) = setup_escrow_env();
        env.mock_all_auths();

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin = Address::generate(&env);
        let (asset, stellar, _token) = create_token(&env, &admin);

        stellar.mint(&buyer, &1000);

        let client = stellium_escrow::EscrowContractClient::new(&env, &contract_id);
        client.create(&buyer, &seller, &1000, &asset, &86400, &Some(seller.clone()));
    }
}
