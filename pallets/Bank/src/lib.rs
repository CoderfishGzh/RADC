#![cfg_attr(not(feature = "std"), no_std)]


use frame_support::{PalletId};

use frame_support::traits::{UnixTime, Currency};
use frame_system::pallet_prelude::*;
use sp_core::Bytes;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::Zero;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;

pub use pallet::*;

const PALLET_ID: PalletId = PalletId(*b"RADCGZH!");

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

  #[frame_support::pallet]
  pub mod pallet {
      use frame_support::pallet_prelude::*;
      

      // The struct on which we build all of our Pallet logic.
      #[pallet::pallet]
      #[pallet::generate_store(pub(super) trait Store)]
      pub struct Pallet<T>(_);

      /* Placeholder for defining custom types. */

      // TODO: Update the `config` block below
      #[pallet::config]
      pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        // currency to pay fees and hold balances
        type Currency: frame_support::traits::Currency<Self::AccountId> ;
        
      }

      // TODO: Update the `event` block below
      #[pallet::event]
      #[pallet::generate_deposit(pub(super) fn deposit_event)]
      pub enum Event<T: Config> {
          WithDrawSuccess(T::AccountId, u128),

      }

      // TODO: Update the `error` block below
      #[pallet::error]
      pub enum Error<T> {
          AccountNotExit,
      }

      // TODO: add #[pallet::storage] block

      // TODO: Update the `call` block below
      #[pallet::call]
      impl<T: Config> Pallet<T> {}
  }


impl<T: Config> Pallet<T> {
    pub fn storage_pot() -> T::AccountId { PALLET_ID.into_sub_account(b"stor") }
}