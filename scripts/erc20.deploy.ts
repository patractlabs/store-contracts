import {patract, network} from 'redspot';

const {getContractFactory} = patract;
const {api, getSigners} = network;

async function run() {
    await api.isReady;

    const signers = await getSigners();
    const signer = signers[0];
    const contractFactory = await getContractFactory('erc20_fixed', signer);

    const balance = await api.query.system.account(signer.address);

    console.log('Balance: ', balance.toHuman());

    const contract = await contractFactory.deployed('new', '10000000000000000', 'Jupiter Token', 'JPT', '10', {
        gasLimit: '200000000000',
        value: '0',
        salt: 'Jupiter Token'
    });

    console.log('');
    console.log(
        'Deploy successfully. The contract address: ',
        contract.address.toString()
    );

    api.disconnect();
}

run().catch((err) => {
    console.log(err);
});
