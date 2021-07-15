import BN from 'bn.js';
import { patract, network } from 'redspot';

const { getContractAt, getRandomSigner } = patract;
const { createSigner, keyring, api } = network;

const uri =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

const lotteryContract = '3g4M7y7bwNDK5nUrYw3UWrs3gN8srkBnPbgdHNQPANyBoUhi';

const ticketsNum = 10;

async function run() {
  await api.isReady;

  const signer = createSigner(keyring.createFromUri(uri));
  const balance = await api.query.system.account(signer.address);
  console.log('Balance: ', balance.toHuman());

  const one = new BN(10).pow(new BN(api.registry.chainDecimals));
  const sender = await getRandomSigner(signer, one.muln(100));
  const contract = await getContractAt('patralottery', lotteryContract, sender);
  const epoch = await contract.query.latestEpoch();
  const epochId = epoch.output?.epoch_id.toHuman();
  console.log('Latest epoch: ', epochId);
  // let reqArr = [];
  for (let i=0; i<ticketsNum; i++) {
    const num1 = Math.floor(Math.random() * 10);
    const num2 = Math.floor(Math.random() * 10);
    const num3 = Math.floor(Math.random() * 10);
    const amount = Math.floor(Math.random() * 5 + 1);
    // const buyer = await getRandomSigner(signer, one.muln(amount+1));
    console.log('Buy tickets: ', num1, num2, num3, "amount: ", amount);
    // let req = contract.tx.buyTickets(epochId, [num1, num2, num3], amount, {
    //   signer: buyer,
    //   value: 10000000000 * amount
    // });
    // reqArr.push(req);
    await contract.tx.buyTickets(epochId, [num1, num2, num3], amount, {
      value: 10000000000 * amount
    })
  }
  // await Promise.all(reqArr).then((values) => {
  //   console.log(values);
  // })

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
  process.exit(0);
});
