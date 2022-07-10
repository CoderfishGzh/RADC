use codec::{Decode, Encode};
use frame_support::Parameter;
use frame_system::Account;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::Bytes;
use sp_debug_derive::RuntimeDebug;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;

use crate::{AccountId, Balance, EraIndex};

/// StakingAmount： Pledge account number for market
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct UserInfo<AccountId> {
    // User id
    pub userid: AccountId,
    /// All amounts in the bank of this User
    pub amount: u128,
    /// User key
    pub user_key: u128,
}

impl<AccountId> UserInfo<AccountId> {

    pub fn charge_account(&mut self, amount: u128) {
        self.amount += amount;
    }

    pub fn withdraw_amount(&mut self, amount: u128) -> bool {

        if amount > self.amount {
            return false;
        }

        self.amount -= amount;

        true
    }
}


#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct WithdrawInfo<AccountId> {
    pub who: AccountId,
    pub money: u128,
}

impl<AccountId> WithdrawInfo<AccountId> {
    pub fn new(who: AccountId, money: u128) -> Self {
        Self {
            who,
            money,
        }
    }
}