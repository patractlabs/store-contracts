# Patra Swap

## Deploy

1. Deploy `erc20` contract, issue token

2. Upload `exchange` contract wasm, get contract hash

3. Deploy factory contract

4. Call `factory: initializeFactory` function, params is `exchange` contract hash

5. Call `factory: createExchange` function, params is `erc20` contract account

6. Call `erc20: approve` function to `exchange` contract account



