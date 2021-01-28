#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod factory {
    use ink_lang as ink;

    use exchange::PatraExchange;
    use ink_prelude::vec::Vec;
    use ink_storage::collections::HashMap as StorageHashMap;
    use lpt::LPT;

    #[ink(storage)]
    pub struct PatraFactory {
        exchange_template: Hash,
        lpt_template: Hash,
        token_count: u128,
        // TODO
        swap_pairs: Vec<SwapPair>,
        token_to_exchange: StorageHashMap<SwapPair, AccountId>,
        id_to_token: StorageHashMap<u128, SwapPair>,
    }

    pub type SwapPair = (AccountId, AccountId);

    #[ink::trait_definition]
    pub trait Factory {
        #[ink(constructor)]
        fn new() -> Self;

        #[ink(message)]
        fn initialize_factory(&mut self, template: Hash, lpt: Hash);

        #[ink(message)]
        fn create_exchange(&mut self, from_token: AccountId, to_token: AccountId) -> AccountId;

        #[ink(message)]
        fn get_exchange(&self, from_token: AccountId, to_token: AccountId) -> Option<AccountId>;

        #[ink(message)]
        fn get_token_with_id(&self, token_id: u128) -> Option<SwapPair>;

        #[ink(message)]
        fn get_swap_pairs(&self) -> Vec<SwapPair>;
    }

    #[ink(event)]
    pub struct NewExchange {
        #[ink(topic)]
        token: AccountId,
        #[ink(topic)]
        caller: AccountId,
    }

    impl Factory for PatraFactory {
        #[ink(constructor)]
        fn new() -> Self {
            Self {
                exchange_template: Default::default(),
                lpt_template: Default::default(),
                token_count: 0,
                swap_pairs: Vec::new(),
                token_to_exchange: StorageHashMap::new(),
                id_to_token: StorageHashMap::new(),
            }
        }

        // Can't call initializeFactory on factory twice
        #[ink(message)]
        fn initialize_factory(&mut self, template: Hash, lpt: Hash) {
            assert_eq!(self.exchange_template, Default::default());
            assert_ne!(template, Default::default());
            // exchange template contract code hash
            self.exchange_template = template;
            self.lpt_template = lpt;
        }

        #[ink(message)]
        fn create_exchange(&mut self, from_token: AccountId, to_token: AccountId) -> AccountId {
            for item in self.swap_pairs.iter() {
                if (item.0 == from_token && item.1 == to_token)
                    || item.0 == to_token && item.1 == from_token
                {
                    assert!(false)
                }
            }
            assert_ne!(from_token, AccountId::from([0; 32]));
            assert_ne!(self.exchange_template, Hash::from([0; 32]));
            assert!(!self.token_to_exchange.contains_key(&(from_token, to_token)));

            let salt = 0_u32.to_le_bytes();
            let total_balance = Self::env().balance();
            // instantiate LP token
            let lpt_params = LPT::new()
                .endowment(total_balance / 10)
                .code_hash(self.lpt_template)
                .salt_bytes(salt)
                .params();
            let lpt_account_id = self
                .env()
                .instantiate_contract(&lpt_params)
                .expect("failed at instantiating the `lpt` contract");

            // instantiate exchange
            let exchange_params = PatraExchange::new(from_token, to_token, lpt_account_id)
                .endowment(total_balance / 10)
                .code_hash(self.exchange_template)
                .salt_bytes(salt)
                .params();
            let exchange_account_id = self
                .env()
                .instantiate_contract(&exchange_params)
                .expect("failed at instantiating the `exchange` contract");

            self.token_to_exchange
                .insert((from_token, to_token), exchange_account_id);
            self.token_count += 1;
            self.id_to_token
                .insert(self.token_count, (from_token, to_token));
            Self::env().emit_event(NewExchange {
                token: from_token,
                caller: exchange_account_id,
            });
            exchange_account_id
        }

        #[ink(message)]
        fn get_exchange(&self, from_token: AccountId, to_token: AccountId) -> Option<AccountId> {
            self.token_to_exchange.get(&(from_token, to_token)).cloned()
        }

        #[ink(message)]
        fn get_token_with_id(&self, token_id: u128) -> Option<SwapPair> {
            self.id_to_token.get(&token_id).copied()
        }

        #[ink(message)]
        fn get_swap_pairs(&self) -> Vec<SwapPair> {
            self.swap_pairs.clone()
        }
    }
}
