#![cfg_attr(not(feature = "std"), no_std)]

use ink_babe_random::{BabeRandomness, CustomEnvironment};
use ink_lang as ink;

#[ink::contract(env = crate::CustomEnvironment)]
mod patralottery {
    use crate::BabeRandomness;
    use core::fmt::Write;
    use ink_prelude::{string::String, vec, vec::Vec};
    use ink_storage::{
        collections::HashMap as StorageMap,
        traits::{PackedLayout, SpreadLayout},
    };

    pub const DOTS: Balance = 10_000_000_000;

    pub type EpochID = u64;

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

    #[ink(event)]
    pub struct BuyTickets {
        #[ink(topic)]
        ticket_num: Vec<u32>,
        #[ink(topic)]
        amount: u32,
        #[ink(topic)]
        epoch: EpochID,
    }

    #[ink(event)]
    pub struct DrawLottery {
        #[ink(topic)]
        epoch: EpochID,
        #[ink(topic)]
        randomness: Hash,
        #[ink(topic)]
        win_num: Vec<u32>,
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
        pub win_num: Vec<u32>,
        pub buyers: Vec<AccountId>,
        pub pool_in: Balance,
        pub pool_out: Balance,
        pub end: bool,
    }

    #[derive(
        Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct MyLottery {
        pub epoch_id: EpochID,
        pub random: Hash,
        pub my_num: Vec<u32>,
        pub tickets: u32,
        pub reward: Balance,
    }

    #[derive(
        Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct EpochHistory {
        pub epoch_id: EpochID,
        pub random: Hash,
        pub my_num: Vec<u32>,
        pub buyer: u32,
        pub pool_in: Balance,
        pub pool_out: Balance,
    }

    #[derive(
        Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct EpochInfo {
        pub epoch_id: EpochID,
        pub start_slot: u64,
        pub duration: u64,
        pub current_block: u32,
        pub reward_pool: Balance,
    }

    #[derive(
        Debug,
        PartialEq,
        Eq,
        Clone,
        Default,
        scale::Encode,
        scale::Decode,
        SpreadLayout,
        PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct BiggestWinner {
        pub winner: AccountId,
        pub win_num: Vec<u32>,
        pub tickets: u32,
        pub reward: Balance,
    }

    #[derive(
        Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct Tickets {
        pub num: Vec<u32>,
        pub amount: u32,
        pub reward: Balance,
        pub rank: Rank,
    }

    #[ink(storage)]
    pub struct PatraLottery {
        epochs: StorageMap<EpochID, Lottery>,
        players: StorageMap<(EpochID, AccountId), Vec<Tickets>>,
        buyers: StorageMap<AccountId, Vec<EpochID>>,
        winners: StorageMap<EpochID, BiggestWinner>,
        reward_pool: Balance,
    }

    impl PatraLottery {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                epochs: StorageMap::new(),
                players: StorageMap::new(),
                buyers: StorageMap::new(),
                winners: StorageMap::new(),
                reward_pool: 0,
            }
        }

        #[ink(message, payable)]
        pub fn buy_tickets(&mut self, epoch_id: EpochID, num: Vec<u32>, amount: u32) {
            let ret: BabeRandomness = self.env().extension().next_epoch();
            assert!(epoch_id >= ret.epoch + 1);
            let caller = self.env().caller();
            let spend = self.env().transferred_balance();
            assert!(spend >= DOTS * amount as u128);
            assert_eq!(num.len(), 3);

            self.reward_pool += spend;

            // update epochs
            if let Some(epoch) = self.epochs.get_mut(&epoch_id) {
                epoch.pool_in += spend;
                if !epoch.buyers.contains(&caller) {
                    epoch.buyers.push(caller);
                }
            } else {
                self.epochs.insert(
                    epoch_id,
                    Lottery {
                        random: Default::default(),
                        win_num: vec![],
                        buyers: vec![caller],
                        pool_in: spend,
                        pool_out: 0,
                        end: false,
                    },
                );
            }

            // update players
            let ticket = Tickets {
                num: num.clone(),
                amount,
                reward: 0,
                rank: Rank::None,
            };
            if let Some(tic) = self.players.get_mut(&(epoch_id, caller)) {
                tic.push(ticket);
            } else {
                self.players.insert((epoch_id, caller), vec![ticket]);
            }

            // update buyers
            if let Some(eps) = self.buyers.get_mut(&caller) {
                if !eps.contains(&epoch_id) {
                    eps.push(epoch_id);
                }
            } else {
                self.buyers.insert(caller, vec![epoch_id]);
            }

            self.env().emit_event(BuyTickets {
                ticket_num: num,
                amount,
                epoch: epoch_id,
            })
        }

        #[ink(message)]
        pub fn draw_lottery(&mut self, epoch_id: EpochID) {
            let random_hash = self.env().extension().randomness_of(epoch_id);
            assert!(self.epochs.get(&epoch_id).is_some());
            let lottery = self.epochs.get_mut(&epoch_id).unwrap();
            assert!(!lottery.end);

            let (_hex_random, win_num) = Self::get_winning_number(random_hash);
            lottery.end = true;
            lottery.random = random_hash;
            lottery.win_num = win_num.clone();

            // claim reward
            let caller = Self::env().caller();
            // 0.1 DOT
            Self::env().transfer(caller, DOTS / 10).unwrap();
            self.reward_pool -= DOTS / 10;
            lottery.pool_out += DOTS / 10;

            let mut first_count = 0_u32;
            let mut first_palyers: Vec<AccountId> = Vec::new();
            let mut biggest_winner: BiggestWinner = Default::default();
            for buyer in lottery.buyers.iter() {
                if let Some(player) = self.players.get_mut(&(epoch_id, *buyer)) {
                    for tic in player {
                        let rank = Self::rank(tic.num.clone(), win_num.clone());
                        match rank {
                            Rank::FirstPrize => {
                                tic.rank = Rank::FirstPrize;
                                first_count += tic.amount;
                                first_palyers.push(*buyer);
                            }
                            Rank::SecondPrize => {
                                tic.rank = Rank::SecondPrize;
                                let reward = DOTS * 10 * tic.amount as u128;
                                if self.reward_pool > 0 {
                                    if self.reward_pool > reward {
                                        tic.reward = reward;
                                    } else {
                                        tic.reward = self.reward_pool;
                                    }
                                }
                            }
                            Rank::ThirdPrize => {
                                tic.rank = Rank::ThirdPrize;
                                let reward = DOTS * 2 * tic.amount as u128;
                                if self.reward_pool > 0 {
                                    if self.reward_pool > reward {
                                        tic.reward = reward;
                                    } else {
                                        tic.reward = self.reward_pool;
                                    }
                                }
                            }
                            _ => (),
                        }

                        if tic.reward > 0 {
                            Self::env().transfer(*buyer, tic.reward).unwrap();
                            if tic.reward > biggest_winner.reward {
                                biggest_winner = BiggestWinner {
                                    winner: *buyer,
                                    win_num: tic.num.clone(),
                                    tickets: tic.amount,
                                    reward: tic.reward,
                                }
                            }
                            self.reward_pool = self.reward_pool.saturating_sub(tic.reward);
                            lottery.pool_out += tic.reward;
                        }
                    }
                }
            }

            if self.reward_pool > 0 {
                let reward = self.reward_pool / first_count as u128;
                for player in first_palyers.iter() {
                    let tickets = self.players.get_mut(&(epoch_id, *player)).unwrap();
                    for tic in tickets.iter_mut() {
                        let money = reward * tic.amount as u128;
                        if tic.rank == Rank::FirstPrize {
                            Self::env().transfer(*player, money).unwrap();
                            tic.reward = money;
                        }
                        if tic.reward > biggest_winner.reward {
                            biggest_winner = BiggestWinner {
                                winner: *player,
                                win_num: tic.num.clone(),
                                tickets: tic.amount,
                                reward: tic.reward,
                            }
                        }
                    }
                }
                lottery.pool_out += self.reward_pool;
                self.reward_pool = 0;
            }

            self.winners.insert(epoch_id, biggest_winner);

            self.env().emit_event(DrawLottery {
                epoch: epoch_id,
                randomness: random_hash,
                win_num,
            })
        }

        /// Return the account bought lotteries for the specified `owner`.
        #[ink(message)]
        pub fn lotteries_of(&self, owner: AccountId) -> Vec<MyLottery> {
            let mut my_lotteries = vec![];
            if let Some(epochs) = self.buyers.get(&owner) {
                for ep in epochs.iter() {
                    let lottery = self.epochs.get(ep).unwrap();
                    let tickets = self.players.get(&(*ep, owner)).unwrap();
                    for tic in tickets.iter() {
                        my_lotteries.push(MyLottery {
                            epoch_id: *ep,
                            random: lottery.random,
                            my_num: tic.num.clone(),
                            tickets: tic.amount,
                            reward: tic.reward,
                        });
                    }
                }
            }
            my_lotteries
        }

        #[ink(message)]
        pub fn epoch_history(&self, epoch_id: EpochID) -> Option<EpochHistory> {
            if let Some(lottery) = self.epochs.get(&epoch_id) {
                Some(EpochHistory {
                    epoch_id,
                    random: lottery.random,
                    my_num: lottery.win_num.clone(),
                    buyer: lottery.buyers.len() as u32,
                    pool_in: lottery.pool_in,
                    pool_out: lottery.pool_out,
                })
            } else {
                None
            }
        }

        #[ink(message)]
        pub fn latest_epoch(&self) -> EpochInfo {
            let ret: BabeRandomness = self.env().extension().next_epoch();
            EpochInfo {
                epoch_id: ret.epoch + 1,
                start_slot: ret.start_slot,
                duration: ret.duration,
                current_block: self.env().block_number(),
                reward_pool: self.reward_pool,
            }
        }

        #[ink(message)]
        pub fn biggest_winner(&self, epoch_id: EpochID) -> Option<BiggestWinner> {
            if let Some(winner) = self.winners.get(&epoch_id) {
                Some(winner.clone())
            } else {
                None
            }
        }

        #[ink(message)]
        pub fn randomness_of(&self, epoch_id: EpochID) -> (String, Vec<u32>) {
            let ret = self.env().extension().randomness_of(epoch_id);
            Self::get_winning_number(ret)
        }
    }

    impl PatraLottery {
        fn rank(numbers: Vec<u32>, win_num: Vec<u32>) -> Rank {
            assert_eq!(numbers.len(), 3);
            let count = win_num
                .iter()
                .zip(numbers.iter())
                .filter(|(x, y)| **x == **y)
                .count();
            match count {
                3 => Rank::FirstPrize,
                2 => Rank::SecondPrize,
                1 => Rank::ThirdPrize,
                _ => Rank::None,
            }
        }

        pub fn get_winning_number(random: Hash) -> (String, Vec<u32>) {
            let mut seed = String::new();
            for byte in random.as_ref() {
                write!(&mut seed, "{:x}", byte).expect("Unable to write");
            }
            let mut win: Vec<u32> = vec![];
            for (n, v) in seed.chars().filter_map(|x| x.to_digit(10)).enumerate() {
                if n < 3 {
                    win.push(v);
                }
            }
            (seed, win)
        }
    }
}
