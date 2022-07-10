#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{dispatch::DispatchResult,
                    pallet_prelude::*, PalletId, traits::{Currency, ExistenceRequirement},};
use frame_support::sp_runtime::app_crypto::TryFrom;
use frame_support::sp_runtime::traits::{IdentifyAccount, Verify};
use frame_support::sp_runtime::MultiSignature;
use frame_support::sp_runtime::MultiSigner;
use frame_support::sp_runtime::traits::Convert;
use frame_support::traits::UnixTime;
use frame_system::pallet_prelude::*;
use sp_core::{Bytes, ecdsa};
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::Zero;
use sp_std::convert::TryInto;

use sp_core::crypto::Pair;




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

const PALLET_ID: PalletId = PalletId(*b"radcbank");
const BALANCE_UNIT: Balance = 1_000_000_000_000;

#[frame_support::pallet]
pub mod pallet {
    use log::log;
    // use serde::de::Unexpected::Str;
    use sp_application_crypto::RuntimePublic;

    // use sp_core::Pair;
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
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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

        // User Registration
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_user_account(
            account_id: OriginFor<T>,
        ) -> DispatchResult {
            // Signed?
            let user = ensure_signed(account_id)?;
            // Crate the bank account
            let bank_account = UserInfo {
                userid: user.clone(),
                amount: 0,
                user_key: 0,
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
            let who = ensure_signed(account_id)?;

            // check the account is already authority ?
            if AuthorizationList::<T>::contains_key(who.clone()) {
                let list = AuthorizationList::<T>::get(who.clone()).unwrap();
                if list.contains(&authority_account) {
                    Err(Error::<T>::AlreayAuthority)?
                }
            }

            // create the map about auth_id and user
            if AuthorizationList::<T>::contains_key(who.clone()) {
                // get the list
                let mut list = AuthorizationList::<T>::get(who.clone()).unwrap();
                list.push(authority_account);
            } else {
                // create the list
                let list = vec![authority_account];
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

            // transfer accountid token to staking pot
            T::Currency::transfer(
                &user.clone(),
                &Self::bank_pool(),
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

            // transfer accountid token to staking pot
            T::Currency::transfer(
                &Self::bank_pool(),
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
                let mut list = WithdrawMoneyList::<T>::get(source_id).unwrap();
                list.push(withdraw_info);
                WithdrawMoneyList::<T>::insert(source_id.clone(), list);
            } else {
                let list = vec![withdraw_info];
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

        /// Test signed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn reveal_puzzle (
            account_id: OriginFor<T>,
            answer_hash: Vec<u8>,
            answer_signed: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(account_id)?;
            // code the who to type: u8
            let mut encode_data = who.encode();

            // make the code_data len to be 33
            encode_data.push(20);
            // check the code_data len is 33 ?
            assert_eq!(33, encode_data.len());

            // let raw_data = encode_data.try_into();
            // change the raw_data to [u8; 33]
            let raw_data: Result<[u8;33], Vec<u8>> = encode_data.try_into();
            let mut raw_data = raw_data.unwrap();

            // crate the 25519 public key
            // let public = sp_core::sr25519::Public::from_raw(raw_data);

            // Crate the ecdsa public key
            let public = ecdsa::Public::from_raw(raw_data);

            // check the signed is valid
            Self::check_signed_valid_ecdsa(
                public,
                answer_signed.as_slice(),
                answer_hash.as_slice(),
            );

            Ok(())
        }

        /// test singed
        /// * phrase 助记词
        /// * password
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn signed (
            origin: OriginFor<T>,
            phrase: sp_std::vec::Vec<u8>,
            password: Vec<u8>,
        ) -> DispatchResult {

            let who = ensure_signed(origin)?;

            // 1. 通过种子和密码生成私钥
            let phrase_str = String::from_utf8(phrase).unwrap();

            let (p, s) = Pair::from_phrase(
                phrase_str.as_str(),
                None,
            ).unwrap();

            // // 2. 使用私钥对msg签名
            //
            // let signature = pair.sign("1234".as_bytes());
            //
            // // 3. 获取公钥
            // let account = pair.public().into_account();
            // // 4. 判断是否能够使用  公钥进行签名
            // if account.verify(&"1234".as_bytes(), &signature) {
            //     Self::deposit_event(Event::Yes);
            // } else {
            //     Self::deposit_event(Event::No);
            // }

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// StoragePod
    /// The bank storage, used to storage the user money
    pub fn bank_pool() -> T::AccountId { PALLET_ID.into_sub_account(b"bank") }

    pub fn transaction(who: T::AccountId, money: BalanceOf<T>) {

    }

    // pub fn check_singed_valid(public_id: Public, signature: &[u8], msg: &[u8]) -> bool {
    //     // get the signature
    //     let signature = ecdsa::Signature::try_from(signature).unwrap();
    //
    //
    //     let multi_signer = MultiSigner::from(public_id);
    //
    //
    // }

    // test the ecdsa
    // verity the ecdsa signed
    pub fn check_signed_valid_ecdsa(publid_id: ecdsa::Public, signature: &[u8], msg: &[u8]) -> bool {

        // signature
        let signature = ecdsa::Signature::try_from(signature).unwrap();

        // Determine tha signature is the type MultiSignature
        let multi_signature = MultiSignature::from(signature);

        // public key
        let multi_signer = MultiSigner::from(publid_id);

        // check the signed
        multi_signature.verify(msg, &multi_signer.into_account())
    }
}


