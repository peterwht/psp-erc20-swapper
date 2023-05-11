#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{env::Environment, prelude::vec::Vec};

type DefaultAccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
type DefaultBalance = <ink::env::DefaultEnvironment as Environment>::Balance;

#[ink::chain_extension]
pub trait Psp22Extension {
    type ErrorCode = Psp22Error;

    // PSP22 Metadata interfaces

    #[ink(extension = 0x3d26)]
    fn token_name(asset_id: u32) -> Result<Vec<u8>>;

    #[ink(extension = 0x3420)]
    fn token_symbol(asset_id: u32) -> Result<Vec<u8>>;

    #[ink(extension = 0x7271)]
    fn token_decimals(asset_id: u32) -> Result<u8>;

    // PSP22 interface queries

    #[ink(extension = 0x162d)]
    fn total_supply(asset_id: u32) -> Result<DefaultBalance>;

    #[ink(extension = 0x6568)]
    fn balance_of(asset_id: u32, owner: DefaultAccountId) -> Result<DefaultBalance>;

    #[ink(extension = 0x4d47)]
    fn allowance(
        asset_id: u32,
        owner: DefaultAccountId,
        spender: DefaultAccountId,
    ) -> Result<DefaultBalance>;

    // PSP22 transfer
    #[ink(extension = 0xdb20)]
    fn transfer(asset_id: u32, to: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    // PSP22 transfer_from
    #[ink(extension = 0x54b3)]
    fn transfer_from(
        asset_id: u32,
        from: DefaultAccountId,
        to: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 approve
    #[ink(extension = 0xb20f)]
    fn approve(asset_id: u32, spender: DefaultAccountId, value: DefaultBalance) -> Result<()>;

    // PSP22 increase_allowance
    #[ink(extension = 0x96d6)]
    fn increase_allowance(
        asset_id: u32,
        spender: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;

    // PSP22 decrease_allowance
    #[ink(extension = 0xfecb)]
    fn decrease_allowance(
        asset_id: u32,
        spender: DefaultAccountId,
        value: DefaultBalance,
    ) -> Result<()>;
}

#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Psp22Error {
    TotalSupplyFailed,
}

pub type Result<T> = core::result::Result<T, Psp22Error>;

impl From<scale::Error> for Psp22Error {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

impl ink::env::chain_extension::FromStatusCode for Psp22Error {
    fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::TotalSupplyFailed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

/// An environment using default ink environment types, with PSP-22 extension included
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = DefaultAccountId;
    type Balance = DefaultBalance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;

    type ChainExtension = crate::Psp22Extension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod psp22_ext {
    use ink::{prelude::vec::Vec, storage::Mapping};

    use super::Result;

    pub type AssetId = u32;
    use erc20::Erc20Ref;
    /// A chain extension which implements the PSP-22 fungible token standard.
    /// For more details see <https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md>
    #[ink(storage)]
    pub struct Psp22Extension {
        asset_pairs: Mapping<AssetId, Erc20Ref>,
        asset_pair: ink::contract_ref!(Erc20Trait),
    }

    impl Psp22Extension {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new(erc20_address: AccountId) -> Self {
            Self {
                asset_pairs: Mapping::new(),
                asset_pair: erc20_address.into(),
            }
        }

        #[ink(message)]
        pub fn create_asset_pair(&mut self, asset_id: u32, erc20_address: Erc20Ref) {
            // Map `asset_id` to the ERC20 contract for this pair
            self.asset_pairs
                .insert::<AssetId, Erc20Ref>(asset_id, &erc20_address);
        }

        #[ink(message)]
        pub fn swap_for_asset(&mut self, asset_id: u32, amount: Balance) {
            let mut erc20 = self
                .asset_pairs
                .get(asset_id)
                .expect("Asset pair not found!");

            // contract needs to be approved to spend funds
            let erc20_result =
                erc20.transfer_from(self.env().caller(), self.env().account_id(), amount);

            assert!(erc20_result.is_ok(), "erc20_result {:?}", erc20_result);

            let ext_result = self
                .env()
                .extension()
                .transfer(asset_id, self.env().caller(), amount);

            assert!(ext_result.is_ok(), "ext_result {:?}", ext_result);
        }

        #[ink(message)]
        pub fn swap_asset(&mut self, asset_id: u32, amount: Balance) {
            // contract needs to be approved to spend funds
            let erc20_result =
               Erc20Trait::transfer_from(self, self.env().caller(), self.env().account_id(), amount);

            // OR:
            // let erc20_result =
            //    self.asset_pair.transfer_from(self.env().caller(), self.env().account_id(), amount);

            assert!(erc20_result.is_ok(), "erc20_result {:?}", erc20_result);

            let ext_result = self
                .env()
                .extension()
                .transfer(asset_id, self.env().caller(), amount);

            assert!(ext_result.is_ok(), "ext_result {:?}", ext_result);
        }

        // PSP22 Metadata interfaces

        /// Returns the token name of the specified asset.
        #[ink(message, selector = 0x3d261bd4)]
        pub fn token_name(&self, asset_id: u32) -> Result<Vec<u8>> {
            self.env().extension().token_name(asset_id)
        }

        /// Returns the token symbol of the specified asset.
        #[ink(message, selector = 0x34205be5)]
        pub fn token_symbol(&self, asset_id: u32) -> Result<Vec<u8>> {
            self.env().extension().token_symbol(asset_id)
        }

        /// Returns the token decimals of the specified asset.
        #[ink(message, selector = 0x7271b782)]
        pub fn token_decimals(&self, asset_id: u32) -> Result<u8> {
            self.env().extension().token_decimals(asset_id)
        }

        // PSP22 interface queries

        /// Returns the total token supply of the specified asset.
        #[ink(message, selector = 0x162df8c2)]
        pub fn total_supply(&self, asset_id: u32) -> Result<Balance> {
            self.env().extension().total_supply(asset_id)
        }

        /// Returns the account balance for the specified asset & owner.
        #[ink(message, selector = 0x6568382f)]
        pub fn balance_of(&self, asset_id: u32, owner: AccountId) -> Result<Balance> {
            self.env().extension().balance_of(asset_id, owner)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`
        /// for the specified asset.
        #[ink(message, selector = 0x4d47d921)]
        pub fn allowance(
            &self,
            asset_id: u32,
            owner: AccountId,
            spender: AccountId,
        ) -> Result<Balance> {
            self.env().extension().allowance(asset_id, owner, spender)
        }

        // PSP22 transfer

        /// Transfers `value` amount of specified asset from the caller's account to the
        /// account `to`.
        #[ink(message, selector = 0xdb20f9f5)]
        pub fn transfer(&mut self, asset_id: u32, to: AccountId, value: Balance) -> Result<()> {
            self.env().extension().transfer(asset_id, to, value)
        }

        // PSP22 transfer_from

        /// Transfers `value` amount of specified asset on the behalf of `from` to the
        /// account `to`.
        #[ink(message, selector = 0x54b3c76e)]
        pub fn transfer_from(
            &mut self,
            asset_id: u32,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            self.env()
                .extension()
                .transfer_from(asset_id, from, to, value)
        }

        // PSP22 approve

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount of the specified asset.
        #[ink(message, selector = 0xb20f1bbd)]
        pub fn approve(&mut self, asset_id: u32, spender: AccountId, value: Balance) -> Result<()> {
            self.env().extension().approve(asset_id, spender, value)
        }

        // PSP22 increase_allowance

        /// Atomically increases the allowance for the specified asset granted to
        /// `spender` by the caller.
        #[ink(message, selector = 0x96d6b57a)]
        pub fn increase_allowance(
            &mut self,
            asset_id: u32,
            spender: AccountId,
            value: Balance,
        ) -> Result<()> {
            self.env()
                .extension()
                .increase_allowance(asset_id, spender, value)
        }

        // PSP22 decrease_allowance

        /// Atomically decreases the allowance for the specified asset granted to
        /// `spender` by the caller.
        #[ink(message, selector = 0xfecb57d5)]
        pub fn decrease_allowance(
            &mut self,
            asset_id: u32,
            spender: AccountId,
            value: Balance,
        ) -> Result<()> {
            self.env()
                .extension()
                .decrease_allowance(asset_id, spender, value)
        }

    }

    impl Erc20Trait for Psp22Extension {
        /// Returns the total token supply.
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.asset_pair.total_supply()
        }

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self.asset_pair.balance_of(owner)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.asset_pair.allowance(owner, spender)
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            self.asset_pair.transfer(to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            self.asset_pair.approve(spender, value)
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            self.asset_pair.transfer_from(from, to, value)
        }
    }

    /// Trait implemented by all ERC-20 respecting smart contracts.
    #[ink::trait_definition]
    pub trait Erc20Trait {
        /// Returns the total token supply.
        #[ink(message)]
        fn total_supply(&self) -> Balance;

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance;

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()>;

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()>;

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()>;
    }
}
