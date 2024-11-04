use near_sdk::json_types::U64;
use near_sdk::store::IterableMap;
use near_sdk::{
    env, near, require, serde_json, AccountId, Gas, NearToken, PanicOnDefault, PromiseError,
    PromiseOrValue,
};

pub mod charge_subscription;
pub mod internal_functions;
pub mod signer;
pub mod view_functions;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    subscribers: IterableMap<AccountId, SubscriptionInfo>,
    period_length: u64,
    admin: AccountId,
    mpc_contract: AccountId,
}

#[near(serializers = [borsh])]
pub struct SubscriptionInfo {
    next_payment_due: u64,
    unsubscribe_state: Option<UnsubscribeState>,
}

impl SubscriptionInfo {
    pub fn new(period_length: u64) -> Self {
        Self {
            next_payment_due: env::block_timestamp() + period_length,
            unsubscribe_state: None,
        }
    }
}

#[near(serializers = [json])]
pub struct TransactionInput {
    target_public_key: String,
    nonce: U64,
    block_hash: String,
}

#[derive(Clone)]
#[near(serializers = [json, borsh])]
pub enum UnsubscribeState {
    Now,
    NextPeriod,
}

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
            .insert(account_id, SubscriptionInfo::new(self.period_length))
            .is_some()
        {
            panic!("You are already subscribed");
        }
    }

    // Function to pay the subscription
    // Transaction sent to call this should be signed by the MPC
    #[payable]
    pub fn pay_subscription(&mut self) {
        require!(
            env::attached_deposit() == NearToken::from_near(10),
            "Attached deposit must be 10"
        );

        // Get period length early to avoid borrowing issues
        let period_length = self.period_length;

        let user = self.get_user_mut(env::predecessor_account_id());
        require!(
            user.next_payment_due <= env::block_timestamp(),
            "Payment is not due yet"
        );
        user.next_payment_due = user.next_payment_due + period_length;

        // If the user wanted to unsubscribe before paying for that period
        // then we set the unsubscribe state to Now
        if matches!(user.unsubscribe_state, Some(UnsubscribeState::NextPeriod)) {
            user.unsubscribe_state = Some(UnsubscribeState::Now);
        }
    }

    // Function to let a user unsubscribe
    pub fn user_unsubscribe(&mut self) {
        self.internal_unsubscribe(env::predecessor_account_id());
    }

    // Function for the admin to cancel a user's subscription
    pub fn cancel_user_subscription(&mut self, user: AccountId) {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only admin can cancel subscription"
        );
        let unsub_user = self.get_user(&user);

        // If the user can be unsubscribed immediately then remove them from the map
        // otherwise proceed with the internal unsubscribe
        if matches!(unsub_user.unsubscribe_state, Some(UnsubscribeState::Now)) {
            self.subscribers.remove(&user);
        } else {
            self.internal_unsubscribe(user);
        }
    }
}
