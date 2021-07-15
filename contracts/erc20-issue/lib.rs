#![cfg_attr(not(feature = "std"), no_std)]

pub use self::erc20::Erc20;

#[metis_lang::contract]
mod erc20 {
    use ink_prelude::string::String;
    use metis_erc20::{self as erc20, Error as MErc20Error};
    use metis_ownable::{self as ownable};
    use metis_pausable::{self as pausable};
    use metis_lang::{
        import,
        metis,
    };

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_lang as ink;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{collections::HashMap as StorageHashMap};

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        InsufficientSupply,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
        BlacklistedUser,
        InvalidAmount,
        OnlyOwnerAccess,
        InvalidNewOwner,
        NotBlacklistedUser,
    }

    /// Convert Metic-Erc20-Error to Error
    impl From<MErc20Error> for Error {
        fn from(e: MErc20Error) -> Self {
            match e {
                MErc20Error::AccountIsZero => {
                    panic!("Zero-Addressed Account Disallowed.")
                }
                MErc20Error::InsufficientAllowance => {
                    Self::InsufficientAllowance
                }
                MErc20Error::InsufficientBalance => {
                    Self::InsufficientBalance
                }
            }
        }
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink::trait_definition]
    pub trait BlackList {
        /// Whether the user is blacklisted.
        #[ink(message)]
        fn get_blacklist_status(&self, maker: AccountId) -> bool;

        /// Add illegal user to blacklist.
        #[ink(message)]
        fn add_blacklist(&mut self, evil_user: AccountId) -> Result<()>;

        /// Remove the user from blacklist.
        #[ink(message)]
        fn remove_blacklist(&mut self, cleared_user: AccountId) -> Result<()>;

        /// Destroy blacklisted user funds from total supply.
        #[ink(message)]
        fn destroy_blackfunds(&mut self, blacklisted_user: AccountId) -> Result<()>;
    }

    #[ink(storage)]
    #[import(erc20, ownable, pausable)]
    pub struct Erc20 {
        erc20: erc20::Data<Erc20>,
        ownable: ownable::Data<Erc20>,
        pausable: pausable::Data,

        blacklisted: StorageHashMap<AccountId, bool>,
    }

        /// Event emitted when a token transfer occurs.
    #[ink(event)]
    #[metis(erc20)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    #[metis(erc20)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// Event emitted when Owner AccountId Transferred
    #[ink(event)]
    #[metis(ownable)]
    pub struct OwnershipTransferred {
        /// previous owner account id
        #[ink(topic)]
        previous_owner: Option<AccountId>,
        /// new owner account id
        #[ink(topic)]
        new_owner: Option<AccountId>,
    }

    /// Event emitted when Pause
    #[ink(event)]
    #[metis(pausable)]
    pub struct Paused {
        /// paused caller
        #[ink(topic)]
        account: AccountId,
    }

    /// Event emitted when unPause
    #[ink(event)]
    #[metis(pausable)]
    pub struct Unpaused {
        /// unpaused caller
        #[ink(topic)]
        account: AccountId,
    }

    #[ink(event)]
    pub struct DestroyedBlackFunds {
        #[ink(topic)]
        blacklisted_user: AccountId,
        #[ink(topic)]
        balance: Balance,
    }

    #[ink(event)]
    pub struct AddedBlackList {
        #[ink(topic)]
        user: AccountId,
    }

    #[ink(event)]
    pub struct RemovedBlackList {
        #[ink(topic)]
        user: AccountId,
    }

    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    #[cfg(not(feature = "ink-as-dependency"))]
    impl erc20::Impl<Erc20> for Erc20 {}

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
                ownable: ownable::Data::new(),
                pausable: pausable::Data::new(),
                blacklisted: Default::default(),
            };

            erc20::Impl::init(
                &mut instance,
                name,
                symbol,
                decimals.unwrap_or(18),
                initial_supply,
            );

            ownable::Impl::init(&mut instance);

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
            erc20::Impl::transfer(self, &to, value).map_err(|e| e.into())
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
            erc20::Impl::transfer_from(self, &from, &to, value).map_err(|e| e.into())
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            erc20::Impl::approve(self, &spender, value).map_err(|e| e.into())
        }

        /// Mint a new amount of tokens
        /// these tokens are deposited into the owner address
        #[ink(message)]
        pub fn mint(&mut self, user: AccountId, amount: Balance) -> Result<()> {
            ownable::Impl::ensure_caller_is_owner(self);

            erc20::Impl::_mint(self, &user, amount).map_err(|e| e.into())
        }

        /// Burn tokens.
        /// These tokens are withdrawn from the owner address
        /// if the balance must be enough to cover the redeem
        /// or the call will fail.
        #[ink(message)]
        pub fn burn(&mut self, user: AccountId, amount: Balance) -> Result<()> {
            ownable::Impl::ensure_caller_is_owner(self);
            erc20::Impl::_burn(self, &user, amount).map_err(|e| e.into())
        }

        /// Return the owner of contract
        #[ink(message)]
        pub fn owner(&self) -> Option<AccountId> {
            *ownable::Impl::owner(self)
        }

        /// Transfer the ownership of contract
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) {
            ownable::Impl::transfer_ownership(self, &new_owner)
        }

        #[ink(message)]
        pub fn paused(&self) -> bool {
            pausable::Impl::paused(self)
        }

        #[ink(message)]
        pub fn pause(&mut self) {
            ownable::Impl::ensure_caller_is_owner(self);
            pausable::Impl::_pause(self)
        }

        #[ink(message)]
        pub fn unpause(&mut self) {
            ownable::Impl::ensure_caller_is_owner(self);
            pausable::Impl::_unpause(self)
        }
    }

    impl BlackList for Erc20 {
        /// Whether the user is blacklisted.
        #[ink(message)]
        fn get_blacklist_status(&self, maker: AccountId) -> bool {
            self.blacklisted.get(&maker).copied().unwrap_or(false)
        }

        /// Add illegal user to blacklist.
        #[ink(message)]
        fn add_blacklist(&mut self, evil_user: AccountId) -> Result<()> {
            ownable::Impl::ensure_caller_is_owner(self);

            self.blacklisted.insert(evil_user, true);
            Ok(())
        }

        /// Remove the user from blacklist.
        #[ink(message)]
        fn remove_blacklist(&mut self, cleared_user: AccountId) -> Result<()> {
            ownable::Impl::ensure_caller_is_owner(self);

            self.blacklisted.take(&cleared_user);
            Ok(())
        }

        /// Destroy blacklisted user funds from total supply.
        #[ink(message)]
        fn destroy_blackfunds(&mut self, blacklisted_user: AccountId) -> Result<()> {
            ownable::Impl::ensure_caller_is_owner(self);

            if !self.get_blacklist_status(blacklisted_user) {
                return Err(Error::NotBlacklistedUser);
            }

            erc20::Impl::_burn(
                self,
                &blacklisted_user,
                self.balance_of(blacklisted_user)
            ).map_err(|e| e.into())
        }
    }
}
