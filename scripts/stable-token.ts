import { patract, network } from 'redspot';
import type Contract from '@redspot/patract/contract';

const { getContractFactory } = patract;
const { createSigner, keyring, api } = network;

const uri =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

async function run() {
  await api.isReady;

  const signer = createSigner(keyring.createFromUri(uri));
  const contractFactory = await getContractFactory('erc20_issue', signer);

  const balance = await api.query.system.account(signer.address);

  console.log('Balance: ', balance.toHuman());
  console.log('');

  let contract: Contract

  // Tether USD, USDT, 2, 10亿
  contract = await contractFactory.deployed('IErc20,new', '1000000000000000', 'Tether USD', 'USDT', '6', {
    gasLimit: '200000000000',
    value: '0',
    salt: 'Tether USD Token'
  });

  console.log(
    'Deploy USDT successfully. The contract address: ',
    contract.address.toString()
  );
  console.log('');

  // Jupiter Bitcoin, jBTC, 8, 1百万
  contract = await contractFactory.deployed('IErc20,new', '100000000000000', 'Jupiter Bitcoin', 'jBTC', '8', {
    gasLimit: '200000000000',
    value: '0',
    salt: 'Jupiter Bitcoin Token'
  });

  console.log(
    'Deploy jBTC successfully. The contract address: ',
    contract.address.toString()
  );
  console.log('');

  // Jupiter Ethereum, jETH, 18, 1千万
  contract = await contractFactory.deployed('IErc20,new', '10000000000000000000000000', 'Jupiter Ethereum', 'jETH', '18', {
    gasLimit: '200000000000',
    value: '0',
    salt: 'Jupiter Ethereum Token'
  });

  console.log(
    'Deploy jETH successfully. The contract address: ',
    contract.address.toString()
  );
  console.log('');

  // Moved to patramaker contract deploy
  // Maker DAI, DAI 18
  // contract = await contractFactory.deployed('IErc20,new', '0', 'Maker DAI', 'DAI', '18', {
  //   gasLimit: '200000000000',
  //   value: '0',
  //   salt: 'Maker DAI Token'
  // });
  //
  // console.log(
  //   'Deploy DAI successfully. The contract address: ',
  //   contract.address.toString()
  // );
  // console.log('');

  const issue_code_hash = await contractFactory.putCode();
  console.log(
    'Put issue erc20 code successfully. The contract code hash: ',
    issue_code_hash.toString()
  );
  console.log('');

  const fixedFactory = await getContractFactory('erc20_fixed', signer);
  const fixed_code_hash = await fixedFactory.putCode();
  console.log(
    'Put fixed erc20 code successfully. The contract code hash: ',
    fixed_code_hash.toString()
  );
  console.log('');

  api.disconnect();
}

run().catch((err) => {
  console.log(err);
});
