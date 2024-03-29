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
    const contractFactory = await getContractFactory('erc20_fixed', sender);
    const contract = await contractFactory.deploy('new', '1000000', 'Jupiter Token', 'JPT', '10');
    const abi = artifacts.readArtifact('erc20_fixed');
    const receiver = await getRandomSigner();

    return { sender, contractFactory, contract, abi, receiver, Alice, one };
  }

  it('Assigns initial balance', async () => {
    const { contract, sender } = await setup();
    const result = await contract.query.balanceOf(sender.address);
    expect(result.output).to.equal(1000000);
  });

  it('Transfer adds amount to destination account', async () => {
    const { contract, receiver } = await setup();

    await contract.tx.transfer(receiver.address, 7);

    const result = await contract.query.balanceOf(receiver.address);
    expect(result.output).to.equal(7);
  });

  it('Transfer emits event', async () => {
    const { contract, sender, receiver } = await setup();

    await expect(contract.tx.transfer(receiver.address, 7))
      .to.emit(contract, 'Transfer')
      .withArgs(sender.address, receiver.address, 7);
  });

  it('Can not transfer above the amount', async () => {
    const { contract, receiver } = await setup();

    await expect(contract.tx.transfer(receiver.address, 1000001)).to.not.emit(
      contract,
      'Transfer'
    );
  });

  it('Can not transfer from empty account', async () => {
    const { contract, Alice, one, sender } = await setup();

    const emptyAccount = await getRandomSigner(Alice, one.muln(10));

    await expect(
      contract.connect(emptyAccount).tx.transfer(sender.address, 7)
    ).to.not.emit(contract, "Transfer");
  });

  it('Approve token', async () => {
    const { contract, Alice, one, sender } = await setup();

    const emptyAccount = await getRandomSigner(Alice, one.muln(10));

    await contract.tx.approve(emptyAccount.address, 7);

    const result = await contract.query.allowance(sender.address, emptyAccount.address);

    expect(result.output).to.equal(7);
  });

  it('TransferFrom with approve token', async () => {
    const { contract, Alice, one, sender } = await setup();

    const emptyAccount = await getRandomSigner(Alice, one.muln(10));

    await contract.tx.approve(emptyAccount.address, 7);

    const receiver = await getRandomSigner();

    await contract.connect(emptyAccount).tx.transferFrom(sender.address, receiver.address, 7);

    const result = await contract.query.balanceOf(receiver.address);
    expect(result.output).to.equal(7);
  });
});
