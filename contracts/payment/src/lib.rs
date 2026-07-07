#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol};

const PAYMENT_COUNTER: Symbol = symbol_short!("PAY_CNT");

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Payment(u64),
}

#[derive(Clone)]
#[contracttype]
pub struct Payment {
    pub id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub amount: i128,
    pub asset: Address,
    pub metadata: Bytes,
    pub completed: bool,
}

#[contract]
pub struct PaymentContract;

#[contractimpl]
impl PaymentContract {
    /// Create and execute a direct payment.
    pub fn pay(
        env: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        asset: Address,
        metadata: Bytes,
    ) -> u64 {
        sender.require_auth();

        let mut counter: u64 = env.storage().instance().get(&PAYMENT_COUNTER).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&PAYMENT_COUNTER, &counter);

        let payment = Payment {
            id: counter,
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount,
            asset: asset.clone(),
            metadata,
            completed: true,
        };

        env.storage()
            .instance()
            .set(&DataKey::Payment(counter), &payment);

        let client = soroban_sdk::token::Client::new(&env, &asset);
        client.transfer(&sender, &recipient, &amount);

        env.events().publish(
            (symbol_short!("payment"), symbol_short!("created")),
            (counter, sender, recipient, amount),
        );

        counter
    }

    /// Get payment details by ID.
    pub fn get_payment(env: Env, payment_id: u64) -> Payment {
        env.storage()
            .instance()
            .get(&DataKey::Payment(payment_id))
            .expect("payment not found")
    }

    /// Check if a payment exists and is completed.
    pub fn verify(env: Env, payment_id: u64) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, Payment>(&DataKey::Payment(payment_id))
            .map(|p| p.completed)
            .unwrap_or(false)
    }
}
