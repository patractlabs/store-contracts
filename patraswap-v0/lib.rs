#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod factory {
    use ink_lang as ink;

    use exchange::PatraExchange;
    use ink_storage::collections::HashMap as StorageHashMap;

    #[ink(storage)]
    pub struct PatraFactory {
        exchange_template: Hash,
        token_count: u128,
        token_to_exchange: StorageHashMap<AccountId, AccountId>,
        id_to_token: StorageHashMap<u128, AccountId>,
    }

    #[ink::trait_definition]
    pub trait Factory {
        #[ink(constructor)]
        fn new() -> Self;

        #[ink(message)]
        fn initialize_factory(&mut self, template: Hash);

        #[ink(message)]
        fn create_exchange(&mut self, token: AccountId) -> AccountId;

        #[ink(message)]
        fn get_exchange(&self, token: AccountId) -> AccountId;

        #[ink(message)]
        fn get_token_with_id(&self, token_id: u128) -> AccountId;
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
                token_count: 0,
                token_to_exchange: StorageHashMap::new(),
                id_to_token: StorageHashMap::new(),
            }
        }

        // Can't call initializeFactory on factory twice
        #[ink(message)]
        fn initialize_factory(&mut self, template: Hash) {
            let zero_hash = Hash::from([0; 32]);
            assert_eq!(self.exchange_template, zero_hash);
            assert_ne!(template, zero_hash);
            // exchange template contract code hash
            self.exchange_template = template;
        }

        #[ink(message)]
        fn create_exchange(&mut self, token: AccountId) -> AccountId {
            assert_ne!(token, AccountId::from([0; 32]));
            assert_ne!(self.exchange_template, Hash::from([0; 32]));
            assert!(!self.token_to_exchange.contains_key(&token));

            // instantiate exchange
            let salt = 0_u32.to_le_bytes();
            let total_balance = Self::env().balance();
            PatraExchange::new(token)
                .endowment(total_balance / 2)
                .code_hash(self.exchange_template)
                .salt_bytes(salt)
                .instantiate()
                .expect("failed at instantiating the `exchange` contract");

            let exchange_account_id = self.env().caller();
            self.token_to_exchange.insert(token, exchange_account_id);
            self.token_count += 1;
            self.id_to_token.insert(self.token_count, token);
            Self::env().emit_event(NewExchange {
                token,
                caller: exchange_account_id,
            });
            exchange_account_id
        }

        #[ink(message)]
        fn get_exchange(&self, token: AccountId) -> AccountId {
            self.token_to_exchange
                .get(&token)
                .copied()
                .unwrap_or(AccountId::from([0; 32]))
        }

        #[ink(message)]
        fn get_token_with_id(&self, token_id: u128) -> AccountId {
            self.id_to_token
                .get(&token_id)
                .copied()
                .unwrap_or(AccountId::from([0; 32]))
        }
    }
}
