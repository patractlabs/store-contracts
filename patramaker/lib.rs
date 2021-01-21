#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod patramaker {
    use ink_lang as ink;
    use ink_storage::{
        collections::HashMap as StorageMap,
        traits::{PackedLayout, SpreadLayout},
    };

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        OnlyOwnerAccess,
        InvalidNewOwner,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    pub type CdpId = u32;
    pub type USD = u32;

    #[derive(
        Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct CDP {
        pub owner: AccountId,
        // 1 DAI = 1 USD
        pub dai: Balance,
        pub dot: Balance,
        pub collateral_ratio: u32,
        pub valid: bool,
    }

    #[ink(storage)]
    pub struct PatraMaker {
        cdps: StorageMap<CdpId, CDP>,
        cdp_count: u32,
        total_collateral_dot: Balance,
        total_issue_dai: Balance,
        min_collateral_ratio: u32,
        min_liquidation_ratio: u32,
        liquidater_reward_ratio: u32,
        dot_price: USD,
        owner: AccountId,
    }

    /// The Ownable contract has an owner address, and provides basic authorization control
    /// functions, this simplifies the implementation of "user permissions".
    #[ink::trait_definition]
    pub trait Ownable {
        /// Contract owner.
        #[ink(message)]
        fn owner(&self) -> AccountId;

        /// transfer contract ownership to new owner.
        #[ink(message)]
        fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()>;
    }

    impl Ownable for PatraMaker {
        /// Contract owner.
        #[ink(message)]
        fn owner(&self) -> AccountId {
            self.owner
        }

        /// transfer contract ownership to new owner.
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

    impl PatraMaker {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self {
                cdps: StorageMap::new(),
                cdp_count: 0,
                total_collateral_dot: 0,
                total_issue_dai: 0,
                min_collateral_ratio: 150,
                min_liquidation_ratio: 110,
                liquidater_reward_ratio: 5,
                dot_price: 16,
                owner: caller,
            }
        }

        /// Adjust Min Collateral Ratio only admin
        #[ink(message)]
        pub fn adjust_mcr(&mut self, mcr: u32) -> Result<()> {
            self.only_owner()?;
            self.min_collateral_ratio = mcr;
            Ok(())
        }

        // Adjust Min Liquidation Ratio only admin
        #[ink(message)]
        pub fn adjust_mlr(&mut self, mlr: u32) -> Result<()> {
            self.only_owner()?;
            self.min_liquidation_ratio = mlr;
            Ok(())
        }

        /// Adjust Liquidater Reward Ratio only admin
        #[ink(message)]
        pub fn adjust_lrr(&mut self, lrr: u32) -> Result<()> {
            self.only_owner()?;
            self.liquidater_reward_ratio = lrr;
            Ok(())
        }

        /// Adjust dot price only admin
        #[ink(message)]
        pub fn adjust_dot_price(&mut self, price: USD) -> Result<()> {
            self.only_owner()?;
            self.dot_price = price;
            Ok(())
        }

        #[ink(message)]
        pub fn system_params(&self) -> (u32, u32, u32, u32) {
            (
                self.min_collateral_ratio,
                self.min_liquidation_ratio,
                self.liquidater_reward_ratio,
                self.dot_price,
            )
        }

        #[ink(message)]
        pub fn total_collateral_dot(&self) -> Balance {
            self.total_collateral_dot
        }

        #[ink(message)]
        pub fn total_issue_dai(&self) -> Balance {
            self.total_issue_dai
        }

        #[ink(message)]
        pub fn query_cdp(&self, cdp_id: CdpId) -> Option<CDP> {
            self.cdps.get(&cdp_id).cloned().and_then(|cdp| Some(cdp))
        }

        #[ink(message, payable)]
        pub fn issue_dai(&mut self, cr: u32) -> (CdpId, Balance) {
            assert!(cr >= self.min_collateral_ratio);
            let caller = self.env().caller();
            let collateral = self.env().transferred_balance();
            let dai = collateral * self.dot_price as u128 * 100 / cr as u128;
            let cdp = CDP {
                owner: caller,
                dai,
                dot: collateral,
                collateral_ratio: cr,
                valid: true,
            };
            self.cdp_count += 1;
            self.cdps.insert(self.cdp_count, cdp);
            self.total_collateral_dot += collateral;
            self.total_issue_dai += dai;

            (self.cdp_count, dai)
        }

        #[ink(message, payable)]
        pub fn add_collateral(&mut self, cdp_id: CdpId) {
            assert!(self.cdps.contains_key(&cdp_id));
            let caller = self.env().caller();
            let collateral = self.env().transferred_balance();
            let cdp = self.cdps.get_mut(&cdp_id).unwrap();
            assert!(cdp.owner == caller);
            let cr =
                (collateral + self.total_collateral_dot as u128) * self.dot_price as u128 * 100
                    / self.total_issue_dai;
            assert!(cr >= self.min_collateral_ratio.into());
            assert!(cdp.valid);
            cdp.collateral_ratio = cr as u32;
            cdp.dot += collateral;
            self.total_collateral_dot += collateral;
        }

        #[ink(message, payable)]
        pub fn minus_collateral(&mut self, cdp_id: CdpId) {
            assert!(self.cdps.contains_key(&cdp_id));
            let caller = self.env().caller();
            let collateral = self.env().transferred_balance();
            let cdp = self.cdps.get_mut(&cdp_id).unwrap();
            assert!(cdp.owner == caller);
            let cr = (self.total_collateral_dot - collateral) * self.dot_price as u128 * 100
                / self.total_issue_dai;
            assert!(cr >= self.min_collateral_ratio.into());
            assert!(cdp.valid);
            cdp.collateral_ratio = cr as u32;
            cdp.dot -= collateral;
            self.total_collateral_dot -= collateral;
        }

        #[ink(message)]
        pub fn withdraw_dot(&mut self, cdp_id: CdpId, amount: Balance) -> Balance {
            assert!(self.cdps.contains_key(&cdp_id));
            let caller = self.env().caller();
            let cdp = self.cdps.get_mut(&cdp_id).unwrap();
            assert!(cdp.owner == caller);
            assert!(cdp.collateral_ratio >= self.min_collateral_ratio);
            assert!(cdp.valid);
            assert!(amount <= cdp.dai);

            let dot = cdp.dot * amount / cdp.dai;
            cdp.dot -= dot;
            cdp.dai -= amount;
            self.env().transfer(caller, dot).unwrap();
            self.total_collateral_dot -= dot;
            self.total_issue_dai -= amount;
            dot
        }

        #[ink(message)]
        pub fn liquidate_collateral(&mut self, cdp_id: CdpId, amount: Balance) {
            assert!(self.cdps.contains_key(&cdp_id));
            let cdp = self.cdps.get_mut(&cdp_id).unwrap();
            assert!(cdp.collateral_ratio <= self.min_collateral_ratio);
            assert!(cdp.valid);
            cdp.valid = false;
            let owner = cdp.owner;
            let dot = amount / self.dot_price as u128;
            self.env().transfer(owner, dot).unwrap();
            let caller = self.env().caller();
            let keeper_reward =
                amount * self.liquidater_reward_ratio as u128 / (100 * self.dot_price as u128);
            self.env().transfer(caller, keeper_reward).unwrap();
            self.total_issue_dai -= amount;
            self.total_collateral_dot -= dot;
        }

        fn only_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::OnlyOwnerAccess);
            }
            Ok(())
        }
    }
}
