import BN from 'bn.js';
import { expect } from 'chai';
import { patract, network, artifacts } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;

const { api, getSigners } = network;

describe('ERC20', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    const one = new BN(10).pow(new BN(api.registry.chainDecimals));
    const signers = await getSigners();
    const Alice = signers[0];
    const sender = await getRandomSigner(Alice, one.muln(100));

    const daiContractFactory = await getContractFactory('erc20_issue', sender);
    const daiContract = await daiContractFactory.deployed('new', '0', 'Maker DAI', 'DAI', '18');
    const contractFactory = await getContractFactory('patramaker', sender);
    const contract = await contractFactory.deploy('new', daiContract.address);
    await daiContract.tx['transferOwnership'](contract.address.toString())
    const abi = artifacts.readArtifact('patramaker');
    const receiver = await getRandomSigner();

    return { sender, contractFactory, contract, abi, receiver, Alice, one, daiContract };
  }

  it('issue dai', async () => {
    const { contract } = await setup();
    await expect(contract.tx.issueDai(200, {
      value: 1000000000000000
    })).to.emit(contract, 'IssueDAI');
  });

  it('add collateral', async () => {
    const { contract } = await setup();
    await contract.tx.issueDai(200, {
      value: 1000000000000000
    });
    await expect(contract.tx.addCollateral(1, {
      value: 500000000000
    })).to.emit(contract, 'AddCollateral');
  });

  it('minus collateral', async () => {
    const { contract } = await setup();
    await contract.tx.issueDai(200, {
      value: 1000000000000000
    });
    await expect(contract.tx.minusCollateral(1, 500000000000))
      .to.emit(contract, 'MinusCollateral');
  });

  it('withdraw dot', async () => {
    const { contract } = await setup();
    await contract.tx.issueDai(200, {
      value: 1000000000000000
    });
    await expect(contract.tx.withdrawDot(1, 2000000000000000000))
      .to.emit(contract, 'Withdraw');
  });

  it('liquidate collateral', async () => {
    const { contract } = await setup();
    await contract.tx.issueDai(200, {
      value: 1000000000000000
    });
    await contract.tx.withdrawDot(1, 2000000000000000000);
    await expect(contract.tx.liquidateCollateral(1, 2000000000000000000))
      .to.emit(contract, ' Liquidate');
  });

});
