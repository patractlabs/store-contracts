#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod patralottery {
    // use ink_env::AccountId;
    use ink_prelude::{string::String, vec, vec::Vec};
    use ink_storage::{
        collections::HashMap as StorageMap,
        traits::{PackedLayout, SpreadLayout},
    };

    pub const DOTS: Balance = 1_000_000_000_000;

    pub type EpochID = u32;

    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone, Copy, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub enum Rank {
        None,
        FirstPrize,
        SecondPrize,
        ThirdPrize,
    }

    #[derive(
        Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct Lottery {
        pub random: Hash,
        pub win_num: String,
        pub buyers: Vec<AccountId>,
    }

    #[derive(
        Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct Tickets {
        pub num: String,
        pub amount: u32,
        pub reward: Balance,
        pub rank: Rank,
    }

    #[ink(storage)]
    pub struct PatraLottery {
        epochs: StorageMap<EpochID, Lottery>,
        players: StorageMap<(EpochID, AccountId), Vec<Tickets>>,
        reward_pool: Balance,
        current_epoch: EpochID,
    }

    impl PatraLottery {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                epochs: StorageMap::new(),
                players: StorageMap::new(),
                reward_pool: 0,
                current_epoch: 0,
            }
        }

        #[ink(message, payable)]
        pub fn bug_tickets(&mut self, num: String, amount: u32) {
            let caller = self.env().caller();
            let spend = self.env().transferred_balance();
            assert_eq!(spend, DOTS * amount as u128);

            self.reward_pool += spend;

            let ticket = Tickets {
                num,
                amount,
                reward: 0,
                rank: Rank::None,
            };

            if let Some(tic) = self.players.get_mut(&(self.current_epoch, caller)) {
                tic.push(ticket);
            } else {
                self.players
                    .insert((self.current_epoch, caller), vec![ticket]);
            }

            if let Some(ep) = self.epochs.get_mut(&self.current_epoch) {
                if !ep.buyers.contains(&caller) {
                    ep.buyers.push(caller)
                }
            }
        }

        #[ink(message)]
        pub fn draw_lottery(&mut self, epoch: EpochID) {
            assert_eq!(epoch, self.current_epoch);
            let win_str = "1 2 3";
            let win_num: Vec<i32> = win_str
                .split_whitespace()
                .map(|s| s.parse().expect("parse error"))
                .collect();

            let caller = self.env().caller();
            // 0.1 DOT
            self.env().transfer(caller, DOTS / 10).unwrap();
            self.reward_pool -= DOTS / 10;

            if let Some(lottery) = self.epochs.get(&epoch) {
                let mut first_count = 0_u32;
                let mut first_palyers: StorageMap<AccountId, u32> = StorageMap::new();
                for buyer in lottery.buyers.iter() {
                    if let Some(player) = self.players.get(&(epoch, *buyer)) {
                        for tic in player {
                            let rank = Self::rank(tic.num.clone(), win_num.clone());
                            match rank {
                                Rank::FirstPrize => {
                                    first_count += tic.amount;
                                    // TODO
                                    first_palyers.insert(*buyer, tic.amount);
                                }
                                Rank::SecondPrize => {
                                    let reward = DOTS * 10 * tic.amount as u128;
                                    let rest = self.reward_pool - reward;
                                    if rest > 0 {
                                        self.env().transfer(*buyer, reward).unwrap();
                                        self.reward_pool -= reward;
                                    } else {
                                        self.env().transfer(*buyer, self.reward_pool).unwrap();
                                        self.reward_pool = 0;
                                    }
                                }
                                Rank::ThirdPrize => {
                                    let reward = DOTS * 2 * tic.amount as u128;
                                    let rest = self.reward_pool - reward;
                                    if rest > 0 {
                                        self.env().transfer(*buyer, reward).unwrap();
                                        self.reward_pool -= reward;
                                    } else {
                                        self.env().transfer(*buyer, self.reward_pool).unwrap();
                                        self.reward_pool = 0;
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }

                if self.reward_pool > 0 {
                    let reward = self.reward_pool / first_count as u128;
                    for (k, v) in first_palyers.iter() {
                        self.env().transfer(*k, reward * *v as u128).unwrap();
                        self.reward_pool -= reward * *v as u128;
                    }
                }
            }
        }

        fn rank(numbers: String, win_num: Vec<i32>) -> Rank {
            let nums: Vec<i32> = numbers
                .split_whitespace()
                .map(|s| s.parse().expect("parse error"))
                .collect();
            assert_eq!(numbers.len(), 3);

            let mut count = 0_u8;
            for (i, v) in win_num.iter().enumerate() {
                if nums[i] == *v {
                    count += 1;
                }
            }
            match count {
                3 => Rank::FirstPrize,
                2 => Rank::SecondPrize,
                1 => Rank::ThirdPrize,
                _ => Rank::None,
            }
        }
    }
}
