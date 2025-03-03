use crate::*;

#[near(serializers = [json])]
pub struct SubscriptionView {
    pub account_id: AccountId,
    pub next_payment_due: U64,
}

#[near]
impl Contract {
    pub fn get_subscribers(&self) -> Vec<AccountId> {
        self.subscribers.clone()
    }

    pub fn is_subscribed(&self, account_id: AccountId) -> bool {
        self.subscribers.contains(&account_id)
    }
}
