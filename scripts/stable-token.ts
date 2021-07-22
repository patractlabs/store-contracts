import {patract, network} from 'redspot';

const {getContractFactory} = patract;
const {api, getSigners} = network;

async function run() {
    await api.isReady;

    const signers = await getSigners();
    const signer = signers[0];
    const contractFactory = await getContractFactory('erc20_issue', signer);

    const balance = await api.query.system.account(signer.address);

    console.log('Balance: ', balance.toHuman());
    console.log('');

    // Mock contract to put erc20 hash
    await contractFactory.deployed('new', '1000000000000000', 'MOCK', 'MOCK', '6', {
        gasLimit: '200000000000',
        value: '0',
        salt: 'MOCK'
    });

    // Tether USD, USDT, 2, 10亿
    let contract = await contractFactory.instantiate('new', '1000000000000000', 'Tether USD', 'USDT', '6', {
        gasLimit: '200000000000',
        value: '0',
        salt: 'Tether USD Token'
    });

    console.log(
        'Deploy USDT successfully. The contract address: ',
        contract.toString()
    );
    console.log('');

    // Jupiter Bitcoin, jBTC, 8, 1百万
    contract = await contractFactory.instantiate('new', '100000000000000', 'Jupiter Bitcoin', 'jBTC', '8', {
        gasLimit: '200000000000',
        value: '0',
        salt: 'Jupiter Bitcoin Token'
    });

    console.log(
        'Deploy jBTC successfully. The contract address: ',
        contract.toString()
    );
    console.log('');

    // Jupiter Ethereum, jETH, 18, 1千万
    contract = await contractFactory.instantiate('new', '10000000000000000000000000', 'Jupiter Ethereum', 'jETH', '18', {
        gasLimit: '200000000000',
        value: '0',
        salt: 'Jupiter Ethereum Token'
    });

    console.log(
        'Deploy jETH successfully. The contract address: ',
        contract.toString()
    );
    console.log('');

//   Moved to patramaker contract deploy
//   Maker DAI, DAI 18
//     contract = await contractFactory.instantiate('new', '0', 'Maker DAI', 'DAI', '18', {
//         gasLimit: '200000000000',
//         value: '0',
//         salt: 'Maker DAI Token'
//     });
//
//     console.log(
//         'Deploy DAI successfully. The contract address: ',
//         contract.toString()
//     );
//     console.log('');

    api.disconnect();
}

run().catch((err) => {
    console.log(err);
});
