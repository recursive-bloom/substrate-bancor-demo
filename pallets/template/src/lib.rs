#![cfg_attr(not(feature = "std"), no_std)]

/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

/// https://github.com/bifrost-finance/bifrost/blob/develop/brml/swap/src/lib.rs

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	use core::convert::{From, Into, TryInto};
	use core::ops::Div;
	use fixed_point::{
		traits::FromFixed,
		transcendental,
		types::{extra, *},
		FixedI128,
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn basesupplyoftoken)]
	pub type BaseSupplyOfToken<T> = StorageValue<_, u128>;

	#[pallet::storage]
	#[pallet::getter(fn basebalanceofvstoken)]
	pub type BaseBalanceOfVSToken<T> = StorageValue<_, u128>;

	#[pallet::storage]
	#[pallet::getter(fn realupplyoftoken)]
	pub type RealSupplyOfToken<T> = StorageValue<_, u128>;

	#[pallet::storage]
	#[pallet::getter(fn realbalanceofvstoken)]
	pub type RealBalanceOfVSToken<T> = StorageValue<_, u128>;

	#[pallet::storage]
	#[pallet::getter(fn tokensheet)]
	pub type TokenSheet<T: Config> = StorageMap<_, Blake2_128Concat,  T::AccountId, u128, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		BancorInit(u128, u128, T::AccountId),
		VsTokenToToken(u128, u128, T::AccountId),
		TokenToVsToken(u128, u128, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn base_init(origin: OriginFor<T>, base_balance: u128) -> DispatchResultWithPostInfo {
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;
			match <BaseSupplyOfToken<T>>::get() {
				// Return an error if the value has not been set.
				None => {
					// Update storage.
					let base_supply = base_balance.saturating_mul(2);
					<BaseSupplyOfToken<T>>::put(base_supply);
					<BaseBalanceOfVSToken<T>>::put(base_balance);
					<RealSupplyOfToken<T>>::put(0);
					<RealBalanceOfVSToken<T>>::put(0);

					// Emit an event.
					Self::deposit_event(Event::BancorInit(base_supply, base_balance, who.clone()));
				},
				Some(_) => { },
			}
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn vstoken_buy_token(origin: OriginFor<T>, vstoken: u128) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let saved_vstoken = vstoken;
			let base_supply = <BaseSupplyOfToken<T>>::get().unwrap();
			let base_balance = <BaseBalanceOfVSToken<T>>::get().unwrap();
			let real_supply = <RealSupplyOfToken<T>>::get().unwrap();
			let real_balance = <RealBalanceOfVSToken<T>>::get().unwrap();
			let virtual_supply = base_supply.saturating_add(real_supply);
			let virtual_balance = base_balance.saturating_add(real_balance);

			let vs = I64F64::from_num(virtual_supply);
			let vb = I64F64::from_num(virtual_balance);
			let vstoken = I64F64::from_num(vstoken);

			let m = vstoken.saturating_div(vb);
			let m = FixedI128::<extra::U64>::from_num(1).saturating_add(m);
			let m : FixedI128<extra::U64> = transcendental::sqrt(m).unwrap();
			let m = m.saturating_sub(FixedI128::<extra::U64>::from_num(1));
			let token = m.saturating_mul(vs);

			let token = u128::from_fixed(token);

			let real_supply = real_supply.saturating_add(token);
			let real_balance = real_balance.saturating_add(saved_vstoken);

			<TokenSheet<T>>::insert(who.clone(), token);
			<RealBalanceOfVSToken<T>>::put(real_balance);
			<RealSupplyOfToken<T>>::put(real_supply);

			// Emit an event.
			Self::deposit_event(Event::VsTokenToToken(saved_vstoken, token, who.clone()));

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn token_buy_vstoken(origin: OriginFor<T>, token: u128) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let saved_token = token;
			let base_supply = <BaseSupplyOfToken<T>>::get().unwrap();
			let base_balance = <BaseBalanceOfVSToken<T>>::get().unwrap();
			let real_supply = <RealSupplyOfToken<T>>::get().unwrap();
			let real_balance = <RealBalanceOfVSToken<T>>::get().unwrap();
			let virtual_supply = base_supply.saturating_add(real_supply);
			let virtual_balance = base_balance.saturating_add(real_balance);

			let vs = I64F64::from_num(virtual_supply);
			let vb = I64F64::from_num(virtual_balance);
			let token = I64F64::from_num(token);

			let m = token.saturating_div(vs);
			let m = FixedI128::<extra::U64>::from_num(1).saturating_sub(m);
			let m = m.saturating_mul(m);
			let m = FixedI128::<extra::U64>::from_num(1).saturating_sub(m);
			let vstoken = m.saturating_mul(vb);

			let vstoken = u128::from_fixed(vstoken);

			let real_supply = real_supply.saturating_sub(saved_token);
			let real_balance = real_balance.saturating_sub(vstoken);

			<TokenSheet<T>>::insert(who.clone(), <TokenSheet<T>>::get(who.clone()).unwrap().saturating_sub(saved_token));
			<RealBalanceOfVSToken<T>>::put(real_balance);
			<RealSupplyOfToken<T>>::put(real_supply);

			// Emit an event.
			Self::deposit_event(Event::TokenToVsToken(saved_token, vstoken, who.clone()));

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}
	}
}



