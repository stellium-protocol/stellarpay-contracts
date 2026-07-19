#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

/// Auto-incrementing counter for generating unique escrow IDs.
/// Stored as a single Symbol key; each new escrow increments this value,
/// guaranteeing no two escrows share the same ID within this contract instance.
const ESCROW_COUNTER: Symbol = symbol_short!("ESC_CNT");

/// Keys for looking up data in contract storage.
/// The `Escrow(u64)` variant maps an escrow ID to its full record.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow(u64),
}

/// Represents an escrow agreement between a buyer and seller.
///
/// The contract holds `amount` of `asset` tokens until either:
/// - The buyer releases funds to the seller, or
/// - The timeout passes and the buyer refunds themselves.
#[derive(Clone)]
#[contracttype]
pub struct Escrow {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub asset: Address,
    /// Absolute ledger timestamp at which the escrow times out.
    /// Stored as an absolute value (not relative) so we can compare directly
    /// against `env.ledger().timestamp()` without recomputing offsets.
    pub timeout: u64,
    pub released: bool,
    pub refunded: bool,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Create a new escrow. Transfers `amount` of `asset` from the buyer into
    /// this contract, where it is held until release or refund.
    ///
    /// Returns the auto-generated escrow ID.
    pub fn create(
        env: Env,
        buyer: Address,
        seller: Address,
        amount: i128,
        asset: Address,
        timeout: u64,
    ) -> u64 {
        // Auth check: ensures the transaction is signed by the buyer.
        // This prevents anyone else from creating escrows on the buyer's behalf
        // or moving the buyer's tokens without their explicit approval.
        buyer.require_auth();

        // Read and increment the counter to generate a unique escrow ID.
        // Uses instance storage — this counter lives only for the lifetime of
        // this contract instance (not across upgrades), which is fine because
        // IDs only need to be unique within a single contract deployment.
        let mut counter: u64 = env.storage().instance().get(&ESCROW_COUNTER).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&ESCROW_COUNTER, &counter);

        // Convert the relative timeout (seconds from now) into an absolute
        // ledger timestamp. Storing an absolute value lets us check
        // `current_time >= timeout` without needing to know when the escrow
        // was created.
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

        // Instance storage is used here (not persistent) because escrow records
        // are tied to this contract deployment. Instance storage is cheaper and
        // sufficient — escrows don't need to survive contract upgrades.
        env.storage().instance().set(&DataKey::Escrow(counter), &escrow);

        // Transfer tokens from buyer to this contract using the SEP-41 token
        // interface. The contract now holds the funds as a neutral custodian.
        let client = soroban_sdk::token::Client::new(&env, &asset);
        client.transfer(&buyer, &env.current_contract_address(), &amount);

        // Emit an event so off-chain indexers can track new escrows.
        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("created")),
            (counter, buyer, seller, amount),
        );

        counter
    }

    /// Release escrow funds to the seller. Only the buyer can call this,
    /// confirming they are satisfied with the transaction.
    pub fn release(env: Env, escrow_id: u64) {
        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        // Only the buyer (the party who deposited funds) can authorize release.
        escrow.buyer.require_auth();
        assert!(!escrow.released, "already released");
        assert!(!escrow.refunded, "already refunded");

        escrow.released = true;
        env.storage()
            .instance()
            .set(&DataKey::Escrow(escrow_id), &escrow);

        // Transfer the held funds from the contract to the seller.
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

    /// Refund escrow funds back to the buyer. Only callable after the timeout
    /// has passed, protecting the seller from premature refunds.
    pub fn refund(env: Env, escrow_id: u64) {
        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        escrow.buyer.require_auth();
        assert!(!escrow.released, "already released");
        assert!(!escrow.refunded, "already refunded");

        // Compare current ledger time against the stored absolute timeout.
        // The timeout was computed at creation as (creation_time + relative_timeout),
        // so this check ensures enough real time has passed before refunding.
        assert!(
            env.ledger().timestamp() >= escrow.timeout,
            "timeout not reached"
        );

        escrow.refunded = true;
        env.storage()
            .instance()
            .set(&DataKey::Escrow(escrow_id), &escrow);

        // Return the held funds to the buyer.
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

    /// Retrieve the full escrow record by its ID.
    /// Panics if the escrow does not exist.
    pub fn get_escrow(env: Env, escrow_id: u64) -> Escrow {
        env.storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found")
    }
}
