#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod patrapk {
    use ink_env::hash::Blake2x256;
    use ink_prelude::string::String;
    use ink_storage::{
        collections::HashMap as StorageMap,
        traits::{PackedLayout, SpreadLayout},
    };

    pub type GameID = u32;

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        GameCreator,
        NotCreator,
        CannotJoin,
        CannotReveal,
        CannotDelete,
        CannotExpire,
        NotExpired,
        InvalidStake,
        InvalidSalt,
        InvalidChoice,
        GameNotFound,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone, Copy, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub enum Choice {
        None,
        Rock,
        Paper,
        Scissors,
    }

    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone, Copy, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub enum GameStatus {
        Join,
        Delete,
        Settle,
        End,
        Expire,
    }

    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone, Copy, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub enum GameResult {
        None,
        Draw,
        CreatorWin,
        JoinerWin,
    }

    #[derive(
        Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct GameDetails {
        pub creator: AccountId,
        pub start_time: BlockNumber,
        pub salt_hash: Hash,
        pub create_choice: Choice,
        pub value: Balance,
        pub status: GameStatus,
        pub joiner: AccountId,
        pub joiner_choice: Choice,
        pub result: GameResult,
    }

    #[ink(storage)]
    pub struct Patrapk {
        games: StorageMap<GameID, GameDetails>,
        counter: u32,
    }

    impl Patrapk {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                games: StorageMap::new(),
                counter: 0,
            }
        }

        #[ink(message, payable)]
        pub fn create(&mut self, salt_hash: Hash) -> GameID {
            let game = GameDetails {
                creator: self.env().caller(),
                start_time: self.env().block_number(),
                salt_hash,
                create_choice: Choice::None,
                value: self.env().transferred_balance(),
                status: GameStatus::Join,
                joiner: Default::default(),
                joiner_choice: Choice::None,
                result: GameResult::None,
            };
            self.counter += 1;
            self.games.insert(self.counter, game);
            self.counter
        }

        #[ink(message)]
        pub fn delete(&mut self, game_id: GameID) -> Result<()> {
            let game = self.games.get(&game_id).ok_or(Error::GameNotFound)?;
            if game.creator != self.env().caller() {
                return Err(Error::NotCreator);
            }
            if game.status != GameStatus::Join {
                return Err(Error::CannotDelete);
            }
            self.env().transfer(game.creator, game.value).unwrap();
            self.games.get_mut(&game_id).and_then(|x| {
                x.status = GameStatus::Delete;
                Some(x)
            });
            Ok(())
        }

        #[ink(message, payable)]
        pub fn join(&mut self, game_id: GameID, choice: Choice) -> Result<()> {
            let caller = self.env().caller();
            let value = self.env().transferred_balance();
            match self.games.get_mut(&game_id) {
                Some(game) => {
                    if game.creator == caller {
                        return Err(Error::GameCreator);
                    }
                    if game.status != GameStatus::Join {
                        return Err(Error::CannotJoin);
                    }
                    if value != game.value {
                        return Err(Error::InvalidStake);
                    }
                    game.status = GameStatus::Settle;
                    game.joiner = caller;
                    game.joiner_choice = choice;
                    Ok(())
                }
                None => Err(Error::GameNotFound),
            }
        }

        #[ink(message)]
        pub fn reveal(&mut self, salt: String, choice: Choice, game_id: GameID) -> Result<()> {
            let game = self.games.get(&game_id).ok_or(Error::GameNotFound)?;
            if game.status != GameStatus::Settle {
                return Err(Error::CannotReveal);
            }
            // TODO
            let salt_hash = self.env().hash_bytes::<Blake2x256>(salt.as_bytes());
            if Hash::from(salt_hash) != game.salt_hash {
                return Err(Error::InvalidSalt);
            }
            let result = Self::judgment(choice, game.joiner_choice).unwrap();
            match result {
                GameResult::Draw => {
                    self.env().transfer(game.creator, game.value).unwrap();
                    self.env().transfer(game.joiner, game.value).unwrap();
                }
                GameResult::CreatorWin => {
                    self.env().transfer(game.creator, game.value * 2).unwrap();
                }
                // TODO
                GameResult::JoinerWin => {
                    let creator_reward = game.value * 2 * 5 / 100;
                    self.env()
                        .transfer(game.creator, creator_reward)
                        .unwrap();
                    self.env()
                        .transfer(game.joiner, game.value * 2 - creator_reward)
                        .unwrap();
                }
                _ => (),
            }
            self.games.get_mut(&game_id).and_then(|x| {
                x.create_choice = choice;
                x.status = GameStatus::End;
                x.result = result;
                Some(x)
            });
            Ok(())
        }

        #[ink(message)]
        pub fn expire(&mut self, game_id: GameID) -> Result<()> {
            let game = self.games.get(&game_id).ok_or(Error::GameNotFound)?;
            if game.status != GameStatus::Settle {
                return Err(Error::CannotExpire);
            }
            if self.env().block_number() < game.start_time + 14400 {
                return Err(Error::NotExpired);
            }
            self.env().transfer(game.joiner, game.value * 2).unwrap();
            self.games.get_mut(&game_id).and_then(|x| {
                x.status = GameStatus::Expire;
                x.result = GameResult::JoinerWin;
                Some(x)
            });
            Ok(())
        }

        #[ink(message)]
        pub fn game_of(&self, game_id: GameID) -> Result<GameDetails> {
            let game = self.games.get(&game_id).ok_or(Error::GameNotFound)?;
            Ok(game.clone())
        }
    }

    impl Patrapk {
        fn judgment(creator: Choice, joiner: Choice) -> Option<GameResult> {
            match (creator, joiner) {
                (Choice::Rock, Choice::Rock)
                | (Choice::Paper, Choice::Paper)
                | (Choice::Scissors, Choice::Scissors) => Some(GameResult::Draw),
                (Choice::Rock, Choice::Scissors)
                | (Choice::Paper, Choice::Rock)
                | (Choice::Scissors, Choice::Paper) => Some(GameResult::CreatorWin),
                (Choice::Scissors, Choice::Rock)
                | (Choice::Rock, Choice::Paper)
                | (Choice::Paper, Choice::Scissors) => Some(GameResult::JoinerWin),
                _ => None,
            }
        }
    }
}
