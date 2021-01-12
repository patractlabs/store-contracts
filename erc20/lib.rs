#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_prelude::vec::Vec;

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_lang as ink;

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{collections::HashMap as StorageHashMap, lazy::Lazy};

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

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Trait implemented by all ERC-20 respecting smart contracts.
    #[ink::trait_definition]
    pub trait Erc20 {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        fn new(initial_supply: Balance, name: Vec<u8>, symbol: Vec<u8>) -> Self;

        /// Returns the total token supply.
        #[ink(message)]
        fn total_supply(&self) -> Balance;

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance;

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()>;

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()>;

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()>;
    }

    /// The Ownable contract has an owner address, and provides basic authorization control
    //  functions, this simplifies the implementation of "user permissions".
    #[ink::trait_definition]
    pub trait Ownable {
        #[ink(message)]
        fn owner(&self) -> AccountId;

        #[ink(message)]
        fn only_owner(&self) -> Result<()>;

        #[ink(message)]
        fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()>;
    }

    /// Base contract which allows children to implement an emergency stop mechanism.
    #[ink::trait_definition]
    pub trait Pausable {
        #[ink(message)]
        fn pause(&mut self) -> Result<()>;

        #[ink(message)]
        fn unpause(&mut self) -> Result<()>;

        #[ink(message)]
        fn pause_state(&self) -> bool;
    }

    #[ink::trait_definition]
    pub trait BlackList {
        #[ink(message)]
        fn get_blacklist_status(&self, maker: AccountId) -> bool;

        #[ink(message)]
        fn add_blacklist(&mut self, evil_user: AccountId) -> Result<()>;

        #[ink(message)]
        fn remove_blacklist(&mut self, cleared_user: AccountId) -> Result<()>;

        #[ink(message)]
        fn destroy_blackfunds(&mut self, blacklisted_user: AccountId) -> Result<()>;
    }

    /// Basic version of StandardToken, with no allowances.
    #[ink(storage)]
    pub struct StandardToken {
        /// Token Name
        name: Vec<u8>,
        /// Token symbol
        symbol: Vec<u8>,
        /// Token decimals
        decimals: u128,
        /// Total token supply.
        total_supply: Lazy<Balance>,
        /// Mapping from owner to number of owned token.
        balances: StorageHashMap<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,
        /// Implement an emergency stop mechanism.
        pause: bool,
        /// The contract owner, provides basic authorization control
        /// functions, this simplifies the implementation of "user permissions".
        owner: AccountId,

        blacklisted: StorageHashMap<AccountId, bool>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    #[ink(event)]
    pub struct Pause {}

    #[ink(event)]
    pub struct Unpause {}

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
    pub struct Issue {
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct Redeem {
        #[ink(topic)]
        amount: Balance,
    }

    impl Erc20 for StandardToken {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        fn new(initial_supply: Balance, name: Vec<u8>, symbol: Vec<u8>) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, initial_supply);
            let instance = Self {
                name,
                symbol,
                decimals: 0,
                total_supply: Lazy::new(initial_supply),
                balances,
                allowances: StorageHashMap::new(),
                pause: false,
                owner: caller,
                blacklisted: StorageHashMap::new(),
            };
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });
            instance
        }

        /// Returns the total token supply.
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(&owner).copied().unwrap_or(0)
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
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            if self.get_blacklist_status(from) {
                return Err(Error::BlacklistedUser);
            }

            self.transfer_from_to(from, to, value)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set `0`.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
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
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            if self.get_blacklist_status(from) {
                return Err(Error::BlacklistedUser);
            }

            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }
            self.transfer_from_to(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }
    }

    impl Ownable for StandardToken {
        #[ink(message)]
        fn owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        fn only_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::OnlyOwnerAccess);
            }
            Ok(())
        }

        #[ink(message)]
        fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            self.only_owner()?;

            if new_owner != AccountId::from([0x00; 32]) {
                self.owner = new_owner;
            } else {
                return Err(Error::InvalidNewOwner);
            }
            Ok(())
        }
    }

    impl Pausable for StandardToken {
        #[ink(message)]
        fn pause(&mut self) -> Result<()> {
            self.only_owner()?;

            if !self.pause {
                self.pause = true;
                self.env().emit_event(Pause {})
            }
            Ok(())
        }

        #[ink(message)]
        fn unpause(&mut self) -> Result<()> {
            self.only_owner()?;
            if self.pause {
                self.pause = false;
                self.env().emit_event(Unpause {})
            }
            Ok(())
        }

        #[ink(message)]
        fn pause_state(&self) -> bool {
            self.pause
        }
    }

    impl BlackList for StandardToken {
        #[ink(message)]
        fn get_blacklist_status(&self, maker: AccountId) -> bool {
            self.blacklisted.get(&maker).copied().unwrap_or(false)
        }

        #[ink(message)]
        fn add_blacklist(&mut self, evil_user: AccountId) -> Result<()> {
            self.only_owner()?;
            self.blacklisted.insert(evil_user, true);
            Ok(())
        }

        #[ink(message)]
        fn remove_blacklist(&mut self, cleared_user: AccountId) -> Result<()> {
            self.only_owner()?;
            self.blacklisted.take(&cleared_user);
            Ok(())
        }

        #[ink(message)]
        fn destroy_blackfunds(&mut self, blacklisted_user: AccountId) -> Result<()> {
            self.only_owner()?;
            if !self.get_blacklist_status(blacklisted_user) {
                return Err(Error::NotBlacklistedUser);
            }
            let dirty_funds = self.balance_of(blacklisted_user);
            self.balances.insert(blacklisted_user, 0);
            *self.total_supply -= dirty_funds;
            self.env().emit_event(DestroyedBlackFunds {
                blacklisted_user,
                balance: dirty_funds,
            });
            Ok(())
        }
    }

    impl StandardToken {
        /// Issue a new amount of tokens
        /// these tokens are deposited into the owner address
        #[ink(message)]
        pub fn issue(&mut self, amount: Balance) -> Result<()> {
            self.only_owner()?;
            if amount <= 0 {
                return Err(Error::InvalidAmount);
            }

            let owner_balance = self.balance_of(self.owner);
            self.balances.insert(self.owner, owner_balance + amount);
            *self.total_supply += amount;
            self.env().emit_event(Issue { amount });
            Ok(())
        }

        /// Redeem tokens.
        /// These tokens are withdrawn from the owner address
        /// if the balance must be enough to cover the redeem
        /// or the call will fail.
        #[ink(message)]
        pub fn redeem(&mut self, amount: Balance) -> Result<()> {
            self.only_owner()?;
            if *self.total_supply < amount {
                return Err(Error::InsufficientSupply);
            }
            let owner_balance = self.balance_of(self.owner);
            if owner_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(self.owner, owner_balance - amount);
            *self.total_supply -= amount;
            self.env().emit_event(Redeem { amount });
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }
            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });
            Ok(())
        }
    }
}
