// TODO

use near_sdk::json_types::{U128, U64};
use near_sdk::near;
use near_workspaces::error::Error;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::{AccountId, Gas, NearToken, PublicKey};
use near_workspaces::Account;
use serde_json::json;

const FIFTY_NEAR: NearToken = NearToken::from_near(10);
const ROOT_PUBLIC_KEY: &str = "secp256k1:4NfTiv3UsGahebgTaHyD9vF8KYKMBnfd6kh94mK6xv8fGBiJB8TBtFMP5WWXz6B89Ac1fbpzPwAvoyQebemHFwx3";

#[tokio::test]
async fn test_contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;

    let root = sandbox.root_account()?;

    // Create accounts
    let alice = create_account(&root, "alice").await?;
    let admin = create_account(&root, "admin").await?;
    let contract_account = create_account(&root, "contract").await?;
    let mpc_contract_account = create_account(&root, "mpc_contract").await?;

    // Deploy MPC contract

    // Deploy and initialize contract
    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let period_length = U64(60_000_000_000); // One minute in nanoseconds

    let mut result = contract
        .call("init")
        .args_json(json!({"period_length": period_length, "admin": admin.id(), "mpc_contract": mpc_contract_account.id()}))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Contract initialization failed: {:?}",
        dbg!(result)
    );

    // Derive secp256k1 key for alice's subscription
    let epsilon = derive_epsilon(&alice.id(), "subscription");
    println!("Epsilon: {:?}", epsilon);

    // Start subscription
    result = alice
        .call(contract_account.id(), "start_subscription")
        .gas(Gas::from_tgas(50))
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Subscription failed: {:?}",
        dbg!(result)
    );

    // Try to start subscription for alice again
    result = alice
        .call(contract_account.id(), "start_subscription")
        .gas(Gas::from_tgas(50))
        .transact()
        .await?;

    assert!(
        result.is_failure(),
        "Subscription should have failed: {:?}",
        dbg!(result)
    );

    // Need to add the key
    // Need to send the transaction to the sandbox environment

    Ok(())
}

async fn create_account(root: &near_workspaces::Account, name: &str) -> Result<Account, Error> {
    let subaccount = root
        .create_subaccount(name)
        .initial_balance(FIFTY_NEAR)
        .transact()
        .await?
        .unwrap();

    Ok(subaccount)
}

fn derive_public_key(path: String, predecessor: AccountId) -> PublicKey {
    let epsilon = derive_epsilon(&predecessor, &path);
}

// pub fn derive_epsilon(predecessor_id: &AccountId, path: &str) -> Scalar {
//     const EPSILON_DERIVATION_PREFIX: &str = "epsilon:";

//     let derivation_path = format!("{EPSILON_DERIVATION_PREFIX}{},{}", predecessor_id, path);
//     let mut hasher = Sha3_256::new();
//     hasher.update(derivation_path);
//     let hash: [u8; 32] = hasher.finalize().into();
//     Scalar::from_non_biased(hash)
// }
