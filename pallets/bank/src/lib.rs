#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{dispatch::DispatchResult,
                    pallet_prelude::*, PalletId, traits::{Currency, ExistenceRequirement}};
use frame_support::sp_runtime::traits::Convert;
use frame_support::traits::UnixTime;
use frame_system::pallet_prelude::*;
use sp_core::Bytes;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::Zero;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;
pub use primitives::p_bank::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const PALLET_ID: PalletId = PalletId(*b"ttchain!");


#[frame_support::pallet]
pub mod pallet {
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

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {

        RegisterSuccess(T::AccountId),

        ChargeAmontSuccess(T::AccountId, u128),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {

        NotExitBankAccount,
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
                &Self::order_pool(),
                money,
                ExistenceRequirement::KeepAlive,
            )?;

            // updata the bank account information
            let mut bank_account_info = BankAccounts::<T>::get(user.clone()).unwrap();
            bank_account_info.charge_account(T::BalanceToNumber::convert(money));
            BankAccounts::<T>::insert(user.clone(), bank_account_info);
            // Send the event that user charge amount success
            Self::deposit_event(Event::ChargeAmontSuccess(user.clone(), T::BalanceToNumber::convert(money)));
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// StoragePod
    /// The bank storage, used to storage the user money
    pub fn order_pool() -> T::AccountId { PALLET_ID.into_sub_account(b"order") }
}


