#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod patrapixel {
    use ink_prelude::{string::String, vec, vec::Vec};
    use ink_storage::collections::HashMap as StorageHashMap;

    pub type TokenId = u32;

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        TokenExists,
        TokenNotFound,
        CannotInsert,
    }

    #[ink(event)]
    pub struct Minted {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        token_id: TokenId,
    }

    #[ink(storage)]
    pub struct Patrapixel {
        name: String,
        symbol: String,
        total_supply: u32,
        token_owner: StorageHashMap<TokenId, AccountId>,
        list_of_owner_tokens: StorageHashMap<AccountId, Vec<TokenId>>,
        referenced_metadata: StorageHashMap<TokenId, String>,
    }

    impl Patrapixel {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                name: "PatraPixel".parse().unwrap(),
                symbol: "PPX".parse().unwrap(),
                total_supply: 0,
                token_owner: StorageHashMap::new(),
                list_of_owner_tokens: StorageHashMap::new(),
                referenced_metadata: StorageHashMap::new(),
            }
        }

        /// Get and returns the address currently marked as the owner of tokenID.
        #[ink(message)]
        pub fn owner_of(&self, token_id: TokenId) -> Option<AccountId> {
            self.token_owner.get(&token_id).cloned()
        }

        /// Get and return the total supply of token held by this contract.
        #[ink(message)]
        pub fn total_supply(&self) -> u32 {
            self.total_supply
        }

        /// Get and return the balance of token held by _owner.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Option<Vec<TokenId>> {
            self.list_of_owner_tokens.get(&owner).cloned()
        }

        /// Get and returns a metadata of tokenId
        #[ink(message)]
        pub fn token_metadata(&self, token_id: TokenId) -> Result<String, Error> {
            match self.referenced_metadata.get(&token_id).cloned() {
                Some(v) => Ok(v),
                None => Err(Error::TokenNotFound),
            }
        }

        /// Mint a new token with metadata
        #[ink(message)]
        pub fn mint_with_metadata(&mut self, metadata: String) -> Result<(), Error> {
            let owner = self.env().caller();
            self.total_supply += 1;
            let token_id = self.total_supply;
            self.set_token_owner(token_id, owner)?;
            self.add_token_to_owners_list(owner, token_id);
            self.insert_token_metadata(token_id, metadata)?;
            self.env().emit_event(Minted { owner, token_id });
            Ok(())
        }

        fn set_token_owner(&mut self, token_id: TokenId, owner: AccountId) -> Result<(), Error> {
            match self.token_owner.insert(token_id, owner) {
                Some(_) => Err(Error::CannotInsert),
                None => Ok(()),
            }
        }

        fn add_token_to_owners_list(&mut self, owner: AccountId, token_id: TokenId) {
            if self.list_of_owner_tokens.contains_key(&owner) {
                let tokens = self.list_of_owner_tokens.get_mut(&owner).unwrap();
                tokens.push(token_id);
            } else {
                self.list_of_owner_tokens.insert(owner, vec![token_id]);
            }
        }

        fn insert_token_metadata(
            &mut self,
            token_id: TokenId,
            metadata: String,
        ) -> Result<(), Error> {
            match self.referenced_metadata.insert(token_id, metadata) {
                Some(_) => Err(Error::CannotInsert),
                None => Ok(()),
            }
        }
    }
}
