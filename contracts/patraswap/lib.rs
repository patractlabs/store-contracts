#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod factory {
    use ink_lang as ink;

    use erc20_stub::Erc20Stub;

    use lpt::Erc20;
    use exchange::PatraExchange;
    use exchange2::PatraExchange as PatraExchange2;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_env::call::FromAccountId;
    use ink_env::hash::Blake2x256;
    use ink_prelude::vec::Vec;
    use ink_storage::collections::HashMap as StorageHashMap;
    use scale::Encode;

    #[ink(storage)]
    pub struct PatraFactory {
        exchange_template: Hash,
        exchange_template2: Hash,
        lpt: Hash,
        token_count: u128,
        swap_pairs: Vec<SwapPair>,
        token_to_exchange: StorageHashMap<SwapPair, AccountId>,
        id_to_token: StorageHashMap<u128, SwapPair>,
        id_to_exchange: StorageHashMap<u128, ExchangeAccount>,
    }

    pub type SwapPair = (AccountId, AccountId);
    pub type ExchangeAccount = (AccountId, AccountId);

    #[ink::trait_definition]
    pub trait Factory {
        #[ink(constructor)]
        fn new() -> Self;

        #[ink(message)]
        fn initialize_factory(&mut self, template: Hash, template2: Hash, lpt: Hash);

        #[ink(message)]
        fn create_exchange(
            &mut self,
            from_token: AccountId,
            to_token: AccountId,
            salt_op: Option<Hash>,
        );

        #[ink(message)]
        fn create_exchange_with_dot(&mut self, token: AccountId, salt_op: Option<Hash>);

        #[ink(message)]
        fn get_exchange(&self, from_token: AccountId, to_token: AccountId) -> Option<AccountId>;

        #[ink(message)]
        fn get_token_with_id(&self, token_id: u128) -> Option<SwapPair>;

        #[ink(message)]
        fn get_exchange_with_id(&self, token_id: u128) -> Option<ExchangeAccount>;

        #[ink(message)]
        fn get_swap_pairs(&self) -> Vec<SwapPair>;
    }

    #[ink(event)]
    pub struct NewExchange {
        #[ink(topic)]
        token: AccountId,
        #[ink(topic)]
        exchange: AccountId,
        #[ink(topic)]
        lpt: AccountId,
    }

    impl Factory for PatraFactory {
        #[ink(constructor)]
        fn new() -> Self {
            Self {
                exchange_template: Default::default(),
                exchange_template2: Default::default(),
                lpt: Default::default(),
                token_count: 0,
                swap_pairs: Vec::new(),
                token_to_exchange: StorageHashMap::new(),
                id_to_token: StorageHashMap::new(),
                id_to_exchange: StorageHashMap::new(),
            }
        }

        // Can't call initializeFactory on factory twice
        #[ink(message)]
        fn initialize_factory(&mut self, template: Hash, template2: Hash, lpt: Hash) {
            assert_eq!(self.exchange_template, Default::default());
            assert_ne!(template, Default::default());
            // exchange template contract code hash
            self.exchange_template = template;
            self.exchange_template2 = template2;
            self.lpt = lpt;
        }

        #[ink(message)]
        fn create_exchange(
            &mut self,
            from_token: AccountId,
            to_token: AccountId,
            salt_op: Option<Hash>,
        ) {
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

            let salt;
            if salt_op.is_none() {
                let mut from = from_token.encode();
                from.extend(to_token.encode());
                salt = Hash::from(self.env().hash_bytes::<Blake2x256>(from.as_slice()));
            } else {
                salt = salt_op.unwrap();
            }

            let from_token_contract: Erc20Stub = FromAccountId::from_account_id(from_token);

            // instantiate lp token
            let lpt_params = Erc20::new(
                0,
                Some("LP Token".parse().unwrap()),
                Some("LPT".parse().unwrap()),
                from_token_contract.token_decimals(),
            )
            .endowment(0)
            .code_hash(self.lpt)
            .salt_bytes(salt)
            .params();
            let lpt_account_id = self
                .env()
                .instantiate_contract(&lpt_params)
                .expect("failed at instantiating the `lp token` contract");

            let salt = Hash::from(self.env().hash_bytes::<Blake2x256>(salt.clone().as_ref()));
            // instantiate exchange
            let exchange_params = PatraExchange::new(from_token, to_token, lpt_account_id)
                .endowment(0)
                .code_hash(self.exchange_template)
                .salt_bytes(salt)
                .params();
            let exchange_account_id = self
                .env()
                .instantiate_contract(&exchange_params)
                .expect("failed at instantiating the `exchange` contract");

            self.token_to_exchange
                .insert((from_token, to_token), exchange_account_id);
            self.swap_pairs.push((from_token, to_token));
            self.token_count += 1;
            self.id_to_token
                .insert(self.token_count, (from_token, to_token));
            self.id_to_exchange
                .insert(self.token_count, (exchange_account_id, lpt_account_id));
            Self::env().emit_event(NewExchange {
                token: from_token,
                exchange: exchange_account_id,
                lpt: lpt_account_id,
            });
        }

        /// Create ERC20 Token => DOT
        #[ink(message)]
        fn create_exchange_with_dot(&mut self, from_token: AccountId, salt_op: Option<Hash>) {
            assert_ne!(self.exchange_template2, Hash::from([0; 32]));
            assert_ne!(from_token, Default::default());
            let to_token = Default::default();
            for item in self.swap_pairs.iter() {
                if (item.0 == from_token && item.1 == to_token)
                    || item.0 == to_token && item.1 == from_token
                {
                    assert!(false)
                }
            }
            assert!(!self.token_to_exchange.contains_key(&(from_token, to_token)));

            let salt;
            if salt_op.is_none() {
                let mut from = from_token.encode();
                from.extend(to_token.encode());
                salt = Hash::from(self.env().hash_bytes::<Blake2x256>(from.as_slice()));
            } else {
                salt = salt_op.unwrap();
            }

            let from_token_contract: Erc20Stub = FromAccountId::from_account_id(from_token);
            // instantiate lp token
            let lpt_params = Erc20::new(
                0,
                Some("LP Token".parse().unwrap()),
                Some("LPT".parse().unwrap()),
                from_token_contract.token_decimals(),
            )
            .endowment(0)
            .code_hash(self.lpt)
            .salt_bytes(salt)
            .params();
            let lpt_account_id = self
                .env()
                .instantiate_contract(&lpt_params)
                .expect("failed at instantiating the `lp token` contract");

            let salt = Hash::from(self.env().hash_bytes::<Blake2x256>(salt.clone().as_ref()));

            // instantiate exchange
            let exchange_params = PatraExchange2::new(from_token, lpt_account_id)
                .endowment(0)
                .code_hash(self.exchange_template2)
                .salt_bytes(salt)
                .params();
            let exchange_account_id = self
                .env()
                .instantiate_contract(&exchange_params)
                .expect("failed at instantiating the `exchange` contract");

            self.token_to_exchange
                .insert((from_token, to_token), exchange_account_id);
            self.swap_pairs.push((from_token, to_token));
            self.token_count += 1;
            self.id_to_token
                .insert(self.token_count, (from_token, to_token));
            self.id_to_exchange
                .insert(self.token_count, (exchange_account_id, lpt_account_id));
            Self::env().emit_event(NewExchange {
                token: from_token,
                exchange: exchange_account_id,
                lpt: lpt_account_id,
            });
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
        fn get_exchange_with_id(&self, token_id: u128) -> Option<ExchangeAccount> {
            self.id_to_exchange.get(&token_id).copied()
        }

        #[ink(message)]
        fn get_swap_pairs(&self) -> Vec<SwapPair> {
            self.swap_pairs.clone()
        }
    }
}
