import { RedspotUserConfig } from 'redspot/types';
import '@redspot/patract';
import '@redspot/chai';
import '@redspot/gas-reporter';

const defaultNetwork = process.env.REDSPOT_ENV ? process.env.REDSPOT_ENV : 'development'

export default {
  defaultNetwork,
  contract: {
    ink: {
      toolchain: 'nightly',
      sources: ['contracts/**/*']
    }
  },
  networks: {
    development: {
      endpoint: 'wss://jupiter-poa.elara.patract.io/',
      types: {
        LookupSource: "MultiAddress",
      },
      gasLimit: '400000000000',
      explorerUrl: 'https://polkadot.js.org/apps/#/explorer/query/',
      accounts: ['//Alice'],
    },
    development_para: {
      endpoint: 'wss://ws.jupiter.patract-westend.patract.cn/',
      types: {
        LookupSource: "MultiAddress",
      },
      gasLimit: '400000000000',
      explorerUrl: 'https://polkadot.js.org/apps/#/explorer/query/',
      accounts: ['//Alice'],
    },
    paradev: {
      endpoint: 'wss://ws.jupiter.patract-westend.patract.cn/',
      types: {
        LookupSource: "MultiAddress",
      },
      gasLimit: '400000000000',
      explorerUrl: 'https://polkadot.js.org/apps/#/explorer/query/',
      accounts: ['//Alice'],
    },
    substrate: {
      endpoint: 'ws://127.0.0.1:9944',
      gasLimit: '400000000000',
      accounts: ['//Alice'],
      types: {}
    }
  },
  mocha: {
    timeout: 60000
  }
} as RedspotUserConfig;
