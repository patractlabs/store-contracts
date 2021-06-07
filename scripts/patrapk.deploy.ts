import {patract, network} from 'redspot';

const {getContractFactory} = patract;
const {api, getSigners} = network;

async function run() {
    await api.isReady;

    const signers = await getSigners();
    const signer = signers[0];
    const contractFactory = await getContractFactory('patrapk', signer);

    const balance = await api.query.system.account(signer.address);

    console.log('Balance: ', balance.toHuman());

    const contract = await contractFactory.deployed('new', 14400, {
        gasLimit: '200000000000',
        value: '0',
        salt: 'PatraPK'
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
