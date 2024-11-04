use sha2::{Digest, Sha256};

use crate::*;

#[near]
impl Contract {
    // Helper function to unsubscribe a user
    pub(crate) fn internal_unsubscribe(&mut self, account_id: AccountId) {
        let user = self.get_user_mut(account_id);
        require!(
            user.unsubscribe_state.is_some(),
            "User has already unsubscribed"
        );

        // If the user unsubscribes before paying for the current period
        // then we set the unsubscribe state to NextPeriod
        // otherwise we set it to Now
        if user.next_payment_due >= env::block_timestamp() {
            user.unsubscribe_state = Some(UnsubscribeState::NextPeriod);
        } else {
            user.unsubscribe_state = Some(UnsubscribeState::Now);
        }
    }

    // Helper function to get the relevent user as a reference
    pub(crate) fn get_user(&self, user: &AccountId) -> &SubscriptionInfo {
        self.subscribers
            .get(user)
            .unwrap_or_else(|| panic!("User is not subscribed"))
    }

    // Helper function to get the relevent user as a mutable reference
    pub(crate) fn get_user_mut(&mut self, user: AccountId) -> &mut SubscriptionInfo {
        self.subscribers
            .get_mut(&user)
            .unwrap_or_else(|| panic!("User is not subscribed"))
    }
}

// Function to hash payload
pub fn hash_payload(payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let result = hasher.finalize();
    result.into()
}
