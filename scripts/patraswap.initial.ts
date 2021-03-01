
import { patract, network } from 'redspot';

const { getContractAt } = patract;
const { createSigner, keyring, api } = network;

const uri =
  'sample gloom gold judge knock acid seven dice waste amateur strike lady';

async function run() {
  await api.isReady;

  const signer = createSigner(keyring.createFromUri(uri));

  const balance = await api.query.system.account(signer.address);
  console.log('Balance: ', balance.toHuman());

  const usdt =  api.createType('AccountId', "5EuWbAoT1gRjGxCT1NQV2TtZofoCBQUWvfwUCq3yBAwwc55S");
  const dai = api.createType('AccountId', "5E2jQmVUemwwWvTVYc2H93Q6G6EwhHag5GopqUANWN2PYEnK");
  const eth = api.createType('AccountId', "5DQK1qWnBV5cRhKJEMHoopt8BBxg3xf5WZsyLvpvStjq9AZg");
  const btc = api.createType('AccountId', "5D2QZYiR656LJCfde3Bc8GJb6eRe54iepMXTf4dFuqWSpeGq");

  const swap = api.createType('AccountId', "5GHi6bBPnY5ZiaXYKeKKe8ea7x8hGx8MNbQtJ9E6pahK9eDy")
  const contract = await getContractAt('factory', swap, signer);

  await contract.query['factory,getSwapPairs']();
  // USDT -> DAI
  // await contract.tx['factory,createExchange'](usdt, dai, undefined);

  // USDT -> DOT
  // await contract.tx['factory,createExchangeWithDot'](usdt, undefined);

  // USDT -> jETH
  // await contract.tx['factory,createExchange'](usdt, eth, undefined);

  // USDT -> jBTC
  // await contract.tx['factory,createExchange'](usdt, btc, undefined);

  // jBTC -> DOT
  await contract.tx['factory,createExchangeWithDot'](btc, undefined);

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
});
