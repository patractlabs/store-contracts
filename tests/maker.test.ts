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
    const daiContract = await daiContractFactory.deployed('IErc20,new', '0', 'Maker DAI', 'DAI', '18');
    const contractFactory = await getContractFactory('patramaker', sender);
    const contract = await contractFactory.deploy('new', daiContract.address);
    await daiContract.tx['ownable,transferOwnership'](contract.address.toString())
    const abi = artifacts.readArtifact('patramaker');
    const receiver = await getRandomSigner();

    return { sender, contractFactory, contract, abi, receiver, Alice, one, daiContract };
  }

  it('issue dai', async () => {
    const { contract, receiver } = await setup();

    await expect(contract.tx.issueDai(200, {
      value: 1000000000000000
    }).;


  });

});
