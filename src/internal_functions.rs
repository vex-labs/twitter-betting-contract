use sha2::{Digest, Sha256};

use crate::*;

#[near]
impl Contract {
    // Helper function to get the relevent user as a reference
    pub(crate) fn get_next_payment(&self, user: &AccountId) -> NextPaymentDue {
        self.subscribers
            .get(user)
            .unwrap_or_else(|| panic!("User is not subscribed"))
            .clone()
    }
}

// Function to hash payload
pub fn hash_payload(payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let result = hasher.finalize();
    result.into()
}
