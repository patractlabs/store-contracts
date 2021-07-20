import {patract, network} from 'redspot';

const {getContractFactory} = patract;
const {api, getSigners} = network;

async function run() {
    await api.isReady;

    const signers = await getSigners();
    const signer = signers[0];
    const contractFactory = await getContractFactory('factory', signer);

    const balance = await api.query.system.account(signer.address);

    console.log('Balance: ', balance.toHuman());

    const exchangeFactory = await getContractFactory('exchange', signer);
    await exchangeFactory.deployed('new',
        '3e7hitVykGr2EyVdh81cph6CTRu5Y5VMEGQPSTtakRP32TE3',
        '3e7hitVykGr2EyVdh81cph6CTRu5Y5VMEGQPSTtakRP32TE3',
        '3e7hitVykGr2EyVdh81cph6CTRu5Y5VMEGQPSTtakRP32TE3', {
            gasLimit: '200000000000',
            value: '0',
            salt: 'PatraSwap'
        });
    console.log('');
    exchangeFactory.abi.project.source.wasmHash.toHex()

    const exchange2Factory = await getContractFactory('exchange2', signer);
    await exchange2Factory.deployed('new',
        '3e7hitVykGr2EyVdh81cph6CTRu5Y5VMEGQPSTtakRP32TE3',
        '3e7hitVykGr2EyVdh81cph6CTRu5Y5VMEGQPSTtakRP32TE3', {
            gasLimit: '200000000000',
            value: '0',
            salt: 'PatraSwap'
        });
    console.log('');

    const lptFactory = await getContractFactory('lpt', signer);
    await lptFactory.deployed('new', '', '', '', '', signer.address, {
        gasLimit: '200000000000',
        value: '0',
        salt: 'PatraSwap'
    });
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

    await contract.tx['factory,initializeFactory'](exchangeFactory.abi.project.source.wasmHash.toHex(),
        exchange2Factory.abi.project.source.wasmHash.toHex(), lptFactory.abi.project.source.wasmHash.toHex());
    console.log('initializeFactory successfully.');
    console.log('');

    api.disconnect();
}

run().catch((err) => {
    console.log(err);
});
