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

    const usdt = api.createType('AccountId', "13vTbmbUoXXt1QDfRqny1n7Yxm7gsaZqVqa3X48gWqTH1iPc");
    const dai = api.createType('AccountId', "14oZHYiuo6G9SrvjhKg24MCbFjhHMuxhd4ZFvqSBmy12JqSu");
    const eth = api.createType('AccountId', "128WkLU8zYUVT3XPSooYwZE7xUYUbHuMQGPZrGcf4VHARNUu");
    const btc = api.createType('AccountId', "12GPiDqSWJr4JFuuZuvG9oskdSnnH1ukE83WaeEExJRjAAxC");

    const swap = api.createType('AccountId', "1vtPXR3PHVZDFUoyvRVfGreDZ9G6uKYj8dTWhLuMZ7N1VQp")

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
