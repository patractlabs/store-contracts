# Store Dapps

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

### ERC20 Example
```
cd erc20
```

#### Compile
```
npx redspot compile
```

#### Deploy
```
npx redspot run scripts/deploy.ts
```

#### Test
```
npx redspot test
```

