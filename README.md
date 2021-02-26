# Store Dapps

## Get started

### [Redspot](https://redspot.patract.io/zh-CN/tutorial/#get-started)

Redspot is the most advanced WASM smart contract development, testing and debugging framework. It can simplify the development workflow and connects with other tools as the core of Patract Hub.

### Run a local node
```
jupiter-dev --dev --execution=Native --tmp
```

### Compile
compile all contracts 
```
npx redspot compile
```
compile single contract. eg:
```
npx redspot compile "contracts/erc20-fixed"
```

### Test
test all contracts
```
npx redspot test --no-compile
```
test single contract. eg:
```
npx redspot test --no-compile ./tests/erc20.test.ts
```

### Deploy ERC20
```
npx redspot run scripts/erc20.deploy.ts --no-compile
```
