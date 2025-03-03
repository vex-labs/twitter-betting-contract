const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');

// Specify the path for the .env file directly in the current utils directory
const envFilePath = path.join(__dirname, '.env');

// Generate account IDs
const adminAccountId = generateRandomAccountId('admin');
const subscriberAccountId = generateRandomAccountId('subscriber');
const contractAccountId = generateRandomAccountId('contract');

// Fixed testnet contract addresses
const mpcContractId = "v1.signer-prod.testnet";
const bettingContractId = "betting.betvex.testnet";
const vexTokenContractId = "token.betvex.testnet";

// Run the functions
Promise.all([createAccount(adminAccountId), createAccount(subscriberAccountId), createAccount(contractAccountId)])
    .then(() => {
        // Save account IDs and contract addresses to a .env file in the current utils directory
        const content = `SUBSCRIBER_ACCOUNT=${subscriberAccountId}
ADMIN_ACCOUNT=${adminAccountId}
CONTRACT_ACCOUNT=${contractAccountId}
MPC_CONTRACT=${mpcContractId}
BETTING_CONTRACT=${bettingContractId}
VEX_TOKEN_CONTRACT=${vexTokenContractId}`;

        fs.writeFileSync(envFilePath, content, 'utf8');

        // Log output to the console
        console.log('Created accounts:');
        console.log(`  Subscriber account: ${subscriberAccountId}`);
        console.log(`  Admin account: ${adminAccountId}`);
        console.log(`  Contract account: ${contractAccountId}`);
        console.log('\nContract addresses:');
        console.log(`  MPC contract: ${mpcContractId}`);
        console.log(`  Betting contract: ${bettingContractId}`);
        console.log(`  VEX token contract: ${vexTokenContractId}`);

        // Deploy the contract to the `contract` account
        console.log('\nDeploying contract...');
        return deployContract(contractAccountId, adminAccountId, mpcContractId, bettingContractId, vexTokenContractId);
    })
    .then((deployMessage) => {
        console.log(deployMessage);
        console.log('\nSetup completed successfully! You can now:');
        console.log('1. Run "npm run start" to start a subscription');
        console.log('2. Run "npm run bet" to place a bet');
        console.log('3. Run "npm run end" to end a subscription');
    })
    .catch((error) => console.error(error));

// Generate a random account ID with a specified prefix
function generateRandomAccountId(prefix) {
    const randomNumbers = Math.floor(1000000000 + Math.random() * 9000000000); // Generates a 10-digit random number
    return `${prefix}-${randomNumbers}.testnet`;
}

// Create the accounts
function createAccount(accountId) {
    return new Promise((resolve, reject) => {
        const command = `near account create-account sponsor-by-faucet-service ${accountId} autogenerate-new-keypair save-to-legacy-keychain network-config testnet create`;

        exec(command, (error, stdout, stderr) => {
            if (error) {
                console.error(`Error details: ${stderr}`);
                reject(`Error creating account ${accountId}: ${error.message}`);
            } else {
                resolve(accountId);
            }
        });
    });
}

// Deploy contract
function deployContract(contractAccountId, adminAccountId, mpcContractId, bettingContractId, vexTokenContractId) {
    return new Promise((resolve, reject) => {
        const deployCommand = `cargo near deploy build-non-reproducible-wasm ${contractAccountId} with-init-call init json-args '{"period_length": "2592000000000000", "admin": "${adminAccountId}", "mpc_contract": "${mpcContractId}", "betting_contract": "${bettingContractId}", "vex_token_contract": "${vexTokenContractId}"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-legacy-keychain send`;

        // Set the current working directory to ../../contract to ensure the correct Cargo.toml is used
        exec(deployCommand, { cwd: path.join(__dirname, '../../contract'), shell: true }, (error, stdout, stderr) => {
            if (error) {
                console.error(`Deployment error details:\nstdout: ${stdout}\nstderr: ${stderr}`);
                reject(`Error deploying contract to ${contractAccountId}: ${error.message}`);
            } else {
                resolve('Contract deployed successfully!');
            }
        });
    });
}
