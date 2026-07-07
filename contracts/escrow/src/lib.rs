#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

const ESCROW_COUNTER: Symbol = symbol_short!("ESC_CNT");

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow(u64),
}

#[derive(Clone)]
#[contracttype]
pub struct Escrow {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub asset: Address,
    pub timeout: u64,
    pub released: bool,
    pub refunded: bool,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Create a new escrow. Funds are transferred from buyer into the contract.
    pub fn create(
        env: Env,
        buyer: Address,
        seller: Address,
        amount: i128,
        asset: Address,
        timeout: u64,
    ) -> u64 {
        buyer.require_auth();

        let mut counter: u64 = env.storage().instance().get(&ESCROW_COUNTER).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&ESCROW_COUNTER, &counter);

        let ledger_time = env.ledger().timestamp();
        let timeout_timestamp = ledger_time + timeout;

        let escrow = Escrow {
            id: counter,
            buyer: buyer.clone(),
            seller: seller.clone(),
            amount,
            asset: asset.clone(),
            timeout: timeout_timestamp,
            released: false,
            refunded: false,
        };

        env.storage().instance().set(&DataKey::Escrow(counter), &escrow);

        // Transfer asset from buyer to this contract
        let client = soroban_sdk::token::Client::new(&env, &asset);
        client.transfer(&buyer, &env.current_contract_address(), &amount);

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("created")),
            (counter, buyer, seller, amount),
        );

        counter
    }

    /// Release escrow funds to the seller. Only the buyer can release.
    pub fn release(env: Env, escrow_id: u64) {
        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        escrow.buyer.require_auth();
        assert!(!escrow.released, "already released");
        assert!(!escrow.refunded, "already refunded");

        escrow.released = true;
        env.storage()
            .instance()
            .set(&DataKey::Escrow(escrow_id), &escrow);

        let client = soroban_sdk::token::Client::new(&env, &escrow.asset);
        client.transfer(
            &env.current_contract_address(),
            &escrow.seller,
            &escrow.amount,
        );

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("released")),
            (escrow_id, escrow.seller, escrow.amount),
        );
    }

    /// Refund escrow to buyer. Only callable after timeout.
    pub fn refund(env: Env, escrow_id: u64) {
        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        escrow.buyer.require_auth();
        assert!(!escrow.released, "already released");
        assert!(!escrow.refunded, "already refunded");
        assert!(
            env.ledger().timestamp() >= escrow.timeout,
            "timeout not reached"
        );

        escrow.refunded = true;
        env.storage()
            .instance()
            .set(&DataKey::Escrow(escrow_id), &escrow);

        let client = soroban_sdk::token::Client::new(&env, &escrow.asset);
        client.transfer(
            &env.current_contract_address(),
            &escrow.buyer,
            &escrow.amount,
        );

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("refunded")),
            (escrow_id, escrow.buyer, escrow.amount),
        );
    }

    /// Get escrow details by ID.
    pub fn get_escrow(env: Env, escrow_id: u64) -> Escrow {
        env.storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found")
    }
}
