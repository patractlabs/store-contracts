import { patract, network } from 'redspot';

const { getContractFactory } = patract;
const { createSigner, keyring, api } = network;

const uri =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

async function run() {
  await api.isReady;

  const signer = createSigner(keyring.createFromUri(uri));
  const contractFactory = await getContractFactory('erc20-issue', signer);

  const balance = await api.query.system.account(signer.address);

  console.log('Balance: ', balance.toHuman());
  console.log('');

  // Tether USD, USDT, 2, 10亿
  const contract = await contractFactory.deployed('erc20,new', '100000000000', 'Tether USD', 'USDT', '2', {
    gasLimit: '200000000000',
    value: '10000000000000',
    salt: 'Tether USD ERC20'
  });

  console.log(
    'Deploy USDT successfully. The contract address: ',
    contract.address.toString()
  );
  console.log('');

  // Jupiter Bitcoin, jBTC, 8, 1百万
  const contract = await contractFactory.deployed('erc20,new', '100000000000000', 'Jupiter Bitcoin', 'jBTC', '8', {
    gasLimit: '200000000000',
    value: '10000000000000',
    salt: 'Jupiter Bitcoin ERC20'
  });

  console.log(
    'Deploy jBTC successfully. The contract address: ',
    contract.address.toString()
  );
  console.log('');

  // Jupiter Ethereum, jETH, 18, 1千万
  const contract = await contractFactory.deployed('erc20,new', '10000000000000000000000000', 'Jupiter Ethereum', 'jETH', '18', {
    gasLimit: '200000000000',
    value: '10000000000000',
    salt: 'Jupiter Ethereum ERC20'
  });

  console.log(
    'Deploy jETH successfully. The contract address: ',
    contract.address.toString()
  );
  console.log('');

  // Maker DAI, DAI
  const contract = await contractFactory.deployed('erc20,new', '0', 'Maker DAI', 'DAI', '0', {
    gasLimit: '200000000000',
    value: '10000000000000',
    salt: 'Maker DAI ERC20'
  });

  console.log(
    'Deploy DAI successfully. The contract address: ',
    contract.address.toString()
  );
  console.log('');

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
});
