import BN from 'bn.js';
import { expect } from 'chai';
import { patract, network, artifacts } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;

const { api, getSigners } = network;

describe('PatraSwap', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    const one = new BN(10).pow(new BN(api.registry.chainDecimals));
    const signers = await getSigners();
    const Alice = signers[0];
    const sender = await getRandomSigner(Alice, one.muln(100));
    const contractFactory = await getContractFactory('factory', sender);
    const abi = artifacts.readArtifact('factory');
    const receiver = await getRandomSigner();

    const exchangeFactory = await getContractFactory('exchange', sender);
    const exchangeCodeHash = await exchangeFactory.putCode();

    const exchange2Factory = await getContractFactory('exchange2', sender);
    const exchange2CodeHash = await exchange2Factory.putCode();

    const lptFactory = await getContractFactory('lpt', sender);
    const lptCodeHash = await lptFactory.putCode();

    const contract = await contractFactory.deployed('factory,new');
    await contract.tx['factory,initializeFactory'](exchangeCodeHash, exchange2CodeHash, lptCodeHash);

    const erc20ContractFactory = await getContractFactory('erc20_issue', sender);
    // Tether USD, USDT, 2, 10亿
    const ethContract = await erc20ContractFactory.deployed('IErc20,new', '1000000000000000', 'Tether USD', 'USDT', '6');
    // Jupiter Bitcoin, jBTC, 8, 1百万
    const btcContract = await erc20ContractFactory.deployed('IErc20,new', '100000000000000', 'Jupiter Bitcoin', 'jBTC', '8');

    return { sender, contractFactory, contract, abi, receiver, Alice, one, ethContract, btcContract };
  }

  it('create exchange', async () => {
    const { contract, ethContract, btcContract } = await setup();

    await contract.tx['factory,createExchange'](ethContract.address, btcContract.address, undefined);
  });
});
