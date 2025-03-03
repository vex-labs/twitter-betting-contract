use hex::FromHex;
use internal_functions::hash_payload;
use omni_transaction::near::types::{
    Action, FunctionCallAction, Secp256K1Signature, Signature, U128 as OmniU128, U64 as OmniU64,
};
use omni_transaction::near::utils::PublicKeyStrExt;
use omni_transaction::near::NearTransaction;
use omni_transaction::NEAR;
use omni_transaction::{TransactionBuilder, TxBuilder};

use crate::signer::*;
use crate::*;

const OMNI_GAS: OmniU64 = OmniU64(100_000_000_000_000); // 100 Tgas
const OMNI_DEPOSIT: OmniU128 = OmniU128(1); // 1 yocto NEAR
const SIGN_CALLBACK_GAS: Gas = Gas::from_tgas(50);

#[near(serializers = [json])]
pub struct TransactionInput {
    subscriber_public_key: String,
    nonce: U64,
    block_hash: String,
}

#[near(serializers = [json])]
pub struct BetInput {
    match_id: String,
    team: Team,
    amount: U128,
}

#[near(serializers = [json])]
pub enum Team {
    Team1,
    Team2,
}

#[near]
impl Contract {
    #[payable]
    pub fn proxy_bet(
        &mut self,
        account_id: AccountId,
        transaction_input: TransactionInput,
        bet_input: BetInput,
    ) -> PromiseOrValue<String> {
        require!(
            env::predecessor_account_id() == self.admin,
            "Only admin can charge subscription"
        );

        // Prepare function call action
        let function_call_action = Action::FunctionCall(Box::new(FunctionCallAction {
            method_name: "ft_transfer_call".to_string(),
            args: serde_json::to_vec(&serde_json::json!({
                "receiver_id": self.betting_contract,
                "amount": bet_input.amount,
                "msg": serde_json::json!({
                    "Bet": {
                        "match_id": bet_input.match_id,
                        "team": bet_input.team,
                    }
                }).to_string()
            }))
            .unwrap_or_else(|e| panic!("Failed to serialize ft_transfer_call args: {:?}", e)),
            gas: OMNI_GAS,         // TODO should be normal U64
            deposit: OMNI_DEPOSIT, // TODO should be NearToken
        }));

        // Add the action to the actions vector
        let actions = vec![function_call_action];

        // Build the transaction
        let near_tx = TransactionBuilder::new::<NEAR>()
            .signer_id(account_id.to_string())
            .signer_public_key(
                transaction_input
                    .subscriber_public_key
                    .to_public_key()
                    .unwrap(),
            )
            .nonce(transaction_input.nonce.0)
            .receiver_id(self.vex_token_contract.to_string())
            .block_hash(transaction_input.block_hash.to_block_hash().unwrap())
            .actions(actions)
            .build();

        // Serialize transaction into a string to pass into callback
        let tx_json_string = serde_json::to_string(&near_tx)
            .unwrap_or_else(|e| panic!("Failed to serialize NearTransaction: {:?}", e))
            // Convert numeric values to strings to fix serialization issues
            .replace(
                &format!("\"nonce\":{}", transaction_input.nonce.0),
                &format!("\"nonce\":\"{}\"", transaction_input.nonce.0),
            )
            .replace(
                &format!("\"amount\":{}", bet_input.amount.0),
                &format!("\"amount\":\"{}\"", bet_input.amount.0),
            );

        // Create the payload, hash it and convert to a 32-byte array
        let payload = near_tx.build_for_signing();
        let hashed_payload = hash_payload(&payload);
        let mpc_payload: [u8; 32] = hashed_payload
            .try_into()
            .unwrap_or_else(|e| panic!("Failed to convert payload {:?}", e));

        let key_version = 0;
        let path = account_id.to_string();

        // Call MPC contract
        PromiseOrValue::Promise(
            ext_signer::ext(self.mpc_contract.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .sign(SignRequest::new(mpc_payload, path, key_version))
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(SIGN_CALLBACK_GAS)
                        .with_unused_gas_weight(0)
                        .sign_callback(tx_json_string),
                ),
        )
    }

    // Callback from MPC contract
    #[private]
    pub fn sign_callback(
        &self,
        #[callback_result] result: Result<SignResult, PromiseError>,
        tx_json_string: String,
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
            let near_tx = serde_json::from_str::<NearTransaction>(&tx_json_string)
                .unwrap_or_else(|e| panic!("Failed to deserialize transaction: {:?}", e));

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
