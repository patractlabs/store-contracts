#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::Environment;
use ink_lang as ink;
pub enum CustomEnvironment {}

#[derive(Debug, PartialEq, scale::Decode)]
pub struct Randomness {
    pub epoch: u64,
    pub randomness: <ink_env::DefaultEnvironment as Environment>::Hash,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum ErrorCode {
    InvalidKey,
}

impl ink_env::chain_extension::FromStatusCode for ErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::InvalidKey),
            _ => panic!("encountered unknown status code"),
        }
    }
}

/// Custom chain extension to read to and write from the runtime.
#[ink::chain_extension]
pub trait BabeRandomness {
    type ErrorCode = ErrorCode;

    /// Reads from runtime storage.
    #[ink(extension = 0x02000000, returns_result = false)]
    fn read(key: &[u8]) -> Randomness;
}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = BabeRandomness;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod patralottery {
    use crate::Randomness;
    use core::fmt::Write;
    use ink_prelude::{string::String, vec, vec::Vec};
    use ink_storage::{
        collections::HashMap as StorageMap,
        traits::{PackedLayout, SpreadLayout},
    };

    pub const DOTS: Balance = 1_000_000_000_000;

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
        pub num: Vec<u32>,
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
        pub fn bug_tickets(&mut self, num: Vec<u32>, amount: u32) {
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
        pub fn draw_lottery(&mut self) {
            let ret: Randomness = self.env().extension().read("".as_bytes()).unwrap();
            let epoch = ret.epoch;
            assert_eq!(epoch, self.current_epoch);
            let random = self.env().random("".as_bytes());
            let win_num = self.get_winning_number(random);

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

        pub fn get_winning_number(&self, random: Hash) -> Vec<u32> {
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
            win
        }

        #[ink(message)]
        pub fn winning_number(&self) -> (String, Vec<u32>) {
            let mut seed = String::new();
            let ret: Randomness = self.env().extension().read("".as_bytes()).unwrap();
            for byte in ret.randomness.as_ref() {
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
