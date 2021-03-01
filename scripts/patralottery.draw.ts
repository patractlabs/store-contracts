import BN from 'bn.js';
import { patract, network } from 'redspot';

const { getContractAt, getRandomSigner } = patract;
const { createSigner, keyring, api } = network;

const uri =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

const lotteryContract = '5FcjY9tStk6LjTsFCUDZXFPWugrkVaonwcdppVh6xi58QD4j';

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

  await contract.tx.drawLottery(epochId-1);

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
  process.exit(0);
});
