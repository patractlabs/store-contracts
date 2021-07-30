import {patract, network} from 'redspot';
import Contract from "@redspot/patract/contract";

const {getContractFactory} = patract;
const {api, getSigners} = network;

async function run() {
    await api.isReady;

    const signers = await getSigners();
    const signer = signers[0];

    const contractFactory = await getContractFactory('factory', signer);

    const balance = await api.query.system.account(signer.address);
    console.log('Balance: ', balance.toHuman());

    const usdt = api.createType('AccountId', "3faReJaqfwFLsEzozvJwRxGEwN17y3AyLzJChPdbBtktAc6o");
    const btc = api.createType('AccountId', "3dKmLie6EcEDJPyn1UEnKdZCBysNsmv2NNu2FzAAntkYYrVB");
    const eth = api.createType('AccountId', "3btyVk6ahBJS7syGu4GYAWmyfZAVHMrUeRLdL1kadKerv4i6");
    const dai = api.createType('AccountId', "3h2crvKgDDbtQGHACsfrpFp2pF959dkQptGqw1rGrfqSeQcF");

    const swap = api.createType('AccountId', "3d6zXxdiHUXFwg958yS5BeugwYSFxxR8xEBXGUqtqLfR5jfm")

    const contract = new Contract(
        swap,
        contractFactory.abi,
        contractFactory.api,
        contractFactory.signer
    );

    await contract.query['factory,getSwapPairs']();
    // USDT -> DAI
    await contract.tx['factory,createExchange'](usdt, dai, undefined);

    // USDT -> DOT
    await contract.tx['factory,createExchangeWithDot'](usdt, undefined);

    // USDT -> jETH
    await contract.tx['factory,createExchange'](usdt, eth, undefined);

    // USDT -> jBTC
    await contract.tx['factory,createExchange'](usdt, btc, undefined);

    // jBTC -> DOT
    await contract.tx['factory,createExchangeWithDot'](btc, undefined);

    api.disconnect();
}

run().catch((err) => {
    console.log(err);
});
