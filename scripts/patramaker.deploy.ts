import {patract, network} from 'redspot';
import type Contract from '@redspot/patract/contract';

const {getContractFactory} = patract;
const {getSigners, api} = network;

async function run() {
    await api.isReady;

    const signers = await getSigners();
    const signer = signers[0];
    const contractFactory = await getContractFactory('patramaker', signer);

    const balance = await api.query.system.account(signer.address);
    console.log('Balance: ', balance.toHuman());

    const daiContractFactory = await getContractFactory('erc20_issue', signer);
    //   Maker DAI, DAI 18
    let daiContract = await daiContractFactory.instantiate('IErc20,new', '0', 'Maker DAI', 'DAI', '18', {
        gasLimit: '200000000000',
        value: '0',
        salt: 'Maker DAI Token '
    });

    console.log(
        'Deploy DAI successfully. The contract address: ',
        daiContract.toString()
    );
    console.log('');

    // const daiContract = await daiContractFactory.deployed('new', '0', 'Maker DAI', 'DAI', '18', {
    //   gasLimit: '200000000000',
    //   value: '0',
    //   salt: 'Maker DAI Token'
    // });
    // console.log(
    //   'Deploy dai successfully. The contract address: ',
    //   daiContract.address.toString()
    // );
    // console.log('');

    const contract = await contractFactory.deployed('new', daiContract, {
        gasLimit: '200000000000',
        value: '0',
        salt: 'PatraMaker'
    });
    console.log(
        'Deploy maker successfully. The contract address: ',
        contract.address.toString()
    );

    // @ts-ignore
    const dcontract = new Contract(
        daiContract,
        daiContractFactory.abi,
        daiContractFactory.api,
        daiContractFactory.signer
    );

    // transfer dai contract ownership to maker
    await dcontract.tx.transferOwnership(contract.address.toString())

    // init dai with 100k DOT
    await contract.tx.issueDai(200, {
        value: 1000000000000000
    });

    api.disconnect();
}

run().catch((err) => {
    console.log(err);
});
