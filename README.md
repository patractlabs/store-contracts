# Store Dapps

## Develop Testnet Dapps Address
wss://ws.staging.jupiter.patract.cn
### ERC20 Code Hash
1. fixed
```
0xf78f775b3c9d7f865129d86e418a725734446f3a289df8e69f0c487f02fbeccf
```
2. issue
```
0x2735794dd9b9394e7ea716272ece04074fac54a78106e87bc62f9436f67ce4a5
```
### PatraPK
```
5DkPJueCHTqyVjRUsZnD1HzvtrscESZc3utXR7PCoz4MSNQZ
```
### PatraPixel
```
5CSuyAbVkhJ84WzDbJohNMs4eQizoYoqds6yQiwv3iDim2hD
```
### PatraLottery
```
5DYvhVPkjTJdbQNz66kza3e2Cn7XvxBpR4F1s1si9Vd96wFh
```
### PatraSwap
Factory
```
5C8pqeB3Cj1ePZm49D1wFwHAJtZARviCGhXkpGtSJSmdgHm7
```

### 固定地址的预发行资产列表：
1. Tether USD, USDT, 2, 10亿, 可增发,
   地址：`5EWH8uG8DYgcurXhJ2GANxeCyRDHgVAK5uupqsQSekPSRM9g`
1. Jupiter Bitcoin, jBTC, 8, 1百万，可增发,
   地址：`5GtCRvPhaSo9iUJHcE7TyDsHveh3duEzeike9E8PLoxfUhzL`
1. Jupiter Ethereum, jETH, 18, 1千万，可增发,
   地址：`5DLB2GYTjdQHJigkDx1SWGC4a12VwGAWtuJrnh82Y1BdrKBr`
1. Maker DAI, DAI, 18, x, 可增发,
   地址：`5GnhhemyFoDQjo53rFZbm6D48rv4EdfjBS3fqGCpQvYWD3jP`

## DeFi

### Tether USD
中心化权限发行稳定币

[ethereum contract address](https://etherscan.io/address/0xdac17f958d2ee523a2206206994597c13d831ec7)

### Patra Maker

https://github.com/makerdao

### OneClickAsset
Issuing ERC20, Issuing ERC721

https://yjfb.net/

### Uniswap
精简版（治理模块忽略）

## Game

### CryptoPixel
https://pixelchain.art/editor

一块1024*1024的画布, 单像素价格从1DOT起步，统计总画过的像素数，画第几遍的时候，单价就是2的几次方。资产汇集到国库。

### Dice
(当前区块的随机数)，选择1到6，打包时计算对了就拿走奖池一半的钱，错了就留到奖池。

### Lottery
(BABE随机数)，选择三位数的数字，如果当前babe随机数的后几位符合，就拿走奖池一半奖金，错了就留到奖池。

# Get started

## Development Tools
Details visit: [Redspot](https://redspot.patract.io/zh-CN/tutorial/#get-started)

### Run a local node
```
jupiter-dev --dev --execution=Native --tmp
```

### ERC20 Example
```
cd erc20
```

#### Compile
```
npx redspot compile
```

#### Test
```
npx redspot test --no-compile
```

#### Deploy
```
npx redspot run scripts/deploy.ts
```
