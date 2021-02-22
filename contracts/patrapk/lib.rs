#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod patrapk {
    use ink_env::hash::Blake2x256;
    use ink_prelude::{format, string::String};
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
        None,
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

    #[ink(event)]
    pub struct PKCreate {
        #[ink(topic)]
        creator: AccountId,
        #[ink(topic)]
        salt_hash: Hash,
        #[ink(topic)]
        value: Balance,
    }

    #[ink(event)]
    pub struct PKDelete {
        #[ink(topic)]
        game_id: GameID,
        #[ink(topic)]
        creator: AccountId,
    }

    #[ink(event)]
    pub struct PKJoin {
        #[ink(topic)]
        game_id: GameID,
        #[ink(topic)]
        joiner: AccountId,
        #[ink(topic)]
        joiner_choice: Choice,
    }

    #[ink(event)]
    pub struct PKReveal {
        #[ink(topic)]
        game_id: GameID,
        #[ink(topic)]
        result: GameResult,
    }

    #[ink(event)]
    pub struct PKExpire {
        #[ink(topic)]
        game_id: GameID,
        #[ink(topic)]
        status: GameStatus,
        #[ink(topic)]
        result: GameResult,
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
        pub join_block: BlockNumber,
        pub salt: String,
        pub salt_hash: Hash,
        pub create_choice: Choice,
        pub value: Balance,
        pub status: GameStatus,
        pub joiner: AccountId,
        pub joiner_choice: Choice,
        pub result: GameResult,
    }

    impl Default for GameDetails {
        fn default() -> GameDetails {
            GameDetails {
                creator: Default::default(),
                join_block: 0,
                salt: "".parse().unwrap(),
                salt_hash: Default::default(),
                create_choice: Choice::None,
                value: 0,
                status: GameStatus::None,
                joiner: Default::default(),
                joiner_choice: Choice::None,
                result: GameResult::None,
            }
        }
    }

    #[ink(storage)]
    pub struct Patrapk {
        games: StorageMap<GameID, GameDetails>,
        counter: u32,
        expire_time: BlockNumber,
    }

    impl Patrapk {
        #[ink(constructor)]
        pub fn new(expire_time: BlockNumber) -> Self {
            Self {
                games: StorageMap::new(),
                counter: 0,
                expire_time,
            }
        }

        // salt_hash = Hash("salt-rock/paper/scissors")
        #[ink(message, payable)]
        pub fn create(&mut self, salt_hash: Hash) -> GameID {
            let mut game = GameDetails::default();
            game.creator = self.env().caller();
            game.salt_hash = salt_hash;
            game.value = self.env().transferred_balance();
            game.status = GameStatus::Join;
            self.counter += 1;
            self.games.insert(self.counter, game.clone());
            self.env().emit_event(PKCreate {
                creator: game.creator,
                salt_hash,
                value: game.value,
            });
            self.counter
        }

        #[ink(message)]
        pub fn delete(&mut self, game_id: GameID) {
            let game = self.games.get(&game_id).unwrap();
            let caller = self.env().caller();
            assert_eq!(game.creator, caller, "Not creator");
            assert_eq!(game.status, GameStatus::Join, "Cannot delete");

            self.env().transfer(game.creator, game.value).unwrap();
            self.games.get_mut(&game_id).and_then(|x| {
                x.status = GameStatus::Delete;
                Some(x)
            });
            self.env().emit_event(PKDelete {
                game_id,
                creator: caller,
            });
        }

        #[ink(message, payable)]
        pub fn join(&mut self, game_id: GameID, choice: Choice) {
            let caller = self.env().caller();
            let value = self.env().transferred_balance();
            let join_block = self.env().block_number();
            let game = self.games.get_mut(&game_id).unwrap();

            assert_ne!(game.creator, caller, "Game creator");
            assert_eq!(game.status, GameStatus::Join, "Cannot delete");
            assert_eq!(value, game.value, "Invalid stake");

            game.join_block = join_block;
            game.status = GameStatus::Settle;
            game.joiner = caller;
            game.joiner_choice = choice;

            self.env().emit_event(PKJoin {
                game_id,
                joiner: caller,
                joiner_choice: choice,
            });
        }

        #[ink(message)]
        pub fn reveal(&mut self, game_id: GameID, salt: String, choice: Choice) {
            let game = self.games.get(&game_id).unwrap();
            assert_eq!(game.status, GameStatus::Settle, "Cannot Reveal");
            let salt_hash = self.salt_hash(salt.clone(), choice);
            let expected_salt_hash = game.salt_hash;
            assert_eq!(salt_hash, expected_salt_hash, "Invalid Salt");

            let result = Self::judgment(choice, game.joiner_choice).unwrap();
            match result {
                GameResult::Draw => {
                    self.env().transfer(game.creator, game.value).unwrap();
                    self.env().transfer(game.joiner, game.value).unwrap();
                }
                GameResult::CreatorWin => {
                    self.env().transfer(game.creator, game.value * 2).unwrap();
                }
                GameResult::JoinerWin => {
                    let creator_reward = game.value * 2 * 5 / 100;
                    self.env().transfer(game.creator, creator_reward).unwrap();
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
                x.salt = salt;
                Some(x)
            });
            self.env().emit_event(PKReveal { game_id, result });
        }

        #[ink(message)]
        pub fn expire(&mut self, game_id: GameID) {
            let game = self.games.get(&game_id).unwrap();
            assert_eq!(game.status, GameStatus::Settle, "Cannot Expire");
            assert!(
                self.env().block_number() >= game.join_block + self.expire_time,
                "Not Expired"
            );

            self.env().transfer(game.joiner, game.value * 2).unwrap();
            self.games.get_mut(&game_id).and_then(|x| {
                x.status = GameStatus::Expire;
                x.result = GameResult::JoinerWin;
                Some(x)
            });
            self.env().emit_event(PKExpire {
                game_id,
                status: GameStatus::Expire,
                result: GameResult::JoinerWin,
            });
        }

        #[ink(message)]
        pub fn salt_hash(&self, salt: String, choice: Choice) -> Hash {
            let choice_str = match choice {
                Choice::Rock => "rock",
                Choice::Paper => "paper",
                Choice::Scissors => "scissors",
                _ => "",
            };

            let salt = format!("{}-{}", salt, choice_str);
            Hash::from(self.env().hash_bytes::<Blake2x256>(salt.as_bytes()))
        }

        #[ink(message)]
        pub fn game_of(&self, game_id: GameID) -> Result<GameDetails> {
            let game = self.games.get(&game_id).ok_or(Error::GameNotFound)?;
            Ok(game.clone())
        }

        #[ink(message)]
        pub fn expire_of(&self, game_id: GameID) -> BlockNumber {
            let game = self.games.get(&game_id).unwrap();
            let epoch = self.env().block_number().saturating_sub(game.join_block);
            self.expire_time.saturating_sub(epoch)
        }

        #[ink(message)]
        pub fn game_total(&self) -> u32 {
            self.counter
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
                (Choice::Rock, Choice::None)
                | (Choice::Paper, Choice::None)
                | (Choice::Scissors, Choice::None) => Some(GameResult::CreatorWin),
                (Choice::None, Choice::Rock)
                | (Choice::None, Choice::Paper)
                | (Choice::None, Choice::Scissors) => Some(GameResult::JoinerWin),
                _ => None,
            }
        }
    }
}
