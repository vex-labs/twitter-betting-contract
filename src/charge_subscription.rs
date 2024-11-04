use hex::FromHex;
use internal_functions::hash_payload;
use omni_transaction::near::near_transaction::NearTransaction;
use omni_transaction::near::types::{
    Action, FunctionCallAction, Secp256K1Signature, Signature, U128 as OmniU128, U64 as OmniU64,
};
use omni_transaction::near::utils::PublicKeyStrExt;
use omni_transaction::transaction_builder::{TransactionBuilder, TxBuilder};
use omni_transaction::types::NEAR;

use crate::signer::*;
use crate::*;

const OMNI_GAS: OmniU64 = OmniU64(30_000_000_000_000); // 30 Tgas
const OMNI_DEPOSIT: OmniU128 = OmniU128(10_000_000_000_000_000_000_000_000); // 10 NEAR
const SIGN_CALLBACK_GAS: Gas = Gas::from_tgas(50);

#[near]
impl Contract {
    // Charge subscription to the user
    #[payable]
    pub fn charge_subscription(
        &mut self,
        account_id: AccountId,
        transaction_input: TransactionInput,
        mpc_deposit: NearToken,
    ) -> PromiseOrValue<String> {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only admin can charge subscription"
        );

        let user = self.get_user(&account_id);
        require!(
            user.next_payment_due > env::block_timestamp(),
            "User has already paid for this period"
        );

        // If the user unsubscribed for this period then remove them from the subscribers list
        if matches!(user.unsubscribe_state, Some(UnsubscribeState::Now)) {
            self.subscribers.remove(&account_id);
            return PromiseOrValue::Value("User has unsubscribed".to_string());
        }

        // Prepare function call action
        let function_call_action = Action::FunctionCall(Box::new(FunctionCallAction {
            method_name: "pay_subscription".to_string(),
            args: vec![],
            gas: OMNI_GAS,
            deposit: OMNI_DEPOSIT,
        }));

        // Add the action to the actions vector
        let actions = vec![function_call_action];

        // Build the transaction
        let near_tx = TransactionBuilder::new::<NEAR>()
            .signer_id(account_id.to_string())
            .signer_public_key(transaction_input.target_public_key.to_public_key().unwrap())
            .nonce(transaction_input.nonce.0)
            .receiver_id(env::current_account_id().to_string())
            .block_hash(transaction_input.block_hash.to_block_hash().unwrap())
            .actions(actions)
            .build();

        // Get the transaction payload and hash it
        let payload = near_tx.build_for_signing();
        let hashed_payload = hash_payload(&payload);

        // Serialize transaction into a string to pass into callback
        let serialized_tx = serde_json::to_string(&near_tx)
            .unwrap_or_else(|e| panic!("Failed to serialize NearTransaction: {:?}", e));

        // Call MPC contraCT
        PromiseOrValue::Promise(
            ext_signer::ext(self.mpc_contract.clone())
                .with_attached_deposit(mpc_deposit)
                .sign(SignRequest::new(
                    hashed_payload
                        .try_into()
                        .unwrap_or_else(|e| panic!("Failed to convert payload {:?}", e)),
                    account_id.to_string(),
                    0,
                ))
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(SIGN_CALLBACK_GAS)
                        .with_unused_gas_weight(0)
                        .sign_callback(serialized_tx),
                ),
        )
    }

    // Callback from MPC contract
    #[private]
    pub fn sign_callback(
        &self,
        #[callback_result] result: Result<SignResult, PromiseError>,
        serialized_tx: String,
    ) -> Vec<u8> {
        if let Ok(sign_result) = result {
            // Get r and s from the sign result
            let big_r = &sign_result.big_r.affine_point;
            let s = &sign_result.s.scalar;

            // Split big r into its parts
            let r = &big_r[2..];
            let end = &big_r[..2];

            // Convert hex to bytes
            let r_bytes = Vec::from_hex(r).expect("Invalid hex in r");
            let s_bytes = Vec::from_hex(s).expect("Invalid hex in s");
            let end_bytes = Vec::from_hex(end).expect("Invalid hex in end");

            // Add individual bytes together in the correct order
            let mut signature_bytes = [0u8; 65];
            signature_bytes[..32].copy_from_slice(&r_bytes);
            signature_bytes[32..64].copy_from_slice(&s_bytes);
            signature_bytes[64] = end_bytes[0];

            // Create signature
            let omni_signature = Signature::SECP256K1(Secp256K1Signature(signature_bytes));

            // Deserialize transaction
            let near_tx = serde_json::from_str::<NearTransaction>(&serialized_tx)
                .unwrap_or_else(|e| panic!("Failed to deserialize NearTransaction: {:?}", e));

            // Add signature to transaction
            let near_tx_signed = near_tx.build_with_signature(omni_signature);

            // Return signed transaction
            near_tx_signed
        } else {
            let error = result.unwrap_err();
            panic!("Callback failed with error {:?}", error);
        }
    }
}
