#![cfg_attr(not(feature = "std"), no_std)]

pub use self::exchange::{Exchange, ExchangeStub};
use ink_lang as ink;

#[ink::contract]
mod exchange {
    use ink_lang as ink;

    #[ink::trait_definition]
    pub trait Exchange {
        #[ink(constructor)]
        fn new(_token: AccountId) -> Self;
        /// Convert ETH to Tokens.
        /// User specifies maximum input (msg.value) and exact output.
        #[ink(message)]
        fn eth_to_token_swap_input(&mut self) -> Balance;

        #[ink(message)]
        fn eth_to_token_swap_output(&mut self, tokens_bought: Balance) -> Balance;

        #[ink(message)]
        fn token_to_eth_swap_input(&mut self, tokens_sold: Balance) -> Balance;

        #[ink(message)]
        fn token_to_eth_swap_output(&mut self, eth_bought: Balance) -> Balance;

        #[ink(message)]
        fn token_to_token_swap_input(
            &mut self,
            tokens_sold: Balance,
            min_tokens_bought: Balance,
            min_eth_bought: Balance,
            token_addr: AccountId,
        ) -> Balance;

        #[ink(message)]
        fn token_to_token_swap_output(
            &mut self,
            tokens_bought: Balance,
            max_tokens_sold: Balance,
            max_eth_sold: Balance,
            token_addr: AccountId,
        ) -> Balance;

        #[ink(message)]
        fn get_token_to_eth_input_price(&self, tokens_sold: Balance) -> Balance;

        #[ink(message)]
        fn get_eth_to_token_input_price(&self, eth_sold: Balance) -> Balance;

        #[ink(message)]
        fn get_eth_to_token_output_price(&self, tokens_bought: Balance) -> Balance;

        #[ink(message)]
        fn get_token_to_eth_output_price(&self, eth_bought: Balance) -> Balance;
    }

    #[ink(storage)]
    pub struct ExchangeStub {}

    impl Exchange for ExchangeStub {
        #[ink(constructor)]
        fn new(_token: AccountId) -> Self {
            unimplemented!()
        }

        #[ink(message, payable)]
        fn eth_to_token_swap_input(&mut self) -> Balance {
            unimplemented!()
        }

        #[ink(message, payable)]
        fn eth_to_token_swap_output(&mut self, _tokens_bought: Balance) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        fn token_to_eth_swap_input(&mut self, _tokens_sold: Balance) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        fn token_to_eth_swap_output(&mut self, _eth_bought: Balance) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        fn token_to_token_swap_input(
            &mut self,
            _tokens_sold: Balance,
            _min_tokens_bought: Balance,
            _min_eth_bought: Balance,
            _token_addr: AccountId,
        ) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        fn token_to_token_swap_output(
            &mut self,
            _tokens_bought: Balance,
            _max_tokens_sold: Balance,
            _max_eth_sold: Balance,
            _token_addr: AccountId,
        ) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        fn get_token_to_eth_input_price(&self, _tokens_sold: Balance) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        fn get_eth_to_token_input_price(&self, _eth_sold: Balance) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        fn get_eth_to_token_output_price(&self, _tokens_bought: Balance) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        fn get_token_to_eth_output_price(&self, _eth_bought: Balance) -> Balance {
            unimplemented!()
        }
    }
}
