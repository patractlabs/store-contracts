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
    const contract = await contractFactory.deploy('default');
    const abi = artifacts.readArtifact('patrapixel');
    const receiver = await getRandomSigner();

    return { sender, contractFactory, contract, abi, receiver, Alice, one };
  }

  it('draw pixel', async () => {
    const { contract } = await setup();
    await expect(contract.tx.update([[1,1], [2,2], [3,3]], {
      value: 300000000000
    })).to.emit(contract, 'PixelUpdate');
  });
});
