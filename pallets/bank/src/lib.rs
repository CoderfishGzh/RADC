#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{dispatch::DispatchResult,
                    pallet_prelude::*, PalletId,
                    traits::{Currency, ExistenceRequirement, Randomness},};
use frame_support::sp_runtime::app_crypto::TryFrom;
use frame_support::sp_runtime::traits::{IdentifyAccount, Verify};
use frame_support::sp_runtime::MultiSignature;
use frame_support::sp_runtime::MultiSigner;
use frame_support::sp_runtime::traits::Convert;

use frame_support::traits::UnixTime;
use frame_system::pallet_prelude::*;
use sp_core::{Bytes};
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::Zero;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;

use sp_application_crypto::sr25519;
use sp_application_crypto::sr25519::Public;
use sp_application_crypto::sr25519::Signature;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;
use primitives::Balance;
pub use primitives::p_bank::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const PALLET_ID1: PalletId = PalletId(*b"radcban1");
const PALLET_ID2: PalletId = PalletId(*b"radcban2");
const PALLET_ID3: PalletId = PalletId(*b"radcban3");
const PALLET_ID4: PalletId = PalletId(*b"radcban4");
const PALLET_ID5: PalletId = PalletId(*b"radcban5");

const BALANCE_UNIT: Balance = 1_000_000_000_000;

#[frame_support::pallet]
pub mod pallet {

    // use serde::de::Unexpected::Str;
    use frame_support::traits::Randomness;
    use sp_core::Hasher;
    use sp_runtime::traits::Hash;
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// currency to pay fees and hold balances
        type Currency: Currency<Self::AccountId>;

        /// block height to number
        type BlockNumberToNumber: Convert<Self::BlockNumber, u128> + Convert<u32, Self::BlockNumber>;

        /// digital transfer amount
        type NumberToBalance: Convert<u128, BalanceOf<Self>>;
        /// amount converted to numbers
        type BalanceToNumber: Convert<BalanceOf<Self>, u128>;

        /// health check interval
        #[pallet::constant]
        type HealthCheckInterval: Get<Self::BlockNumber>;

        /// time
        type UnixTime: UnixTime;

        type BankRandomness: Randomness<Self::Hash, Self::BlockNumber>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    /// Bank account
    #[pallet::storage]
    #[pallet::getter(fn bank_accounts)]
    pub(super) type BankAccounts<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        UserInfo<T::AccountId>,
        OptionQuery,
    >;

    // account authorzation list
    #[pallet::storage]
    #[pallet::getter(fn authorization_list)]
    pub(super) type AuthorizationList<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        Vec<T::AccountId>,
        OptionQuery,
    >;

    /// storage withdraw money list
    #[pallet::storage]
    #[pallet::getter(fn withdraw_money_list)]
    pub(super) type WithdrawMoneyList<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        Vec<WithdrawInfo<T::AccountId>>,
        OptionQuery,
    >;

    /// User public_key
    /// When the user want to signed , user will create the key pair
    #[pallet::storage]
    #[pallet::getter(fn users_publickey)]
    pub(super) type UsersPublicKey<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        Vec<u8>,
        OptionQuery,
    >;

    /// Encode text
    #[pallet::storage]
    #[pallet::getter(fn encode_text)]
    pub(super) type EncodeText<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {

        RegisterSuccess(T::AccountId),

        ChargeAmountSuccess(T::AccountId, u128),

        WithdrawSuccess(T::AccountId, u128),

        Yes,

        No,

        // (root, who)
        AuthoritySuccess(T::AccountId, T::AccountId),

        // (applyer, source, money)
        ApplyTradeSuccess(T::AccountId, T::AccountId, BalanceOf<T>),

        // (from, value)
        TradeSuccess(T::AccountId, BalanceOf<T>),

    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {


    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {

        NotExitBankAccount,

        NotEnoughMoney,

        AlreayAuthority,

        NotExitTransactionList,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn test_rand(
            account_id: OriginFor<T>,
        ) -> DispatchResult {

            // create the random
            let (output, block_num) = T::BankRandomness::random_seed();
            let test = output.as_ref()[0];
            let test = test.rem_euclid(5 as u8);


            Ok(())
        }

        // User Registration
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_user_account(
            account_id: OriginFor<T>,
            public_key: Vec<u8>,
        ) -> DispatchResult {
            // Signed?
            let user = ensure_signed(account_id)?;
            // Crate the bank account
            let bank_account = UserInfo {
                userid: user.clone(),
                amount: 0,
                user_key: public_key,
            };

            // Update the UserInfo in bank
            BankAccounts::<T>::insert(user.clone(), bank_account);

            Self::deposit_event(Event::RegisterSuccess(user));
            Ok(())
        }

        /// Account authorization
        /// Add the authorized account to who's authorization list
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn account_authority (
            account_id: OriginFor<T>,
            authority_account: T::AccountId,
        ) -> DispatchResult {
            // get the account id
            let who = ensure_signed(account_id.clone())?;

            // determine who has the bank account
            if !BankAccounts::<T>::contains_key(who.clone()) {
                Err(Error::<T>::NotExitBankAccount)?
            }

            // check the account is already authority ?
            if AuthorizationList::<T>::contains_key(who.clone()) {
                let list = AuthorizationList::<T>::get(who.clone()).unwrap();
                if list.contains(&authority_account.clone()) {
                    Err(Error::<T>::AlreayAuthority)?
                }
            }

            // create the map about auth_id and user
            if AuthorizationList::<T>::contains_key(who.clone()) {
                // get the list
                let mut list = AuthorizationList::<T>::get(who.clone()).unwrap();
                list.push(authority_account.clone());
            } else {
                // create the list
                let mut list = Vec::new();
                list.push(authority_account.clone());
                AuthorizationList::<T>::insert(who.clone(), list);
            }

            Self::deposit_event(Event::AuthoritySuccess(who.clone(), authority_account.clone()));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn charge_user_account (
            account_id: OriginFor<T>,
            money: BalanceOf<T>,
        ) -> DispatchResult {

            // Signed?
            let user = ensure_signed(account_id)?;

            // Determine the User has bank account
            if !BankAccounts::<T>::contains_key(user.clone()) {
                return Err(Error::<T>::NotExitBankAccount.into());
            }

            // create the rand

            // transfer accountid token to staking pot
            T::Currency::transfer(
                &user.clone(),
                &Self::bank_pool1(),
                money,
                ExistenceRequirement::KeepAlive,
            )?;

            // updata the bank account information
            let mut bank_account_info = BankAccounts::<T>::get(user.clone()).unwrap();
            bank_account_info.charge_account(T::BalanceToNumber::convert(money));
            BankAccounts::<T>::insert(user.clone(), bank_account_info);
            // Send the event that user charge amount success
            Self::deposit_event(Event::ChargeAmountSuccess(user.clone(), T::BalanceToNumber::convert(money)));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn withdraw (
            account_id: OriginFor<T>,
            money: BalanceOf<T>,
        ) -> DispatchResult {

            let who = ensure_signed(account_id)?;

            if !BankAccounts::<T>::contains_key(who.clone()) {
                return Err(Error::<T>::NotExitBankAccount.into());
            }

            let mut account_info = BankAccounts::<T>::get(who.clone()).unwrap();

            if account_info.withdraw_amount(T::BalanceToNumber::convert(money)) == false {
                return Err(Error::<T>::NotEnoughMoney.into());
            }

            // rand  hash -> (1,2,3,4,5)

            // transfer accountid token to staking pot
            T::Currency::transfer(
                &Self::bank_pool1(),
                &who.clone(),
                money,
                ExistenceRequirement::KeepAlive,
            )?;

            Self::deposit_event(Event::WithdrawSuccess(who.clone(), T::BalanceToNumber::convert(money)));
            Ok(())
        }

        /// apply for trading
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn apply_trade (
            account_id: OriginFor<T>,
            source_id: T::AccountId,
            money: BalanceOf<T>,
        ) -> DispatchResult {

            let who = ensure_signed(account_id)?;

            let withdraw_info = WithdrawInfo::new(
                who.clone(),
                T::BalanceToNumber::convert(money),
            );

            if WithdrawMoneyList::<T>::contains_key(source_id.clone()) {
                let mut list = WithdrawMoneyList::<T>::get(source_id.clone()).unwrap();
                list.push(withdraw_info);
                WithdrawMoneyList::<T>::insert(source_id.clone(), list);
            } else {
                let mut list = Vec::new();
                list.push(withdraw_info);
                WithdrawMoneyList::<T>::insert(source_id.clone(), list);
            }

            Self::deposit_event(Event::ApplyTradeSuccess(who.clone(), source_id.clone(), money));
            Ok(())
        }


        /// Approval of transactions
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn approval_transactions (
            account_id: OriginFor<T>,
        ) -> DispatchResult {

            let who = ensure_signed(account_id)?;

            // Determine who has transactions list
            if !WithdrawMoneyList::<T>::contains_key(who.clone()) {
                Err(Error::<T>::NotExitTransactionList)?
            }

            // get the apply list
            let list = WithdrawMoneyList::<T>::get(who.clone()).unwrap();

            for info in list {
                let account = info.who;
                let money = info.money;

                Self::transaction(account.clone(), T::NumberToBalance::convert(money));
                Self::deposit_event(Event::TradeSuccess(account.clone(), T::NumberToBalance::convert(money)));
            }

            Ok(())
        }


        /// charge all the bank storage pot
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn charge_bank_pot (
            account_id: OriginFor<T>,
        ) -> DispatchResult {

            let who = ensure_signed(account_id)?;

            T::Currency::transfer(
                &who.clone(),
                &Self::bank_pool1(),
                T::NumberToBalance::convert(300_000_000_000_000),
                ExistenceRequirement::KeepAlive,
            )?;

            T::Currency::transfer(
                &who.clone(),
                &Self::bank_pool2(),
                T::NumberToBalance::convert(300_000_000_000_000),
                ExistenceRequirement::KeepAlive,
            )?;

            T::Currency::transfer(
                &who.clone(),
                &Self::bank_pool3(),
                T::NumberToBalance::convert(300_000_000_000_000),
                ExistenceRequirement::KeepAlive,
            )?;

            T::Currency::transfer(
                &who.clone(),
                &Self::bank_pool4(),
                T::NumberToBalance::convert(300_000_000_000_000),
                ExistenceRequirement::KeepAlive,
            )?;

            T::Currency::transfer(
                &who.clone(),
                &Self::bank_pool5(),
                T::NumberToBalance::convert(300_000_000_000_000),
                ExistenceRequirement::KeepAlive,
            )?;

            Ok(())
        }


    }
}

impl<T: Config> Pallet<T> {
    /// StoragePod
    /// The bank storage, used to storage the user money
    pub fn bank_pool1() -> T::AccountId { PALLET_ID1.into_sub_account(b"bank") }
    pub fn bank_pool2() -> T::AccountId { PALLET_ID2.into_sub_account(b"bank") }
    pub fn bank_pool3() -> T::AccountId { PALLET_ID3.into_sub_account(b"bank") }
    pub fn bank_pool4() -> T::AccountId { PALLET_ID4.into_sub_account(b"bank") }
    pub fn bank_pool5() -> T::AccountId { PALLET_ID5.into_sub_account(b"bank") }

    /// create the rand
    pub fn create_rand() -> u8 {
        let (output, block_num) = T::BankRandomness::random_seed();
        let rand = output.as_ref()[0].rem_euclid(5 as u8);
        rand
    }

    /// use the rand to choose which account to transfer
    pub fn rand_bank_account(rand: u8) -> T::AccountId {
        match rand {
            0 => {
                Self::bank_pool1()
            },
            1 => {
                Self::bank_pool2()
            },
            2 => {
                Self::bank_pool3()
            },
            3 => {
                Self::bank_pool4()
            },
            4 => {
                Self::bank_pool5()
            },

            _ => {
                Self::bank_pool5()
            }
        }
    }

    pub fn transaction(who: T::AccountId, money: BalanceOf<T>) {

    }

    fn check_signed_valid(public_id: Public, signature: &[u8], msg: &[u8]) -> bool {
        let signature = Signature::try_from(signature);
        let signature = signature.unwrap();

        let multi_sig = MultiSignature::from(signature); // OK
        let multi_signer = MultiSigner::from(public_id);
        multi_sig.verify(msg, &multi_signer.into_account())
    }



}


