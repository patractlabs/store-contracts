#![cfg_attr(not(feature = "std"), no_std)]

pub use self::exchange::PatraExchange;
use ink_lang as ink;

#[ink::contract]
mod exchange {
    #[cfg(not(feature = "ink-as-dependency"))]
    use lpt::Erc20;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_env::call::FromAccountId;
    use ink_prelude::string::String;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::Lazy;

    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct ExchangeInfo {
        pub from_symbol: String,
        pub from_decimals: u8,
        pub to_symbol: String,
        pub to_decimals: u8,
        pub from_token_pool: Balance,
        pub to_token_pool: Balance,
        pub lp_token_supply: Balance,
        pub own_lp_token: Balance,
    }

    #[ink(storage)]
    pub struct PatraExchange {
        // address of the ERC20 token traded on this contract
        token_contract: Lazy<Erc20>,
        lp_token_contract: Lazy<Erc20>,
        token: AccountId,
        init_deposit_dot: Balance,
    }

    #[ink(event)]
    pub struct TokenSwap {
        #[ink(topic)]
        buyer: AccountId,
        #[ink(topic)]
        sold: Balance,
        #[ink(topic)]
        bought: Balance,
    }

    #[ink(event)]
    pub struct AddLiquidity {
        #[ink(topic)]
        sender: AccountId,
        #[ink(topic)]
        from_amount: Balance,
        #[ink(topic)]
        to_amount: Balance,
    }

    #[ink(event)]
    pub struct RemoveLiquidity {
        #[ink(topic)]
        sender: AccountId,
        #[ink(topic)]
        from_amount: Balance,
        #[ink(topic)]
        to_amount: Balance,
    }

    #[ink(event)]
    pub struct NewExchangeWithDot {
        #[ink(topic)]
        token: AccountId,
        #[ink(topic)]
        exchange: AccountId,
    }

    impl PatraExchange {
        #[ink(constructor)]
        pub fn new(token: AccountId, lpt: AccountId) -> Self {
            let token_contract: Erc20 = FromAccountId::from_account_id(token);
            let lp_token_contract: Erc20 = FromAccountId::from_account_id(lpt);
            Self::env().emit_event(NewExchangeWithDot {
                token,
                exchange: Self::env().account_id(),
            });
            Self {
                token_contract: Lazy::new(token_contract),
                lp_token_contract: Lazy::new(lp_token_contract),
                token,
                init_deposit_dot: Self::env().balance(),
            }
        }

        #[ink(message, payable)]
        pub fn swap_dot_to_token_input(&mut self) -> Balance {
            let caller = self.env().caller();
            self.dot_to_token_input(self.env().transferred_balance(), caller, caller)
        }

        #[ink(message, payable)]
        pub fn swap_dot_to_token_output(&mut self, tokens_bought: Balance) -> Balance {
            let caller = self.env().caller();
            return self.dot_to_token_output(
                tokens_bought,
                self.env().transferred_balance(),
                caller,
                caller,
            );
        }

        #[ink(message)]
        pub fn swap_token_to_dot_input(&mut self, tokens_sold: Balance) -> Balance {
            let caller = self.env().caller();
            self.token_to_dot_input(tokens_sold, caller, caller)
        }

        #[ink(message)]
        pub fn swap_token_to_dot_output(&mut self, dot_bought: Balance) -> Balance {
            let caller = self.env().caller();
            self.token_to_dot_output(dot_bought, caller, caller)
        }

        /// Public price function for DOT to Token trades with an exact input.
        #[ink(message)]
        pub fn get_dot_to_token_input_price(&self, dot_sold: Balance) -> Balance {
            assert!(dot_sold > 0);
            let token_reserve: Balance = self.token_contract.balance_of(self.env().account_id());
            Self::get_input_price(dot_sold, self.dot_balance(), token_reserve)
        }

        /// Public price function for DOT to Token trades with an exact output.
        #[ink(message)]
        pub fn get_dot_to_token_output_price(&self, tokens_bought: Balance) -> Balance {
            assert!(tokens_bought > 0);
            let token_reserve: Balance = self.token_contract.balance_of(self.env().account_id());
            let dot_sold: Balance =
                Self::get_output_price(tokens_bought, self.dot_balance(), token_reserve);
            return dot_sold;
        }

        /// Public price function for Token to DOT trades with an exact input.
        #[ink(message)]
        pub fn get_token_to_dot_input_price(&self, tokens_sold: Balance) -> Balance {
            assert!(tokens_sold > 0);
            let token_reserve: Balance = self.token_contract.balance_of(self.env().account_id());
            Self::get_input_price(tokens_sold, token_reserve, self.env().balance())
        }

        /// Public price function for Token to DOT trades with an exact output.
        #[ink(message)]
        pub fn get_token_to_dot_output_price(&self, dot_bought: Balance) -> Balance {
            assert!(dot_bought > 0);
            let token_reserve: Balance = self.token_contract.balance_of(self.env().account_id());
            Self::get_output_price(dot_bought, token_reserve, self.env().balance())
        }

        fn dot_to_token_input(
            &mut self,
            dot_sold: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(dot_sold > 0);
            let exchange_account = self.env().account_id();
            let token_reserve: Balance = self.token_contract.balance_of(exchange_account);
            let tokens_bought: Balance =
                Self::get_input_price(dot_sold, self.dot_balance() - dot_sold, token_reserve);
            assert!(self
                .token_contract
                .transfer(recipient, tokens_bought)
                .is_ok());
            self.env().emit_event(TokenSwap {
                buyer,
                sold: dot_sold,
                bought: tokens_bought,
            });
            tokens_bought
        }

        fn dot_to_token_output(
            &mut self,
            tokens_bought: Balance,
            max_dot: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(tokens_bought > 0 && max_dot > 0);
            let token_reserve: Balance = self.token_contract.balance_of(self.env().account_id());
            let dot_sold: Balance =
                Self::get_output_price(tokens_bought, self.dot_balance() - max_dot, token_reserve);
            assert!(dot_sold <= max_dot);
            let dot_refund: Balance = max_dot - dot_sold;
            if dot_refund > 0 {
                assert!(self.env().transfer(buyer, dot_refund).is_ok());
            }
            assert!(self
                .token_contract
                .transfer(recipient, tokens_bought)
                .is_ok());
            self.env().emit_event(TokenSwap {
                buyer,
                sold: dot_sold,
                bought: tokens_bought,
            });
            dot_sold
        }

        fn token_to_dot_input(
            &mut self,
            tokens_sold: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(tokens_sold > 0);
            let exchange_account = self.env().account_id();
            let token_reserve: Balance = self.token_contract.balance_of(exchange_account);
            let dot_bought: Balance =
                Self::get_input_price(tokens_sold, token_reserve, self.env().balance());
            assert!(self.env().transfer(recipient, dot_bought).is_ok());
            assert!(self
                .token_contract
                .transfer_from(buyer, exchange_account, tokens_sold)
                .is_ok());
            self.env().emit_event(TokenSwap {
                buyer,
                sold: tokens_sold,
                bought: dot_bought,
            });
            dot_bought
        }

        fn token_to_dot_output(
            &mut self,
            dot_bought: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(dot_bought > 0);
            let exchange_account = self.env().account_id();
            let token_reserve: Balance = self.token_contract.balance_of(exchange_account);
            let tokens_sold: Balance =
                Self::get_output_price(dot_bought, token_reserve, self.env().balance());
            assert!(self.env().transfer(recipient, dot_bought).is_ok());
            assert!(self
                .token_contract
                .transfer_from(buyer, exchange_account, tokens_sold)
                .is_ok());
            self.env().emit_event(TokenSwap {
                buyer,
                sold: tokens_sold,
                bought: dot_bought,
            });
            tokens_sold
        }
    }

    impl PatraExchange {
        /// Deposit DOT and Tokens (self.token) at current ratio to mint PAT tokens.
        // @return The amount of PAT minted.
        #[ink(message, payable)]
        pub fn add_liquidity(&mut self, from_tokens: Balance) -> Balance {
            let caller = self.env().caller();
            let exchange_account = self.env().account_id();
            let to_tokens = self.env().transferred_balance();
            assert!(from_tokens > 0 && to_tokens > 0);
            // total number of LPT in existence.
            let total_liquidity: Balance = self.lp_token_contract.total_supply();
            if total_liquidity > 0 {
                let from_reserve = self.token_contract.balance_of(exchange_account);
                let to_reserve = self.dot_balance() - to_tokens;
                let token_amount = from_tokens * to_reserve / from_reserve + 1;
                let liquidity_minted = from_tokens * total_liquidity / from_reserve;
                // important
                assert!(to_tokens >= token_amount);
                assert!(self
                    .token_contract
                    .transfer_from(caller, exchange_account, from_tokens)
                    .is_ok());
                assert!(self
                    .lp_token_contract
                    .mint(caller, liquidity_minted)
                    .is_ok());
                self.env().emit_event(AddLiquidity {
                    sender: caller,
                    from_amount: from_tokens,
                    to_amount: token_amount,
                });
                liquidity_minted
            } else {
                assert!(self
                    .token_contract
                    .transfer_from(caller, exchange_account, from_tokens)
                    .is_ok());
                // PAT balance of an account (LP token)
                assert!(self.lp_token_contract.mint(caller, from_tokens).is_ok());
                self.env().emit_event(AddLiquidity {
                    sender: caller,
                    from_amount: from_tokens,
                    to_amount: to_tokens,
                });
                from_tokens
            }
        }

        /// Burn PAT tokens to withdraw DOT and Tokens at current ratio.
        // @param amount Amount of PAT burned.
        // @return The amount of DOT and Tokens withdrawn.
        #[ink(message)]
        pub fn remove_liquidity(&mut self, lp_amount: Balance) -> (Balance, Balance) {
            assert!(lp_amount > 0);
            let total_liquidity = self.lp_token_contract.total_supply();
            assert!(total_liquidity > 0);
            let caller = self.env().caller();
            let exchange_account = self.env().account_id();
            let from_token_reserve = self.token_contract.balance_of(exchange_account);
            let to_token_reserve = self.dot_balance();
            let from_amount = lp_amount * from_token_reserve / total_liquidity;
            let to_amount = lp_amount * to_token_reserve / total_liquidity;
            assert!(self.token_contract.transfer(caller, from_amount).is_ok());
            assert!(self.env().transfer(caller, to_amount).is_ok());
            assert!(self.lp_token_contract.burn(caller, lp_amount).is_ok());
            self.env().emit_event(RemoveLiquidity {
                sender: caller,
                from_amount,
                to_amount,
            });
            (from_amount, to_amount)
        }

        #[ink(message)]
        pub fn estimated_add_liquidity(&self, from_tokens: Balance, to_tokens: Balance) -> Balance {
            let exchange_account = self.env().account_id();
            assert!(from_tokens > 0 && to_tokens > 0);
            let total_liquidity: Balance = self.lp_token_contract.total_supply();
            if total_liquidity > 0 {
                let from_reserve = self.token_contract.balance_of(exchange_account);
                from_tokens * total_liquidity / from_reserve
            } else {
                from_tokens
            }
        }

        #[ink(message)]
        pub fn estimated_remove_liquidity(&self, lp_amount: Balance) -> (Balance, Balance) {
            assert!(lp_amount > 0);
            let total_liquidity = self.lp_token_contract.total_supply();
            assert!(total_liquidity > 0);
            let exchange_account = self.env().account_id();
            let from_token_reserve = self.token_contract.balance_of(exchange_account);
            let to_token_reserve = self.dot_balance();
            let from_amount = lp_amount * from_token_reserve / total_liquidity;
            let to_amount = lp_amount * to_token_reserve / total_liquidity;
            (from_amount, to_amount)
        }

        #[ink(message)]
        pub fn exchange_info(&self) -> ExchangeInfo {
            let caller = self.env().caller();
            let exchange_account = self.env().account_id();
            ExchangeInfo {
                from_symbol: self
                    .token_contract
                    .token_symbol()
                    .unwrap_or(Default::default()),
                from_decimals: self.token_contract.token_decimals().unwrap_or(0),
                to_symbol: "DOT".parse().unwrap(),
                to_decimals: 10,
                from_token_pool: self.token_contract.balance_of(exchange_account),
                to_token_pool: self.dot_balance(),
                lp_token_supply: self.lp_token_contract.total_supply(),
                own_lp_token: self.lp_token_contract.balance_of(caller),
            }
        }

        #[ink(message)]
        pub fn lp_balance_of(&self, user: AccountId) -> Balance {
            self.lp_token_contract.balance_of(user)
        }

        #[ink(message)]
        pub fn lp_token_decimals(&self) -> u8 {
            self.lp_token_contract.token_decimals().unwrap_or(0)
        }

        fn dot_balance(&self) -> Balance {
            self.env().balance().saturating_sub(self.init_deposit_dot)
        }

        /// estimated need to token amount by from tokens
        #[ink(message)]
        pub fn estimated_to_token(&self, from_tokens: Balance) -> Balance {
            let exchange_account = self.env().account_id();
            let from_reserve = self.token_contract.balance_of(exchange_account);
            let to_reserve = self.dot_balance() ;
            from_tokens * to_reserve / from_reserve + 1
        }
    }

    impl PatraExchange {
        // Pricing function for converting between DOT and Tokens.
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

        // Pricing function for converting between DOT and Tokens.
        #[cfg(not(feature = "ink-as-dependency"))]
        fn get_output_price(
            output_amount: Balance,
            input_reserve: Balance,
            output_reserve: Balance,
        ) -> Balance {
            assert!(input_reserve > 0 && output_reserve > 0);
            let numerator: Balance = input_reserve.saturating_mul(output_amount);
            let denominator: Balance = output_reserve.saturating_sub(output_amount);
            numerator / denominator + 1
        }
    }
}
