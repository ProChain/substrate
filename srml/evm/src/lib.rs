// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! EVM execution module for Substrate

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod executor;

pub use crate::executor::{Account, Log};

use support::{dispatch::Result, decl_module, decl_storage, decl_event};
use support::traits::{Currency, WithdrawReason, ExistenceRequirement};
use system::ensure_signed;
use sr_primitives::weights::SimpleDispatchInfo;
use sr_primitives::traits::UniqueSaturatedInto;
use primitives::{U256, H256, H160};

pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait ConvertAccountId<A> {
	fn convert_account_id(account_id: A) -> H160;
}

/// EVM module trait
pub trait Trait: system::Trait {
	/// Convert account ID to H160;
	type ConvertAccountId: ConvertAccountId<Self::AccountId>;
	/// Currency type for deposit and withdraw.
	type Currency: Currency<Self::AccountId>;
	/// The overarching event type.
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Example {
		Accounts get(fn accounts) config(): map H160 => Account;
	}
}

decl_event!(
	/// EVM events
	pub enum Event {
		/// Ethereum events from contracts.
		Log(Log),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = SimpleDispatchInfo::FixedNormal(10_000)]
		fn deposit_balance(origin, value: BalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;

			T::Currency::withdraw(
				&sender,
				value,
				WithdrawReason::Reserve,
				ExistenceRequirement::KeepAlive,
			)?;

			let bvalue = U256::from(UniqueSaturatedInto::<u64>::unique_saturated_into(value));
			let address = T::ConvertAccountId::convert_account_id(sender);
			Accounts::mutate(&address, |account| {
				account.balance += bvalue;
			});

			Ok(())
		}

		#[weight = SimpleDispatchInfo::FixedNormal(10_000)]
		fn withdraw_balance(origin, value: BalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;
			let address = T::ConvertAccountId::convert_account_id(sender.clone());
			let bvalue = U256::from(UniqueSaturatedInto::<u64>::unique_saturated_into(value));

			if Accounts::get(&address).balance < bvalue {
				return Err("Not enough balance")
			}

			Accounts::mutate(&address, |account| {
				account.balance -= bvalue;
			});

			T::Currency::deposit_creating(&sender, value);

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {

}
