# subscription-example

This is a simple example showing how to use chain signatues to allow your account to be controlled by a smart contract. This example is a simple subscription service where a user can subscribe to an arbitary service and allows the admin to charge 5 NEAR from the user every month. 

This example has some scripts to show how to interact with each part of the contract.

## Running the project

You must have the [NEAR CLI](https://github.com/near/near-cli-rs/releases) installed to run this project.

Enter the scripts directory and install the dependencies:
```bash
cd scripts
npm install
```

To interact with the contract you will need three different accounts. A subscriber, an admin and a contract. Run the following command to create the accounts and deploy the contract:
```bash
npm run setup
```

To subscribe to the service run the following command:
```bash
npm run subscribe
```

To charge the subscriber from the admin account run the following command:
```bash
npm run charge
```

To unsubscribe from the service run the following command:
```bash
npm run unsubscribe
```