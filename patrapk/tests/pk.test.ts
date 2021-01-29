import BN from 'bn.js';
import {expect} from 'chai';
import {patract, network, artifacts} from 'redspot';

const {getContractFactory, getRandomSigner} = patract;

const {api, getSigners} = network;

describe('PatraPK', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    const one = new BN(10).pow(new BN(api.registry.chainDecimals));
    const signers = await getSigners();
    const Alice = signers[0];
    const sender = await getRandomSigner(Alice, one.muln(100));
    const contractFactory = await getContractFactory('patrapk', sender);
    const contract = await contractFactory.deploy('new');
    const abi = artifacts.readAbi('patrapk');
    const receiver = await getRandomSigner();

    return {sender, contractFactory, contract, abi, receiver, Alice, one};
  }

  it('Create PK', async () => {
    const {contract} = await setup();

    const salt_result = await contract.query.saltHash('my salt', 'Rock');

    // @ts-ignore
    await expect(contract.tx.create(salt_result.output)).to.emit(
      contract,
      'PKCreate'
    );
  });

  it('Delete PK', async () => {
    const {contract} = await setup();

    const salt_result = await contract.query.saltHash('my salt', 'Rock');

    // @ts-ignore
    await contract.tx.create(salt_result.output);

    await expect(contract.tx.delete(1)).to.emit(
      contract,
      'PKDelete'
    );
  });

  it('Join PK', async () => {
    const {contract, Alice, one} = await setup();

    const salt_result = await contract.query.saltHash('my salt', 'Rock');

    // @ts-ignore
    await contract.tx.create(salt_result.output, {
      value: 1
    });

    const joiner = await getRandomSigner(Alice, one.muln(10));

    await expect(contract.tx.join(1, 'Paper', {
      signer: joiner,
      value: 1
    })).to.emit(
      contract,
      'PKJoin'
    );
  });

  it('Reveal PK', async () => {
    const {contract, Alice, one} = await setup();

    const salt_result = await contract.query.saltHash('my salt', 'Rock');

    // @ts-ignore
    await contract.tx.create(salt_result.output, {
      value: 1
    });
    const joiner = await getRandomSigner(Alice, one.muln(10));
    await contract.tx.join(1, 'Paper', {
      signer: joiner,
      value: 1
    });

    await expect(contract.tx.reveal(1, 'my salt', 'Rock')).to.emit(
      contract,
      'PKReveal'
    );
  });

});
