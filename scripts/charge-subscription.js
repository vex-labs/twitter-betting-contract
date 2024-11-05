const { connect, keyStores, providers } = require('near-api-js');
const homedir = require("os").homedir();
const path = require("path");
const { deriveKey } = require('./utils/derive-mpc-key.js');
require('dotenv').config({ path: path.join(__dirname, './utils/.env') }); 

const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
const myKeyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

const connectionConfig = {
  networkId: "testnet",
  keyStore: myKeyStore,
  nodeUrl: "https://rpc.testnet.near.org",
  walletUrl: "https://testnet.mynearwallet.com/",
  helperUrl: "https://helper.testnet.near.org",
  explorerUrl: "https://testnet.nearblocks.io",
};

// Retrieve account IDs from .env
const subscriberAccountId = process.env.SUBSCRIBER_ACCOUNT;
const contractAccountId = process.env.CONTRACT_ACCOUNT;
const adminAccountId = process.env.ADMIN_ACCOUNT;

async function main() {
    if (!subscriberAccountId || !contractAccountId || !adminAccountId) {
        throw new Error("SUBSCRIBER_ACCOUNT or CONTRACT_ACCOUNT not found in .env file. Please do npm run setup first.");
    }

    const near = await connect(connectionConfig);
    const adminAccount = await near.account(adminAccountId);

    // Get the public on the subscriber account that the MPC has control over
    const publicKey = await deriveKey(contractAccountId, subscriberAccountId);

    // Get the nonce of the key
    const accessKey = await near.connection.provider.query({
        request_type: 'view_access_key',
        account_id: subscriberAccountId,
        public_key: publicKey,
        finality: 'optimistic'
        });
    const nonce = accessKey.nonce;
    
    // Get recent block hash
    const block = await near.connection.provider.block({
        finality: "final",
    });
    const blockHash = block.header.hash;

    // Prepare transaction input
    const transaction_input = {
        target_public_key: publicKey,
        nonce: (nonce + 1).toString(),
        block_hash: blockHash,
    };

    // Call the contract to charge the subscription
    const outcome = await adminAccount.functionCall({
        contractId: contractAccountId,
        methodName: "charge_subscription",
        args: {
            account_id: subscriberAccountId,
            transaction_input,
        },
        gas: "300000000000000",
        attachedDeposit: "500000000000000000000000", // 0.5 NEAR is morrreee than enough
    })

    // Get the signed transaction from the outcome
    result = providers.getTransactionLastResult(outcome);
    const signedSerializedTx = new Uint8Array(result);

    // Send the signed transaction
    const send_result = await near.connection.provider.sendJsonRpc("broadcast_tx_commit", [
        Buffer.from(signedSerializedTx).toString("base64"),
    ]);
    
    console.log(send_result);
}

main().catch(console.error);