# Store Dapps

## Get started

### [Redspot](https://redspot.patract.io/zh-CN/tutorial/#get-started)

Redspot is the most advanced WASM smart contract development, testing and debugging framework. It can simplify the development workflow and connects with other tools as the core of Patract Hub.

### Run a local node
```
jupiter-dev --dev --execution=Native --tmp
```

### Compile
```
npx redspot compile
```

### Test
```
npx redspot test --no-compile
```

### Deploy ERC20
```
npx redspot run scripts/erc20.deploy.ts --no-compile
```
