#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod simple_contract {

    use ink_storage::{traits::SpreadAllocate, Mapping};
    use ink_prelude::vec::Vec;
    use ink_prelude::vec;
    use ink_prelude::{string::String, format};

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct SimpleContract {
        /// Stores a fund of seller on the storage.
        seller_fund: Mapping<AccountId, Balance>,
        /// Stores a asset of seller on the storage.
        seller_asset: Mapping<AccountId, Vec<(Hash, Balance)>>,
        /// Stores a fund of buyer on the storage.
        buyer_fund: Mapping<AccountId, Balance>,
        /// Stores a asset of buyer on the storage.
        buyer_asset: Mapping<AccountId, Vec<(Hash, Balance)>>,
        /// Seller
        seller: AccountId,
        /// Buyer
        buyer: AccountId,
        /// Money
        price: Balance,
        /// Asset
        asset: Hash,
    }

    impl SimpleContract {
        #[ink(constructor)]
        /// Constructor that initializes the sell contract.
        pub fn new_sell(init_item: Hash, init_price: Balance) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.asset = init_item;
                contract.price = init_price;
                let caller = Self::env().caller();
                contract.seller = caller;
                let value = vec![(contract.asset, contract.price)];
                contract.seller_asset.insert(&caller, &value);
            })
        }
    
        /// Default initializes the contract.
        #[ink(constructor)]
        pub fn sell_default() -> Self {
            // Even though we're not explicitly initializing the `Mapping`,
            // we still need to call this
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.asset = Hash::default();
                contract.price = Default::default();
            })
        }

        /// Seller add asset
        #[ink(message)]
        pub fn insert_asset(&mut self, item: Hash, price: Balance) {
            let caller = self.env().caller();
            let asset = item;
            let price = price;
            let _x = self.get_asset_data(caller);
            if self.seller_asset.contains(&caller) {
                panic!("Asset exists");
                ink_env::debug_println!("Asset exists");
            } else {
                self.seller_asset.insert(&caller, &vec![(asset, price)])
            } 
        } 
        
        /// Simply returns the current asset of seller.
        #[ink(message)]
        pub fn get_asset_data(&self, id: AccountId) -> Option<Vec<(Hash, Balance)>> {
            self.seller_asset.get(&id)
        }

        /// Check current fund of buyer
        #[ink(message)]
        pub fn check_fund(&self, id: AccountId) -> Option<Balance> {
            self.buyer_fund.get(&id)
        }

        #[ink(message)]
        pub fn total_status(&self, id: AccountId) -> String {
            if id == self.seller {
                match self.seller_asset.get(&id) {
                    Some(x) => {
                        let item = x[0].0;
                        let price = self.seller_asset.get(&id).unwrap_or_default()[0].1;
                        let fund = self.seller_fund.get(&id).unwrap_or_default();
                        format!("Current item: {:?}. Current price: {}. Fund: {}", item, price, fund)
                    },
                    None => {
                        let fund = self.seller_fund.get(&id).unwrap_or_default();
                        format!("No item data. Fund: {}",fund)
                    },
                }
            } else if id == self.buyer {
                match self.buyer_asset.get(&id) {
                    Some(x) => {
                        let item = x[0].0;
                        let price = self.buyer_asset.get(&id).unwrap_or_default()[0].1;
                        let fund = self.buyer_fund.get(&id).unwrap_or_default();
                        format!("Current item: {:?}. Current price: {}. Fund: {}", item, price, fund)
                    },
                    None => {
                        let fund = self.buyer_fund.get(&id).unwrap_or_default();
                        format!("No item data. Fund: {}",fund)
                    },
                }
            } else {
                format!("No data")
            }      
        }

        /// Buyer deposit money first time or next times
        #[ink(message)]
        pub fn buyer_deposit_money(&mut self, id: AccountId, money: Balance) {
            let caller = self.env().caller();
            let fund = self.buyer_fund.get(&caller).unwrap_or_default() + money;
            self.buyer_fund.remove(&caller);
            self.buyer_fund.insert(&caller, &fund);
            if caller == id { panic!("You own your asset") };
            if caller == self.seller { panic!("Seller can't do this") };
            self.buyer = caller;
            if !self.seller_asset.contains(&id) { panic!("Not available yet")};

        }

        /// Settle the contract when asset from seller & money from buyer was set in. Then terminate contract
        #[ink(message)]
        pub fn settle(&mut self, id: AccountId) {
            let caller = self.env().caller();
            assert!(self.seller_asset.contains(&caller) && self.buyer_fund.contains(&id), "No asset or fund");
            let item = self.seller_asset.get(&caller).unwrap()[0].0;
            let price = self.seller_asset.get(&caller).unwrap()[0].1;

            let fund = self.buyer_fund.get(&id).unwrap();
            if fund < price {
                panic!("Not enough fund");
            }

            let money = fund - price;
            self.buyer_asset.insert(&id, &vec![(item, price)]);
            match money {
                x if x > 0 => {
                    self.buyer_fund.remove(&id);
                    self.buyer_fund.insert(&id, &x);
                },
                _ => self.buyer_fund.remove(&id),
            };
            self.seller_asset.remove(caller);
            self.seller_fund.insert(&caller, &price);
            self.env().terminate_contract(self.env().caller());

        }

    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        fn default_accounts() -> ink_env::test::DefaultAccounts<Environment> {
            ink_env::test::default_accounts::<Environment>()
        }

        fn alice() -> AccountId {
            default_accounts().alice
        }

        fn bob() -> AccountId {
            default_accounts().bob
        }

        fn item(data: [u8; 32]) -> Hash {
            ink_env::Hash::from(data)
        }

        /// We test if the default constructor does its job. Then add new asset to default
        #[ink::test]
        fn default_sell() {
            let caller = alice();
            let mut contract = SimpleContract::sell_default();
            assert_eq!(contract.get_asset_data(caller), None);

            let price = 20;
            contract.insert_asset(item([0; 32]), price);
            assert_eq!(contract.get_asset_data(caller), Some(vec![(item([0; 32]), price)]));
        }

        /// We test new deploy of our contract.
        #[ink::test]
        fn creat_new_sell() {
            let price: Balance = 450;
            let contract = SimpleContract::new_sell(item([1;32]), price);
            let caller = alice();
            assert_eq!(contract.get_asset_data(caller), Some(vec![(item([1;32]), price)]));
            assert!(contract.seller == alice())
        }    
        
        /// We test insert asset to contract where another asset already existed
        #[ink::test]
        #[should_panic]
        fn insert_asset_fail() {
            let price: Balance = 450;
            let mut contract = SimpleContract::new_sell(item([1; 32]), price);
            contract.insert_asset(item([2; 32]), price)
        }

        #[ink::test]
        fn test_settle() {
            let caller = alice();
            let price: Balance = 450;
            let mut contract = SimpleContract::new_sell(item([1; 32]), price);
            assert_eq!(contract.get_asset_data(caller), Some(vec![(item([1; 32]), price)]));
            assert!(contract.seller_asset.contains(&caller));
            contract.buyer_deposit_money(alice(), 600);
            // contract.settle(alice());
            // assert_eq!(contract.buyer_asset.get(&bob()).unwrap(), vec![(item([1; 32]), price)]);
        }

        #[ink::test]
        fn test_deposit() {
            let price: Balance = 450;
            let mut contract = SimpleContract::new_sell(item([1; 32]), price);
            let caller = bob();
            contract.buyer_deposit_money(alice(), 600);
            assert!(contract.seller != alice())
        }
    }
}
