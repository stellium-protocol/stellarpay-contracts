#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env, Bytes};

#[test]
#[should_panic(expected = "amount must be greater than zero")]
fn test_pay_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PaymentContract);
    let client = PaymentContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let asset = Address::generate(&env);
    let metadata = Bytes::new(&env);

    client.pay(&sender, &recipient, &0, &asset, &metadata);
}
