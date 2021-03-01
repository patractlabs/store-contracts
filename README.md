# Store Contracts
This repository contains the [ink!](https://paritytech.github.io/ink-docs) smart contracts source code for [PatraStore](https://patrastore.io).

## Getting Started
You can using [Redspot](https://redspot.patract.io/zh-CN/) to compile and test this project.

Redspot is the most advanced WASM smart contract development, testing and debugging framework. 
It can simplify the development workflow and connects with other tools as the core of Patract Hub.

A more detailed and technical description of the redspot can be found in this repository, [here](https://redspot.patract.io/zh-CN/documentation/#documentation)

## Setup

### Installing Node.js
We require node >=12.0, if not, you can go to the nodejs website and find out how to install or upgrade.
Or we recommend that you install Node using nvm. Windows users can use nvm-windows instead.

### Substrate Prerequisites
Follow the official installation steps from the Substrate Developer Hub Knowledge Base.
```
rustup component add rust-src --toolchain nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```
### Installing The Patract Node

We use [Patract Node](https://github.com/patractlabs/patract) as our contract test chain.
It has some very convenient optimizations for contracts, such as reducing out-of-block time. To install Patract Node:

```
$ cargo install patract-prep --git https://github.com/patractlabs/patract --locked --force
```

### Run a local node
```
patract-prep --dev --execution=Native --tmp
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
