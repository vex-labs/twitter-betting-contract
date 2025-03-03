use near_sdk::json_types::{U128, U64};
use near_sdk::{
    env, near, require, serde_json, AccountId, Gas, NearToken, PanicOnDefault, PromiseError,
    PromiseOrValue,
};

pub mod internal_functions;
pub mod proxy_bet;
pub mod signer;
pub mod view_functions;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    subscribers: Vec<AccountId>,
    admin: AccountId,
    mpc_contract: AccountId,
    betting_contract: AccountId,
    vex_token_contract: AccountId,
}

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(
        admin: AccountId,
        mpc_contract: AccountId,
        betting_contract: AccountId,
        vex_token_contract: AccountId,
    ) -> Self {
        Self {
            subscribers: Vec::new(),
            admin,
            mpc_contract,
            betting_contract,
            vex_token_contract,
        }
    }

    // Create a new subscription
    pub fn start_subscription(&mut self) {
        let account_id = env::predecessor_account_id();

        // Insert the new subscription but panic if the account is already subscribed
        if self.subscribers.contains(&account_id) {
            panic!("You are already subscribed");
        }

        self.subscribers.push(account_id);
    }

    pub fn end_subscription(&mut self) {
        let account_id = env::predecessor_account_id();

        if !self.subscribers.contains(&account_id) {
            panic!("You are not subscribed");
        }

        self.subscribers.remove(
            self.subscribers
                .iter()
                .position(|id| id == &account_id)
                .unwrap(),
        );
    }
}
