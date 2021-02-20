#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod patrapixel {
    use ink_prelude::{string::String, vec, vec::Vec};
    use ink_storage::collections::HashMap as StorageHashMap;

    pub const DOTS: Balance = 10_000_000_000;

    #[ink(event)]
    pub struct PixelUpdate {
        #[ink(topic)]
        creator: AccountId,
    }

    #[ink(storage)]
    pub struct Patrapixel {
        name: String,
        metadata: StorageHashMap<u32, u8>,
        pool: Balance,
        size: (u32, u32),
    }

    impl Patrapixel {
        #[ink(constructor)]
        pub fn new(x: u32, y: u32) -> Self {
            Self {
                name: "PatraPixel".parse().unwrap(),
                metadata: Default::default(),
                pool: 0,
                size: (x, y),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(320, 180)
        }

        /// Get and returns pixel metadata
        #[ink(message)]
        pub fn metadata(&self) -> Vec<(u32, u8)> {
            let mut vec = vec![];
            for (k, v) in self.metadata.iter() {
                vec.push((*k, *v));
            }
            vec
        }

        #[ink(message)]
        pub fn pool(&self) -> Balance {
            self.pool
        }

        #[ink(message)]
        pub fn size(&self) -> (u32, u32) {
            self.size
        }

        /// update pixel with metadata
        #[ink(message, payable)]
        pub fn update(&mut self, points: Vec<(u32, u8)>) {
            assert!(points.len() > 0);
            let cost = self.env().transferred_balance();
            assert!(cost >= points.len() as u128 * DOTS);
            points.iter().for_each(|x| {
                if let Some(v) = self.metadata.get_mut(&x.0) {
                    *v = x.1;
                } else {
                    self.metadata.insert(x.0, x.1);
                }
            });
            self.pool += cost;
            self.env().emit_event(PixelUpdate {
                creator: self.env().caller(),
            });
        }
    }
}
