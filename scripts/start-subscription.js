const { connect, keyStores } = require('near-api-js');
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

async function main() {
    if (!subscriberAccountId || !contractAccountId) {
        throw new Error("SUBSCRIBER_ACCOUNT or CONTRACT_ACCOUNT not found in .env file. Please do npm run setup first.");
    }

    const near = await connect(connectionConfig);
    const subscriberAccount = await near.account(subscriberAccountId);

    // Get the subscription public key
    const publicKey = await deriveKey(contractAccountId, subscriberAccountId);
        
    // Add the public key to the subscriber account
    await subscriberAccount.addKey(publicKey);
    console.log(`secp256k1 key added to account: ${subscriberAccountId}`);

    // Call the contract to start the subscription
    await subscriberAccount.functionCall({
      contractId: contractAccountId,
      methodName: "start_subscription",
      args: {},
      gas: "300000000000000",
      attachedDeposit: 0,
    })
}

main().catch(console.error);
