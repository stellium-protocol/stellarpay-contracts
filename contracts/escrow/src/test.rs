#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
#[should_panic(expected = "escrow amount must be greater than zero")]
fn test_create_escrow_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let asset = Address::generate(&env);
    let timeout = 3600;

    client.create(&buyer, &seller, &0, &asset, &timeout, &None);
}

#[test]
#[should_panic(expected = "arbitrator cannot be buyer")]
fn test_create_escrow_arbitrator_same_as_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let asset = Address::generate(&env);
    let timeout = 3600;

    client.create(&buyer, &seller, &1000, &asset, &timeout, &Some(buyer.clone()));
}

#[test]
#[should_panic(expected = "arbitrator cannot be seller")]
fn test_create_escrow_arbitrator_same_as_seller() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let asset = Address::generate(&env);
    let timeout = 3600;

    client.create(&buyer, &seller, &1000, &asset, &timeout, &Some(seller.clone()));
}

