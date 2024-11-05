use near_sdk::json_types::U64;
use near_sdk::store::IterableMap;
use near_sdk::{
    env, near, require, serde_json, AccountId, Gas, NearToken, PanicOnDefault, PromiseError,
    PromiseOrValue
};

pub mod charge_subscription;
pub mod internal_functions;
pub mod signer;
pub mod view_functions;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    subscribers: IterableMap<AccountId, NextPaymentDue>,
    period_length: u64,
    admin: AccountId,
    mpc_contract: AccountId,
}

type NextPaymentDue = u64; // Type alias

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(period_length: U64, admin: AccountId, mpc_contract: AccountId) -> Self {
        Self {
            subscribers: IterableMap::new(b"s"),
            period_length: period_length.into(),
            admin,
            mpc_contract,
        }
    }

    // Create a new subscription
    pub fn start_subscription(&mut self) {
        let account_id = env::predecessor_account_id();

        // Insert the new subscription but panic if the account is already subscribed
        if self
            .subscribers
            .insert(account_id, env::block_timestamp())
            .is_some()
        {
            panic!("You are already subscribed");
        }
    }

    pub fn end_subscription(&mut self) {
        let account_id = env::predecessor_account_id();

        if self.subscribers.remove(&account_id).is_none() {
            panic!("You are not subscribed");
        }
    }

    // Function to pay the subscription
    // the transaction sent to call this should be signed by the MPC
    #[payable]
    pub fn pay_subscription(&mut self) {
        require!(
            env::attached_deposit() == NearToken::from_near(5),
            "Attached deposit must be 10"
        );

        let account_id = env::predecessor_account_id();

        let mut next_payment_due = self.get_next_payment(&account_id);
        require!(
            next_payment_due <= env::block_timestamp(),
            "Payment is not due yet"
        );

        // Update the next payment due date
        next_payment_due += self.period_length;
        self.subscribers.insert(account_id, next_payment_due);
    }
}
