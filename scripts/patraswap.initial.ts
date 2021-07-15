import {patract, network} from 'redspot';

const {getContractAt} = patract;
const {api, getSigners} = network;

async function run() {
    await api.isReady;

    const signers = await getSigners();
    const signer = signers[0];

    const balance = await api.query.system.account(signer.address);
    console.log('Balance: ', balance.toHuman());

    const usdt = api.createType('AccountId', "3enYFZYCVXW2x6orC4YxNEczjEzZ94vpadaqVEJQJfwWBVq9");
    const dai = api.createType('AccountId', "3bxKXL337YG2ZW91kYDM2ZEduFofVUGcYk7jLUiLFWmoKUdf");
    const eth = api.createType('AccountId', "3byZbKJSpCwUFDgNyeNfCz5PVyGvehhcNe3USJSvVDyJ2rrM");
    const btc = api.createType('AccountId', "3fT7tskxhBsCMza8FgZaHVLXwra55FWb8B4q8DauRUqmyQxK");

    const swap = api.createType('AccountId', "3f77g7XQDo11MMgnte4VnUqGcYA44WSWDbY2H9dzJNDKa7VQ")
    const contract = await getContractAt('factory', swap, signer);

    await contract.query['factory,getSwapPairs']();
    // 注意因为一些错误需要一组一组的create，但调用都是成功的
    // USDT -> DAI
    await contract.tx['factory,createExchange'](usdt, dai, undefined);

    // // USDT -> DOT
    // await contract.tx['factory,createExchangeWithDot'](usdt, undefined);
    //
    // // USDT -> jETH
    // await contract.tx['factory,createExchange'](usdt, eth, undefined);
    //
    // // USDT -> jBTC
    // await contract.tx['factory,createExchange'](usdt, btc, undefined);
    //
    // // jBTC -> DOT
    // await contract.tx['factory,createExchangeWithDot'](btc, undefined);

    api.disconnect();
}

run().catch((err) => {
    console.log(err);
});
