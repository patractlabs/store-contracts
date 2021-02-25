import { patract, network } from 'redspot';

const { getContractFactory } = patract;
const { createSigner, keyring, api } = network;

const uri =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

async function run() {
  await api.isReady;

  const signer = createSigner(keyring.createFromUri(uri));
  const contractFactory = await getContractFactory('patramaker', signer);

  const balance = await api.query.system.account(signer.address);
  console.log('Balance: ', balance.toHuman());

  const daiContractFactory = await getContractFactory('erc20_issue', signer);
  const daiContract = await daiContractFactory.deployed('IErc20,new', '0', 'Maker DAI', 'DAI', '18', {
    gasLimit: '200000000000',
    value: '0',
    salt: 'Maker DAI Token'
  });
  console.log(
    'Deploy dai successfully. The contract address: ',
    daiContract.address.toString()
  );
  console.log('');

  const contract = await contractFactory.deployed('new', daiContract.address, {
    gasLimit: '200000000000',
    value: '0',
    salt: 'PatraMaker'
  });
  console.log(
    'Deploy maker successfully. The contract address: ',
    contract.address.toString()
  );

  // transfer dai contract ownership to maker
  await daiContract.tx['ownable,transferOwnership'](contract.address.toString())

  // init dai with 100k DOT
  await contract.tx.issueDai(200, {
    value: 1000000000000000
  });

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
});
