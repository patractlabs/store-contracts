#![cfg_attr(not(feature = "std"), no_std)]

pub use self::factory::{Factory, FactoryStub};
use ink_lang as ink;

#[ink::contract]
mod factory {
    use ink_lang as ink;

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
        fn get_token(&self, token: AccountId) -> AccountId;

        #[ink(message)]
        fn get_token_with_id(&self, token_id: u128) -> AccountId;
    }

    #[ink(storage)]
    pub struct FactoryStub {}

    impl Factory for FactoryStub {
        #[ink(constructor)]
        fn new() -> Self {
            unimplemented!()
        }

        #[ink(message)]
        fn initialize_factory(&mut self, _template: Hash) {
            unimplemented!()
        }

        #[ink(message)]
        fn create_exchange(&mut self, _token: AccountId) -> AccountId {
            unimplemented!()
        }

        #[ink(message)]
        fn get_exchange(&self, _token: AccountId) -> AccountId {
            unimplemented!()
        }

        #[ink(message)]
        fn get_token(&self, _token: AccountId) -> AccountId {
            unimplemented!()
        }

        #[ink(message)]
        fn get_token_with_id(&self, _token_id: u128) -> AccountId {
            unimplemented!()
        }
    }
}
