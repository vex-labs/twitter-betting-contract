use crate::*;

#[near(serializers = [json])]
pub struct SubscriptionView {
    pub account_id: AccountId,
    pub next_payment_due: U64,
}

#[near]
impl Contract {
    // View the subscription information for a user
    pub fn view_user(&self, account_id: AccountId) -> SubscriptionView {
        let next_payment_due = self.get_next_payment(&account_id);
        self.format_subscription(account_id, next_payment_due)
    }

    // View a number of user's subscription information
    pub fn view_users(
        &self,
        from_index: &Option<u32>,
        limit: &Option<u32>,
    ) -> Vec<SubscriptionView> {
        // If no index or limit is provided then return all users
        let from = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(self.subscribers.len());

        self.subscribers
            .iter()
            .skip(from as usize)
            .take(limit as usize)
            .map(|(account_id, subscription_info)| {
                self.format_subscription(account_id.clone(), subscription_info.clone())
            })
            .collect()
    }

    // Format the subscription information with the account_id
    fn format_subscription(
        &self,
        account_id: AccountId,
        next_payment_due: NextPaymentDue,
    ) -> SubscriptionView {
        SubscriptionView {
            account_id,
            next_payment_due: U64(next_payment_due),
        }
    }
}
