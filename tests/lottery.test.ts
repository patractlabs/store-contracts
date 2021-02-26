import BN from 'bn.js';
import {expect} from 'chai';
import {patract, network, artifacts} from 'redspot';

const {getContractFactory, getRandomSigner} = patract;

const {api, getSigners} = network;

describe('PatraLottery', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    const one = new BN(10).pow(new BN(api.registry.chainDecimals));
    const signers = await getSigners();
    const Alice = signers[0];
    const sender = await getRandomSigner(Alice, one.muln(100));
    const contractFactory = await getContractFactory('patralottery', sender);
    const contract = await contractFactory.deploy('new');
    const abi = artifacts.readArtifact('patralottery');
    const receiver = await getRandomSigner();

    return {sender, contractFactory, contract, abi, receiver, Alice, one};
  }

  it('Buy tickets', async () => {
    const {contract} = await setup();

    await expect(contract.tx.buyTickets(100, [1, 2, 3], 2, {
      value: 20000000000
    })).to.emit(
      contract,
      'BuyTickets'
    );
  });

  it('LotteriesOf', async () => {
    const {sender, contract} = await setup();
    await contract.tx.buyTickets(100, [1, 2, 3], 2, {
      value: 20000000000
    });
    const result = await contract.query.lotteriesOf(sender.address);
    console.log(result.output);
  });

});
