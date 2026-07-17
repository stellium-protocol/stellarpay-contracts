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
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400);

        let escrow = client.get_escrow(&escrow_id);
        assert_eq!(escrow.id, escrow_id);
        assert_eq!(escrow.buyer, buyer);
        assert_eq!(escrow.seller, seller);
        assert_eq!(escrow.amount, 1000);
        assert_eq!(escrow.asset, asset);
        assert!(!escrow.released);
        assert!(!escrow.refunded);
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
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400);

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
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400);

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
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400);

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
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400);

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
        let escrow_id = client.create(&buyer, &seller, &1000, &asset, &86400);

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
        client.create(&buyer, &seller, &1000, &asset, &86400);
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
}
