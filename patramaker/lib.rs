#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod patramaker {
    use dai::DAI;
    use ink_env::call::FromAccountId;
    use ink_lang as ink;
    use ink_storage::{
        collections::HashMap as StorageMap,
        traits::{PackedLayout, SpreadLayout},
        Lazy,
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

    #[ink(event)]
    pub struct IssueDAI {
        #[ink(topic)]
        cdp_id: CdpId,
        #[ink(topic)]
        collateral: Balance,
        #[ink(topic)]
        dai: Balance,
    }

    #[ink(event)]
    pub struct AddCollateral {
        #[ink(topic)]
        cdp_id: CdpId,
        #[ink(topic)]
        add_collateral: Balance,
        #[ink(topic)]
        collateral_ratio: u32,
    }

    #[ink(event)]
    pub struct MinusCollateral {
        #[ink(topic)]
        cdp_id: CdpId,
        #[ink(topic)]
        minus_collateral: Balance,
        #[ink(topic)]
        collateral_ratio: u32,
    }

    #[ink(event)]
    pub struct Withdraw {
        #[ink(topic)]
        cdp_id: CdpId,
        #[ink(topic)]
        collateral: Balance,
        #[ink(topic)]
        dai: Balance,
    }

    #[ink(event)]
    pub struct Liquidate {
        #[ink(topic)]
        cdp_id: CdpId,
        #[ink(topic)]
        collateral: Balance,
        #[ink(topic)]
        keeper_reward: Balance,
    }

    #[derive(
        Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct CDP {
        pub issuer: AccountId,
        pub collateral_dot: Balance,
        // 1 DAI = 1 USD
        pub issue_dai: Balance,
        pub collateral_ratio: u32,
        pub valid: bool,
    }

    #[ink(storage)]
    pub struct PatraMaker {
        dai_token: Lazy<DAI>,
        cdps: StorageMap<CdpId, CDP>,
        cdp_count: u32,
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

            if new_owner != Default::default() {
                self.owner = new_owner;
            } else {
                return Err(Error::InvalidNewOwner);
            }
            Ok(())
        }
    }

    impl PatraMaker {
        #[ink(constructor)]
        pub fn new(dai_contract: AccountId) -> Self {
            assert_ne!(dai_contract, Default::default());
            let caller = Self::env().caller();
            let dai_token: DAI = FromAccountId::from_account_id(dai_contract);
            // let salt = 0_u32.to_le_bytes();
            // let total_balance = Self::env().balance();
            // let dai_token = DAI::new()
            //     .endowment(total_balance / 2)
            //     .code_hash(dai_contract)
            //     .salt_bytes(salt)
            //     .instantiate()
            //     .expect("failed at instantiating the dai token contract");
            Self {
                dai_token: Lazy::new(dai_token),
                cdps: StorageMap::new(),
                cdp_count: 0,
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

        /// System params
        #[ink(message)]
        pub fn system_params(&self) -> (u32, u32, u32, u32) {
            (
                self.min_collateral_ratio,
                self.min_liquidation_ratio,
                self.liquidater_reward_ratio,
                self.dot_price,
            )
        }

        /// Query cdp by id
        #[ink(message)]
        pub fn query_cdp(&self, cdp_id: CdpId) -> Option<CDP> {
            self.cdps.get(&cdp_id).cloned().and_then(|cdp| Some(cdp))
        }

        /// Stake collateral and issue dai
        #[ink(message, payable)]
        pub fn issue_dai(&mut self, cr: u32) -> (CdpId, Balance) {
            assert!(cr >= self.min_collateral_ratio);
            let caller = self.env().caller();
            let collateral = self.env().transferred_balance();
            let dai = collateral * self.dot_price as u128 * 100 / cr as u128;
            let cdp = CDP {
                issuer: caller,
                collateral_dot: collateral,
                issue_dai: dai,
                collateral_ratio: cr,
                valid: true,
            };
            self.cdp_count += 1;
            self.cdps.insert(self.cdp_count, cdp);
            self.dai_token.mint(caller, dai).unwrap();
            self.env().emit_event(IssueDAI {
                cdp_id: self.cdp_count,
                collateral,
                dai,
            });
            (self.cdp_count, dai)
        }

        /// Only issuer can add collateral and update collateral ratio
        #[ink(message, payable)]
        pub fn add_collateral(&mut self, cdp_id: CdpId) {
            assert!(self.cdps.contains_key(&cdp_id));
            let caller = self.env().caller();
            let collateral = self.env().transferred_balance();
            let cdp = self.cdps.get_mut(&cdp_id).unwrap();
            assert!(cdp.valid);
            assert!(cdp.issuer == caller);
            let cr = (collateral + cdp.collateral_dot as u128) * self.dot_price as u128 * 100
                / cdp.issue_dai;
            assert!(cr >= self.min_collateral_ratio.into());
            cdp.collateral_ratio = cr as u32;
            cdp.collateral_dot += collateral;
            self.env().emit_event(AddCollateral {
                cdp_id,
                add_collateral: collateral,
                collateral_ratio: cr as u32,
            });
        }

        /// Only issuer can minus collateral and update collateral ratio
        #[ink(message)]
        pub fn minus_collateral(&mut self, cdp_id: CdpId, collateral: Balance) {
            assert!(self.cdps.contains_key(&cdp_id));
            let caller = self.env().caller();
            let cdp = self.cdps.get_mut(&cdp_id).unwrap();
            assert!(cdp.valid);
            assert!(cdp.issuer == caller);
            let cr =
                (cdp.collateral_dot - collateral) * self.dot_price as u128 * 100 / cdp.issue_dai;
            assert!(cr >= self.min_collateral_ratio.into());
            cdp.collateral_ratio = cr as u32;
            cdp.collateral_dot -= collateral;
            self.env().transfer(caller, collateral).unwrap();
            self.env().emit_event(MinusCollateral {
                cdp_id,
                minus_collateral: collateral,
                collateral_ratio: cr as u32,
            });
        }

        /// Only issuer can withdraw
        #[ink(message)]
        pub fn withdraw_dot(&mut self, cdp_id: CdpId, dai: Balance) -> Balance {
            assert!(self.cdps.contains_key(&cdp_id));
            let caller = self.env().caller();
            let cdp = self.cdps.get_mut(&cdp_id).unwrap();
            assert!(cdp.valid);
            assert!(cdp.issuer == caller);
            assert!(cdp.collateral_ratio >= self.min_collateral_ratio);
            assert!(dai <= cdp.issue_dai);
            let dot = cdp.collateral_dot * dai / cdp.issue_dai;
            cdp.collateral_dot -= dot;
            cdp.issue_dai -= dai;
            self.env().transfer(caller, dot).unwrap();
            self.dai_token.burn(caller, dai).unwrap();
            self.env().emit_event(Withdraw {
                cdp_id,
                collateral: dot,
                dai,
            });
            dot
        }

        /// Anyone can invoke collateral liquidation if current collateral ratio lower than minimum
        #[ink(message)]
        pub fn liquidate_collateral(&mut self, cdp_id: CdpId, dai: Balance) {
            assert!(self.cdps.contains_key(&cdp_id));
            let cdp = self.cdps.get_mut(&cdp_id).unwrap();
            assert!(cdp.valid);
            assert!(cdp.collateral_ratio <= self.min_collateral_ratio);
            assert!(cdp.issue_dai >= dai);
            cdp.valid = false;
            let owner = cdp.issuer;
            let dot = dai / self.dot_price as u128;
            cdp.issue_dai -= dai;
            let keeper_reward =
                dai * self.liquidater_reward_ratio as u128 / (100 * self.dot_price as u128);
            cdp.collateral_dot = cdp.collateral_dot - dot - keeper_reward;
            let mut rest_dot = 0_u128;
            if cdp.issue_dai == 0 && cdp.collateral_dot > 0 {
                rest_dot = cdp.collateral_dot;
                cdp.collateral_dot = 0;
            }
            self.env().transfer(owner, dot).unwrap();
            let caller = self.env().caller();
            self.env().transfer(caller, keeper_reward).unwrap();
            self.dai_token.burn(owner, dai).unwrap();
            if rest_dot > 0 {
                self.env().transfer(owner, rest_dot).unwrap();
            }
            self.env().emit_event(Liquidate {
                cdp_id,
                collateral: dot,
                keeper_reward,
            });
        }

        /// Returns the total dai token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.dai_token.total_supply()
        }

        fn only_owner(&self) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::OnlyOwnerAccess);
            }
            Ok(())
        }
    }
}
