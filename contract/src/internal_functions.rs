use sha2::{Digest, Sha256};

// Function to hash payload
pub fn hash_payload(payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let result = hasher.finalize();
    result.into()
}
