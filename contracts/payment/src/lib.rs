#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol};

/// Auto-incrementing counter for generating unique payment IDs.
/// Each new payment increments this value, guaranteeing uniqueness within
/// this contract instance.
const PAYMENT_COUNTER: Symbol = symbol_short!("PAY_CNT");

/// Keys for looking up data in contract storage.
/// The `Payment(u64)` variant maps a payment ID to its full record.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Payment(u64),
}

/// Represents a completed direct payment from sender to recipient.
///
/// Unlike escrow, payments execute immediately — tokens transfer in the same
/// transaction. There is no holding period or release step.
#[derive(Clone)]
#[contracttype]
pub struct Payment {
    pub id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub amount: i128,
    pub asset: Address,
    /// Arbitrary data attached to the payment. Can store invoice IDs,
    /// order numbers, memos, or any application-specific reference.
    /// Using `Bytes` (rather than `String`) keeps it encoding-agnostic
    /// so callers can embed whatever format they need.
    pub metadata: Bytes,
    /// Always `true` for records stored by this contract. A payment only
    /// gets stored after the token transfer succeeds, so every record
    /// represents a real, completed transfer. This flag exists so
    /// `verify()` can distinguish a completed payment from a missing one
    /// (which returns `false`).
    pub completed: bool,
}

#[contract]
pub struct PaymentContract;

#[contractimpl]
impl PaymentContract {
    /// Execute a direct payment: transfers `amount` of `asset` from `sender`
    /// to `recipient` immediately, then stores a record of the transaction.
    ///
    /// Returns the auto-generated payment ID.
    pub fn pay(
        env: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        asset: Address,
        metadata: Bytes,
    ) -> u64 {
        // Auth check: ensures the transaction is signed by the sender.
        // Prevents unauthorized transfers from the sender's account.
        sender.require_auth();

        // Generate a unique payment ID via auto-incrementing counter.
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
            // Marked completed immediately because the transfer happens
            // synchronously below — there is no waiting period or approval step.
            completed: true,
        };

        // Store the payment record in instance storage (cheaper than persistent,
        // and payment history doesn't need to survive contract upgrades).
        env.storage()
            .instance()
            .set(&DataKey::Payment(counter), &payment);

        // Transfer tokens directly from sender to recipient.
        // This is a standard SEP-41 token transfer — the contract acts as
        // the caller, not a custodian (unlike escrow).
        let client = soroban_sdk::token::Client::new(&env, &asset);
        client.transfer(&sender, &recipient, &amount);

        // Emit an event for off-chain indexing.
        env.events().publish(
            (symbol_short!("payment"), symbol_short!("created")),
            (counter, sender, recipient, amount),
        );

        counter
    }

    /// Retrieve the full payment record by its ID.
    /// Panics if the payment does not exist.
    pub fn get_payment(env: Env, payment_id: u64) -> Payment {
        env.storage()
            .instance()
            .get(&DataKey::Payment(payment_id))
            .expect("payment not found")
    }

    /// Check if a payment exists and is completed.
    ///
    /// Returns `true` if the payment ID exists and `completed` is `true`,
    /// `false` otherwise (including for nonexistent IDs). This lets callers
    /// verify a payment succeeded without panicking on missing entries.
    pub fn verify(env: Env, payment_id: u64) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, Payment>(&DataKey::Payment(payment_id))
            .map(|p| p.completed)
            .unwrap_or(false)
    }
}
