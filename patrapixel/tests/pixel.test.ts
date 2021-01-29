import BN from 'bn.js';
import { expect } from 'chai';
import { patract, network, artifacts } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;

const { api, getSigners } = network;

describe('PatraPixel', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    const one = new BN(10).pow(new BN(api.registry.chainDecimals));
    const signers = await getSigners();
    const Alice = signers[0];
    const sender = await getRandomSigner(Alice, one.muln(100));
    const contractFactory = await getContractFactory('patrapixel', sender);
    const contract = await contractFactory.deploy('new');
    const abi = artifacts.readAbi('patrapixel');
    const receiver = await getRandomSigner();

    return { sender, contractFactory, contract, abi, receiver, Alice, one };
  }

  it('Mint new pixel with metadata', async () => {
    const { contract } = await setup();

    await expect(contract.tx.mintWithMetadata('my pixel')).to.emit(
      contract,
      'Minted'
    );
  });

  it('Balance increase by mint new pixel', async () => {
    const { contract, sender } = await setup();

    await contract.tx.mintWithMetadata('my pixel');

    const result = await contract.query.balanceOf(sender.address);

    await expect(result.output).to.equal([1]);
  });

  it('Total supply increase by mint new pixel', async () => {
    const { contract, sender } = await setup();

    await contract.tx.mintWithMetadata('my pixel 1');
    await contract.tx.mintWithMetadata('my pixel 2');

    const result = await contract.query.totalSupply();

    await expect(result.output).to.equal(2);
  });
});
