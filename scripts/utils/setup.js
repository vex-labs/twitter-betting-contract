const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');

// Specify the path for the .env file directly in the current utils directory
const envFilePath = path.join(__dirname, '.env');

// Generate a random account ID with a specified prefix
function generateRandomAccountId(prefix) {
    const randomNumbers = Math.floor(1000000000 + Math.random() * 9000000000); // Generates a 10-digit random number
    return `${prefix}-${randomNumbers}.testnet`;
}

// Generate account IDs
const adminAccountId = generateRandomAccountId('admin');
const subscriberAccountId = generateRandomAccountId('subscriber');
const contractAccountId = generateRandomAccountId('contract');

// Create the accounts and save the IDs
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

function deployContract(contractAccountId, adminAccountId) {
    return new Promise((resolve, reject) => {
        const deployCommand = `cargo near deploy --no-docker ${contractAccountId} with-init-call init json-args '{"period_length": "2592000000000000", "admin": "${adminAccountId}", "mpc_contract": "v1.signer-prod.testnet"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-legacy-keychain send`;

        // Set the current working directory to ../../contract to ensure the correct Cargo.toml is used
        exec(deployCommand, { cwd: path.join(__dirname, '../../contract'), shell: true }, (error, stdout, stderr) => {
            if (error) {
                console.error(`Deployment error details:\nstdout: ${stdout}\nstderr: ${stderr}`);
                reject(`Error deploying contract to ${contractAccountId}: ${error.message}`);
            } else {
                console.log(`Contract deployed to account: ${contractAccountId}`);
                resolve(stdout || 'Deployment completed with no output');
            }
        });
    });
}


// Run the account creation and set environment variables
Promise.all([createAccount(adminAccountId), createAccount(subscriberAccountId), createAccount(contractAccountId)])
    .then(() => {
        // Save account IDs to a .env file in the current utils directory
        const content = `SUBSCRIBER_ACCOUNT=${subscriberAccountId}\nADMIN_ACCOUNT=${adminAccountId}\nCONTRACT_ACCOUNT=${contractAccountId}\n`;
        fs.writeFileSync(envFilePath, content, 'utf8');

        // Log output to the console
        console.log(`Subscriber account: ${subscriberAccountId}`);
        console.log(`Admin account: ${adminAccountId}`);
        console.log(`Contract account: ${contractAccountId}`);

        // Deploy the contract to the `contract` account
        return deployContract(contractAccountId, adminAccountId);
    })
    .then((deployMessage) => {
        console.log(deployMessage);
    })
    .catch((error) => console.error(error));
