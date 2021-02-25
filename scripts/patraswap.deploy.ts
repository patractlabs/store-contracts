
import { patract, network } from 'redspot';

const { getContractFactory } = patract;
const { createSigner, keyring, api } = network;

const uri =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

async function run() {
  await api.isReady;

  const signer = createSigner(keyring.createFromUri(uri));
  const contractFactory = await getContractFactory('factory', signer);

  const balance = await api.query.system.account(signer.address);

  console.log('Balance: ', balance.toHuman());

  const exchangeFactory = await getContractFactory('exchange', signer);
  const exchangeCodeHash = await exchangeFactory.putCode();
  console.log(
    'Put exchange code successfully. The contract code hash: ',
    exchangeCodeHash.toString()
  );
  console.log('');

  const exchange2Factory = await getContractFactory('exchange2', signer);
  const exchange2CodeHash = await exchange2Factory.putCode();
  console.log(
    'Put exchange2 code successfully. The contract code hash: ',
    exchange2CodeHash.toString()
  );
  console.log('');

  const lptFactory = await getContractFactory('lpt', signer);
  const lptCodeHash = await lptFactory.putCode();
  console.log(
    'Put lpt code successfully. The contract code hash: ',
    lptCodeHash.toString()
  );
  console.log('');

  const contract = await contractFactory.deployed('factory,new', {
    gasLimit: '200000000000',
    value: '0',
    salt: 'PatraSwap'
  });
  console.log(
    'Deploy factory successfully. The contract address: ',
    contract.address.toString()
  );
  console.log('');

  await contract.tx['factory,initializeFactory'](exchangeCodeHash, exchange2CodeHash, lptCodeHash);
  console.log('initializeFactory successfully.');
  console.log('');

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
});
