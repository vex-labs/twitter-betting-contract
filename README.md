# twitter-betting-contract 

> [!Caution]
> This is definitely not production-ready; significant modifications need to be made, you should get an audit and consult some legal people before launching this or something similar.

For this example remember to send the new account some usdc.betvex.testnet after setup.

For this stage this is centralized, only the admin can call the function to proxy bet. In the future we will make this verifiable by the user needing to provide a proof or running in a verifiable agent or something.

TODO: Implement bet limits. This will require cross contract call from the main vex contract probably. We need to make sure the transaction lands.

