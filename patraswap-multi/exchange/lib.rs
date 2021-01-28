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
    use ink_storage::Lazy;
    #[cfg(not(feature = "ink-as-dependency"))]
    use lpt::LPT;

    #[ink(storage)]
    pub struct PatraExchange {
        // address of the ERC20 token traded on this contract
        from_token_contract: Lazy<StandardToken>,
        to_token_contract: Lazy<StandardToken>,
        lp_token_contract: Lazy<LPT>,
        from_token: AccountId,
        to_token: AccountId,
    }

    #[ink(event)]
    pub struct FromTokenPurchase {
        #[ink(topic)]
        buyer: AccountId,
        #[ink(topic)]
        sold: Balance,
        #[ink(topic)]
        bought: Balance,
    }

    #[ink(event)]
    pub struct ToTokenPurchase {
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
    pub struct NewExchange {
        #[ink(topic)]
        from_token: AccountId,
        #[ink(topic)]
        to_token: AccountId,
        #[ink(topic)]
        exchange: AccountId,
    }

    impl PatraExchange {
        #[ink(constructor)]
        pub fn new(from_token: AccountId, to_token: AccountId, lpt: AccountId) -> Self {
            let from_token_contract: StandardToken = FromAccountId::from_account_id(from_token);
            let to_token_contract: StandardToken = FromAccountId::from_account_id(to_token);
            let lp_token_contract: LPT = FromAccountId::from_account_id(lpt);
            Self::env().emit_event(NewExchange {
                from_token,
                to_token,
                exchange: Self::env().account_id(),
            });
            Self {
                from_token_contract: Lazy::new(from_token_contract),
                to_token_contract: Lazy::new(to_token_contract),
                lp_token_contract: Lazy::new(lp_token_contract),
                from_token,
                to_token,
            }
        }

        #[ink(message)]
        pub fn swap_from_to_input(&mut self, from_sold: Balance) -> Balance {
            let caller = self.env().caller();
            self.token_from_to_input(from_sold, caller, caller)
        }

        #[ink(message, payable)]
        pub fn swap_to_from_input(&mut self, to_sold: Balance) -> Balance {
            let caller = self.env().caller();
            self.token_to_from_input(to_sold, caller, caller)
        }

        #[ink(message, payable)]
        pub fn swap_to_from_output(&mut self, from_bought: Balance) -> Balance {
            let caller = self.env().caller();
            return self.token_to_from_output(from_bought, caller, caller);
        }

        #[ink(message)]
        pub fn swap_from_to_output(&mut self, to_bought: Balance) -> Balance {
            let caller = self.env().caller();
            self.token_to_dot_output(to_bought, caller, caller)
        }

        /// Public price function for from swap to Token trades with an exact input.
        #[ink(message)]
        pub fn get_from_swap_to_input_price(&self, from_sold: Balance) -> Balance {
            assert!(from_sold > 0);
            let exchange_account = self.env().account_id();
            let from_reserve: Balance = self.from_token_contract.balance_of(exchange_account);
            let to_reserve: Balance = self.to_token_contract.balance_of(exchange_account);
            Self::get_input_price(from_sold, from_reserve, to_reserve)
        }

        /// Public price function for to swap from Token trades with an exact input.
        #[ink(message)]
        pub fn get_to_swap_from_input_price(&self, to_sold: Balance) -> Balance {
            assert!(to_sold > 0);
            let exchange_account = self.env().account_id();
            let from_reserve: Balance = self.from_token_contract.balance_of(exchange_account);
            let to_reserve: Balance = self.to_token_contract.balance_of(exchange_account);
            Self::get_input_price(to_sold, to_reserve, from_reserve)
        }

        /// Public price function for DOT to Token trades with an exact output.
        #[ink(message)]
        pub fn get_from_swap_to_output_price(&self, to_bought: Balance) -> Balance {
            assert!(to_bought > 0);
            let exchange_account = self.env().account_id();
            let from_reserve: Balance = self.from_token_contract.balance_of(exchange_account);
            let to_reserve: Balance = self.to_token_contract.balance_of(exchange_account);
            Self::get_output_price(to_bought, from_reserve, to_reserve)
        }

        /// Public price function for Token to DOT trades with an exact output.
        #[ink(message)]
        pub fn get_to_swap_from_output_price(&self, from_bought: Balance) -> Balance {
            assert!(from_bought > 0);
            let exchange_account = self.env().account_id();
            let from_reserve: Balance = self.from_token_contract.balance_of(exchange_account);
            let to_reserve: Balance = self.to_token_contract.balance_of(exchange_account);
            Self::get_output_price(from_bought, to_reserve, from_reserve)
        }

        fn token_to_from_input(
            &mut self,
            to_sold: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(to_sold > 0);
            let exchange_account = self.env().account_id();
            let from_reserve: Balance = self.from_token_contract.balance_of(exchange_account);
            let to_reserve: Balance = self.to_token_contract.balance_of(exchange_account);
            let from_bought: Balance = Self::get_input_price(to_sold, to_reserve, from_reserve);
            assert!(self
                .to_token_contract
                .transfer_from(buyer, exchange_account, to_sold)
                .is_ok());
            assert!(self
                .from_token_contract
                .transfer_from(exchange_account, recipient, from_bought)
                .is_ok());
            self.env().emit_event(FromTokenPurchase {
                buyer,
                sold: to_sold,
                bought: from_bought,
            });
            from_bought
        }

        fn token_to_from_output(
            &mut self,
            from_bought: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(from_bought > 0);
            let exchange_account = self.env().account_id();
            let from_reserve: Balance = self.from_token_contract.balance_of(exchange_account);
            let to_reserve: Balance = self.to_token_contract.balance_of(exchange_account);
            let to_sold: Balance = Self::get_output_price(from_bought, to_reserve, from_reserve);
            assert!(self
                .to_token_contract
                .transfer_from(buyer, exchange_account, to_sold)
                .is_ok());
            assert!(self
                .from_token_contract
                .transfer_from(exchange_account, recipient, from_bought)
                .is_ok());
            self.env().emit_event(FromTokenPurchase {
                buyer,
                sold: to_sold,
                bought: from_bought,
            });
            to_sold
        }

        fn token_from_to_input(
            &mut self,
            from_sold: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(from_sold > 0);
            let exchange_account = self.env().account_id();
            let from_reserve: Balance = self.from_token_contract.balance_of(exchange_account);
            let to_reserve: Balance = self.to_token_contract.balance_of(exchange_account);
            let to_bought: Balance = Self::get_input_price(from_sold, from_reserve, to_reserve);
            assert!(self
                .from_token_contract
                .transfer_from(buyer, exchange_account, from_sold)
                .is_ok());
            assert!(self
                .to_token_contract
                .transfer_from(exchange_account, recipient, to_bought)
                .is_ok());
            self.env().emit_event(ToTokenPurchase {
                buyer,
                sold: from_sold,
                bought: to_bought,
            });
            to_bought
        }

        fn token_to_dot_output(
            &mut self,
            to_bought: Balance,
            buyer: AccountId,
            recipient: AccountId,
        ) -> Balance {
            assert!(to_bought > 0);
            let exchange_account = self.env().account_id();
            let from_reserve: Balance = self.from_token_contract.balance_of(exchange_account);
            let to_reserve: Balance = self.to_token_contract.balance_of(exchange_account);
            let from_sold: Balance = Self::get_output_price(to_bought, from_reserve, to_reserve);
            assert!(self
                .from_token_contract
                .transfer_from(buyer, exchange_account, from_sold)
                .is_ok());
            assert!(self
                .to_token_contract
                .transfer_from(exchange_account, recipient, to_bought)
                .is_ok());
            self.env().emit_event(ToTokenPurchase {
                buyer,
                sold: from_sold,
                bought: to_bought,
            });
            from_sold
        }
    }

    impl PatraExchange {
        /// Deposit DOT and Tokens (self.token) at current ratio to mint PAT tokens.
        // @return The amount of PAT minted.
        // 等比例添加
        #[ink(message)]
        pub fn add_liquidity(&mut self, from_tokens: Balance, to_tokens: Balance) -> Balance {
            let caller = self.env().caller();
            let exchange_account = self.env().account_id();
            assert!(from_tokens > 0 && to_tokens > 0);
            // total number of LPT in existence.
            let total_liquidity: Balance = self.lp_token_contract.total_supply();
            if total_liquidity > 0 {
                let from_reserve = self.from_token_contract.balance_of(exchange_account);
                let to_reserve = self.to_token_contract.balance_of(exchange_account);
                let token_amount = from_tokens * to_reserve / from_reserve + 1;
                let liquidity_minted = from_tokens * total_liquidity / from_reserve;
                // important
                assert!(to_tokens >= token_amount);
                assert!(self
                    .from_token_contract
                    .transfer_from(caller, exchange_account, from_tokens)
                    .is_ok());
                assert!(self
                    .to_token_contract
                    .transfer_from(caller, exchange_account, token_amount)
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
                    .from_token_contract
                    .transfer_from(caller, exchange_account, from_tokens)
                    .is_ok());
                assert!(self
                    .to_token_contract
                    .transfer_from(caller, exchange_account, to_tokens)
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
            let exchange_account = self.env().account_id();
            let from_token_reserve = self.from_token_contract.balance_of(exchange_account);
            let to_token_reserve = self.to_token_contract.balance_of(exchange_account);
            let from_amount = lp_amount * from_token_reserve / total_liquidity;
            let to_amount = lp_amount * to_token_reserve / total_liquidity;
            let caller = self.env().caller();
            assert!(self
                .from_token_contract
                .transfer(caller, from_amount)
                .is_ok());
            assert!(self.to_token_contract.transfer(caller, to_amount).is_ok());
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
                let from_reserve = self.from_token_contract.balance_of(exchange_account);
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
            let from_token_reserve = self.from_token_contract.balance_of(exchange_account);
            let to_token_reserve = self.to_token_contract.balance_of(exchange_account);
            let from_amount = lp_amount * from_token_reserve / total_liquidity;
            let to_amount = lp_amount * to_token_reserve / total_liquidity;
            (from_amount, to_amount)
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
