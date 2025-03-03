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
  nodeUrl: "https://test.rpc.fastnear.com",
};

// Retrieve account IDs from .env
const subscriberAccountId = process.env.SUBSCRIBER_ACCOUNT;
const contractAccountId = process.env.CONTRACT_ACCOUNT;
const adminAccountId = process.env.ADMIN_ACCOUNT;

// Constants for this specific bet
const MATCH_ID = "SSG-ENCE-23/03/2025";
const TEAM = "Team1"; // SSG is Team1
const AMOUNT = "1000000"; // 1 USDC (6 decimals)

async function main() {
    if (!subscriberAccountId || !contractAccountId || !adminAccountId) {
        throw new Error("Account IDs not found in .env file. Please do npm run setup first.");
    }

    const near = await connect(connectionConfig);
    const adminAccount = await near.account(adminAccountId);

    // Get the subscription public key
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
        subscriber_public_key: publicKey,
        nonce: (nonce + 1).toString(),
        block_hash: blockHash,
    };

    // Prepare bet input
    const bet_input = {
        match_id: MATCH_ID,
        team: TEAM,
        amount: AMOUNT,
    };

    // Call the contract to proxy the bet
    const outcome = await adminAccount.functionCall({
        contractId: contractAccountId,
        methodName: "proxy_bet",
        args: {
            account_id: subscriberAccountId,
            transaction_input,
            bet_input,
        },
        gas: "300000000000000",
        attachedDeposit: "100000000000000000000000", // 0.1 NEAR for MPC fees
    });

    // Get the signed transaction from the outcome
    const result = providers.getTransactionLastResult(outcome);
    const signedTx = new Uint8Array(result);

    // Send the signed transaction
    const send_result = await near.connection.provider.sendJsonRpc("broadcast_tx_commit", [
        Buffer.from(signedTx).toString("base64"),
    ]);
    
    console.log("Bet transaction proxied successfully!");
    console.log(send_result);
}

main()
    .then(() => process.exit(0))
    .catch(error => {
        console.error(error);
        process.exit(1);
    });