use near_sdk::json_types::{U128, U64};
use near_sdk::near;
use near_workspaces::error::Error;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::{AccountId, Gas, NearToken};
use near_workspaces::Account;
use serde_json::json;

const FIFTY_NEAR: NearToken = NearToken::from_near(10);

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

    // This will actually be fair bit of effort to do in tests
    // We need to deploy a mock mpc contract
    // We need to derive a secp256k1 key in workspaces, find the relevant rust libraries
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
