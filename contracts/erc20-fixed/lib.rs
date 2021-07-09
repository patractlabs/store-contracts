#![cfg_attr(not(feature = "std"), no_std)]

#[metis_lang::contract]
mod erc20 {
    use ink_prelude::string::String;
    use metis_erc20::{self as erc20, Result};
    use metis_lang::{
        import,
        metis,
    };

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    #[metis(erc20)]
    pub struct Transfer {
        #[ink(topic)]
        pub from: Option<AccountId>,
        #[ink(topic)]
        pub to: Option<AccountId>,
        #[ink(topic)]
        pub value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    #[metis(erc20)]
    pub struct Approval {
        #[ink(topic)]
        pub owner: AccountId,
        #[ink(topic)]
        pub spender: AccountId,
        #[ink(topic)]
        pub value: Balance,
    }

    #[ink(storage)]
    #[import(erc20)]
    pub struct Erc20 {
        /// Metis-Erc20 Data
        erc20: erc20::Data<Erc20>,
    }

    impl erc20::Impl<Erc20> for Erc20 {
        fn _before_token_transfer(
            &mut self,
            _from: &AccountId,
            _to: &AccountId,
            _amount: Balance,
        ) -> Result<()>
        {
            Ok(())
        }
    }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(
            initial_supply: Balance,
            name: String,
            symbol: String,
            decimals: Option<u8>,
        ) -> Self {
            let mut instance = Self {
                erc20: erc20::Data::new(),
            };

            erc20::Impl::init(
                &mut instance,
                name,
                symbol,
                decimals.unwrap_or(18),
                initial_supply,
            );

            let caller = Self::env().caller();
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });

            instance
        }

        /// Returns the token name.
        #[ink(message)]
        pub fn token_name(&self) -> String {
            erc20::Impl::name(self)
        }

        /// Returns the token symbol.
        #[ink(message)]
        pub fn token_symbol(&self) -> String {
            erc20::Impl::symbol(self)
        }

        /// Returns the token decimals.
        #[ink(message)]
        pub fn token_decimals(&self) -> u8 {
            erc20::Impl::decimals(self)
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            erc20::Impl::total_supply(self)
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            erc20::Impl::balance_of(self, &owner)
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            erc20::Impl::transfer(self, &to, value)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set `0`.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            erc20::Impl::allowance(self, &owner, &spender)
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the the account balance of `from`.
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            erc20::Impl::transfer_from(self, &from, &to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            erc20::Impl::approve(self, &spender, value)
        }
    }
}
