import BN from 'bn.js';
import { patract, network } from 'redspot';

const { getContractAt, getRandomSigner } = patract;
const { createSigner, keyring, api } = network;

const uri =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

const lotteryContract = '5H9qG8u7dFBTVWontBakVGL2yoqwfDdG2BCpwcBgViQXz9J7';

async function run() {
  await api.isReady;

  const signer = createSigner(keyring.createFromUri(uri));
  const balance = await api.query.system.account(signer.address);
  console.log('Balance: ', balance.toHuman());

  const one = new BN(10).pow(new BN(api.registry.chainDecimals));
  const sender = await getRandomSigner(signer, one.muln(12));
  const contract = await getContractAt('patralottery', lotteryContract, sender);
  const epoch = await contract.query.latestEpoch();
  const epochId = epoch.output?.epoch_id.toHuman();
  console.log('Latest epoch: ', epochId);
  const num1 = Math.floor(Math.random() * 10);
  const num2 = Math.floor(Math.random() * 10);
  const num3 = Math.floor(Math.random() * 10);
  const amount = Math.floor(Math.random() * 10 + 1);
  console.log('Buy tickets: ', num1, num2, num3, "amount: ", amount);
  await contract.tx.buyTickets(epochId, [num1, num2, num3], amount, {
    value: 10000000000 * amount
  })

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
});
