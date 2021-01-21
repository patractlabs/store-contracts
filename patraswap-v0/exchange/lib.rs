#![cfg_attr(not(feature = "std"), no_std)]

pub use self::exchange::PatraExchange;
use ink_lang as ink;

#[ink::contract]
mod exchange {
    #[cfg(not(feature = "ink-as-dependency"))]
    use erc20::StandardToken;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_env::call::FromAccountId;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_prelude::string::String;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{collections::HashMap as StorageHashMap, Lazy};

    #[ink(storage)]
    pub struct PatraExchange {
        // Patraswap
        name: String,
        // PAT
        symbol: String,
        // 18
        decimals: u128,
        // total number of PAT in existence.
        total_supply: Balance,
        // PAT balance of an account (LP token)
        balances: StorageHashMap<AccountId, Balance>,
        // address of the ERC20 token traded on this contract
        token: Lazy<StandardToken>,
        token_account: AccountId,
    }

    #[ink(event)]
    pub struct TokenPurchase {
        #[ink(topic)]
        buyer: AccountId,
        #[ink(topic)]
        eth_sold: Balance,
        #[ink(topic)]
        tokens_bought: Balance,
    }

    #[ink(event)]
    pub struct EthPurchase {
        #[ink(topic)]
        buyer: AccountId,
        #[ink(topic)]
        tokens_sold: Balance,
        #[ink(topic)]
        eth_bought: Balance,
    }

    #[ink(event)]
    pub struct AddLiquidity {
        #[ink(topic)]
        sender: AccountId,
        #[ink(topic)]
        eth_amount: Balance,
        #[ink(topic)]
        token_amount: Balance,
    }

    #[ink(event)]
    pub struct RemoveLiquidity {
        #[ink(topic)]
        sender: AccountId,
        #[ink(topic)]
        eth_amount: Balance,
        #[ink(topic)]
        token_amount: Balance,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    #[ink(event)]
    pub struct NewExchange {
        #[ink(topic)]
        token: AccountId,
        #[ink(topic)]
        exchange: AccountId,
    }

    impl PatraExchange {
        #[ink(constructor)]
        pub fn new(token_account: AccountId) -> Self {
            let token: StandardToken = FromAccountId::from_account_id(token_account);
            Self::env().emit_event(NewExchange {
                token: token_account,
                exchange: Self::env().account_id(),
            });
            Self {
                name: "Patraswap".parse().unwrap(),
                symbol: "PAT".parse().unwrap(),
                decimals: 18,
                total_supply: 0,
                balances: StorageHashMap::new(),
                token: Lazy::new(token),
                token_account,
            }
        }

        #[ink(message, payable)]
        pub fn eth_to_token_swap_input(&mut self) -> Balance {
            let caller = self.env().caller();
            self.eth_to_token_input(self.env().transferred_balance(), caller, caller)
        }

        #[ink(message, payable)]
        pub fn eth_to_token_swap_output(&mut self, tokens_bought: Balance) -> Balance {
            let caller = self.env().caller();
            return self.eth_to_token_output(
                tokens_bought,
                self.env().transferred_balance(),
                caller,
                caller,
            );
        }

        #[ink(message)]
        pub fn token_to_eth_swap_input(&mut self, tokens_sold: Balance) -> Balance {
            let caller = self.env().caller();
            self.token_to_eth_input(tokens_sold, caller, caller)
        }

        #[ink(message)]
        pub fn token_to_eth_swap_output(&mut self, eth_bought: Balance) -> Balance {
            let caller = self.env().caller();
            self.token_to_eth_output(eth_bought, caller, caller)
        }

        #[ink(message)]
        pub fn token_to_token_swap_input(
            &mut self,
            _tokens_sold: Balance,
            _min_tokens_bought: Balance,
            _min_eth_bought: Balance,
            _token_addr: AccountId,
        ) -> Balance {
            unimplemented!()
        }

        #[ink(message)]
        pub fn token_to_token_swap_output(
            &mut self,
            _tokens_bought: Balance,
            _max_tokens_sold: Balance,
            _max_eth_sold: Balance,
            _token_addr: AccountId,
        ) -> Balance {
            unimplemented!()
        }

        /// Public price function for Token to ETH trades with an exact input.
        #[ink(message)]
        pub fn get_token_to_eth_input_price(&self, tokens_sold: Balance) -> Balance {
            assert!(tokens_sold > 0);
            let token_reserve: Balance = self.token.balance_of(self.env().account_id());
            Self::get_input_price(tokens_sold, token_reserve, self.env().balance())
        }

        /// Public price function for ETH to Token trades with an exact input.
        #[ink(message)]
        pub fn get_eth_to_token_input_price(&self, eth_sold: Balance) -> Balance {
            assert!(eth_sold > 0);
            let token_reserve: Balance = self.token.balance_of(self.env().account_id());
            Self::get_input_price(eth_sold, self.env().balance(), token_reserve)
        }

        /// Public price function for ETH to Token trades with an exact output.
        #[ink(message)]
        pub fn get_eth_to_token_output_price(&self, tokens_bought: Balance) -> Balance {
            assert!(tokens_bought > 0);
            let token_reserve: Balance = self.token.balance_of(self.env().account_id());
            let eth_sold: Balance =
                Self::get_output_price(tokens_bought, self.env().balance(), token_reserve);
            return eth_sold;
        }

        /// Public price function for Token to ETH trades with an exact output.
        #[ink(message)]
        pub fn get_token_to_eth_output_price(&self, eth_bought: Balance) -> Balance {
            assert!(eth_bought > 0);
            let token_reserve: Balance = self.token.balance_of(self.env().account_id());
            Self::get_output_price(eth_bought, token_reserve, self.env().balance())
        }

        fn eth_to_token_input(
            &mut self,
            eth_sold: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(eth_sold > 0);
            let token_reserve: Balance = self.token.balance_of(self.env().account_id());
            let tokens_bought: Balance =
                Self::get_input_price(eth_sold, self.env().balance() - eth_sold, token_reserve);
            assert!(self.token.transfer(recipient, tokens_bought).is_ok());
            self.env().emit_event(TokenPurchase {
                buyer,
                eth_sold,
                tokens_bought,
            });
            tokens_bought
        }

        fn eth_to_token_output(
            &mut self,
            tokens_bought: Balance,
            max_eth: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(tokens_bought > 0 && max_eth > 0);
            let token_reserve: Balance = self.token.balance_of(self.env().account_id());
            let eth_sold: Balance = Self::get_output_price(
                tokens_bought,
                self.env().balance() - max_eth,
                token_reserve,
            );
            assert!(eth_sold > max_eth);
            let eth_refund: Balance = max_eth - eth_sold;
            if eth_refund > 0 {
                assert!(self.env().transfer(buyer, eth_refund).is_ok());
            }
            assert!(self.token.transfer(recipient, tokens_bought).is_ok());
            self.env().emit_event(TokenPurchase {
                buyer,
                eth_sold,
                tokens_bought,
            });
            eth_sold
        }

        fn token_to_eth_input(
            &mut self,
            tokens_sold: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(tokens_sold > 0);
            let contract_account = self.env().account_id();
            let token_reserve: Balance = self.token.balance_of(contract_account);
            let eth_bought: Balance =
                Self::get_input_price(tokens_sold, token_reserve, self.env().balance());
            assert!(self.env().transfer(recipient, eth_bought).is_ok());
            assert!(self
                .token
                .transfer_from(buyer, contract_account, tokens_sold)
                .is_ok());
            self.env().emit_event(EthPurchase {
                buyer,
                tokens_sold,
                eth_bought,
            });
            eth_bought
        }

        fn token_to_eth_output(
            &mut self,
            eth_bought: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(eth_bought > 0);
            let contract_account = self.env().account_id();
            let token_reserve: Balance = self.token.balance_of(contract_account);
            let tokens_sold: Balance =
                Self::get_output_price(eth_bought, token_reserve, self.env().balance());
            assert!(self.env().transfer(recipient, eth_bought).is_ok());
            assert!(self
                .token
                .transfer_from(buyer, contract_account, tokens_sold)
                .is_ok());
            self.env().emit_event(EthPurchase {
                buyer,
                tokens_sold,
                eth_bought,
            });
            tokens_sold
        }

        fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(&owner).copied().unwrap_or(0)
        }
    }

    impl PatraExchange {
        /// Deposit ETH and Tokens (self.token) at current ratio to mint PAT tokens.
        // @param max_tokens Maximum number of tokens deposited. Deposits max amount if total PAT supply is 0.
        // @return The amount of PAT minted.
        #[ink(message, payable)]
        pub fn add_liquidity(&mut self, max_tokens: Balance) -> Balance {
            let caller = self.env().caller();
            let value: Balance = self.env().transferred_balance();
            let contract_account = self.env().account_id();
            assert!(max_tokens > 0 && value > 0);
            let total_liquidity: Balance = self.total_supply;
            if total_liquidity > 0 {
                let eth_reserve = self.env().balance() - value;
                let token_reserve = self.token.balance_of(self.env().account_id());
                let token_amount = value * token_reserve / eth_reserve + 1;
                let liquidity_minted = value * total_liquidity / eth_reserve;
                assert!(max_tokens >= token_amount);
                let from_balance = self.balance_of(caller);
                self.balances
                    .insert(caller, from_balance + liquidity_minted);
                self.total_supply = total_liquidity + liquidity_minted;
                assert!(self
                    .token
                    .transfer_from(caller, contract_account, token_amount)
                    .is_ok());
                self.env().emit_event(AddLiquidity {
                    sender: caller,
                    eth_amount: value,
                    token_amount,
                });
                self.env().emit_event(Transfer {
                    from: AccountId::from([0; 32]),
                    to: caller,
                    value: liquidity_minted,
                });
                liquidity_minted
            } else {
                assert!(self.env().transferred_balance() >= 10000);
                let token_amount: Balance = max_tokens;
                let initial_liquidity: Balance = self.env().balance();
                self.total_supply = initial_liquidity;
                self.balances.insert(caller, initial_liquidity);
                assert!(self
                    .token
                    .transfer_from(caller, contract_account, token_amount)
                    .is_ok());
                self.env().emit_event(AddLiquidity {
                    sender: caller,
                    eth_amount: value,
                    token_amount,
                });
                self.env().emit_event(Transfer {
                    from: AccountId::from([0; 32]),
                    to: caller,
                    value: initial_liquidity,
                });
                initial_liquidity
            }
        }

        /// Burn PAT tokens to withdraw ETH and Tokens at current ratio.
        // @param amount Amount of PAT burned.
        // @return The amount of ETH and Tokens withdrawn.
        #[ink(message)]
        pub fn remove_liquidity(&mut self, lp_amount: Balance) -> (Balance, Balance) {
            assert!(lp_amount > 0);
            let total_liquidity = self.total_supply;
            assert!(total_liquidity > 0);
            let token_reserve = self.token.balance_of(self.env().account_id());
            let eth_amount = lp_amount * self.env().balance() / total_liquidity;
            let token_amount = lp_amount * token_reserve / total_liquidity;
            let caller = self.env().caller();
            let from_balance = self.balance_of(caller);
            self.balances.insert(caller, from_balance - lp_amount);
            self.total_supply = total_liquidity - lp_amount;
            assert!(self.env().transfer(caller, eth_amount).is_ok());
            assert!(self.token.transfer(caller, token_amount).is_ok());
            self.env().emit_event(RemoveLiquidity {
                sender: caller,
                eth_amount,
                token_amount,
            });
            self.env().emit_event(Transfer {
                from: caller,
                to: AccountId::from([0; 32]),
                value: lp_amount,
            });
            (eth_amount, token_amount)
        }

        /// Returns the PAT total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }
    }

    impl PatraExchange {
        // Pricing function for converting between ETH and Tokens.
        #[cfg(not(feature = "ink-as-dependency"))]
        fn get_input_price(
            input_amount: Balance,
            input_reserve: Balance,
            output_reserve: Balance,
        ) -> Balance {
            assert!(input_reserve > 0 && output_reserve > 0);
            let numerator: Balance = input_amount.saturating_mul(output_reserve);
            let denominator: Balance = input_reserve.saturating_add(input_amount);
            numerator / denominator
        }

        // Pricing function for converting between ETH and Tokens.
        #[cfg(not(feature = "ink-as-dependency"))]
        fn get_output_price(
            output_amount: Balance,
            input_reserve: Balance,
            output_reserve: Balance,
        ) -> Balance {
            assert!(input_reserve > 0 && output_reserve > 0);
            let numerator: Balance = output_reserve.saturating_mul(output_amount);
            let denominator: Balance = input_reserve.saturating_sub(output_amount);
            numerator / denominator + 1
        }
    }
}
