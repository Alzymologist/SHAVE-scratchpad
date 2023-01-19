#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{inherent::Vec, parameter_types, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

parameter_types! {
	pub MaxDoiSize: u32 = 128;
	pub MaxTagSize: u32 = 64;
	pub MaxAuthorCount: u32 = 64;
}

///Relation between two items
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub enum Relation {
	Reference,
	Superseed,
}

///Opinion relation (1 entity)
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub enum Opinion {
	Endorse,
	Thwart,
}

///Struct to store tags
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct TagRecord {
	name: BoundedVec<u8, MaxTagSize>,
}

/// DOI address
pub type Doi = BoundedVec<u8, MaxDoiSize>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
/// Generic record
pub enum Record<T: Config> {
	Doi(Doi),
	Ipfs(T::Hash),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
#[scale_info(skip_type_params(T))]
pub struct Authorship<T: Config> {
	author: T::AccountId,
	confirmed: bool,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct RecordContent<T: Config> {
	manager: T::AccountId,
	coauthors: BoundedVec<Authorship<T>, MaxAuthorCount>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn checkout)]
	pub type ScientificRecord<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, RecordContent<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn subjective_rate)]
	pub type OpinionRecord<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		Record<T>,
		Opinion,
		OptionQuery,
	>;

	#[pallet::storage]
	pub type RelationRecord<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, Record<T>>,
			NMapKey<Blake2_128Concat, Record<T>>,
		),
		Relation,
		OptionQuery,
	>;

	#[pallet::storage]
	pub type TagMark<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		TagRecord,
		TagRecord,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		ScientificRecordMade { record: T::Hash, author: T::AccountId },
		/// Someont (who) stated or changed their opinion about doi-identified document.
		/// [record, opinion, author]
		OpinionStatedDoi { record: Doi, opinion: Opinion, author: T::AccountId },
		/// Someont (who) stated or changed their opinion about document in distributed fs.
		/// [record, opinion, author]
		OpinionStatedDfs { record: T::Hash, opinion: Opinion, author: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// This record was already published.
		DuplicateRecord,
		/// Too many authors requested for a publication
		TooManyAuthors,
		/// Attempting to reffer a nonexistent record to something
		RecordNotRegistered,
		/// Attempting to manage not owned record
		RecordAccessDenied,
		/// Attempting to make a record that already exists
		DuplicateRelationRecord,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// React to published article backed by authority publisher with DOI
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn react_to_doi(origin: OriginFor<T>, doi: Doi, opinion: Opinion) -> DispatchResult {
			let who = ensure_signed(origin)?;

			OpinionRecord::<T>::insert(who.clone(), Record::<T>::Doi(doi.clone()), opinion.clone());

			Self::deposit_event(Event::OpinionStatedDoi { record: doi, opinion, author: who });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn react(origin: OriginFor<T>, id: T::Hash, opinion: Opinion) -> DispatchResult {
			let who = ensure_signed(origin)?;

			OpinionRecord::<T>::insert(who.clone(), Record::<T>::Ipfs(id.clone()), opinion.clone());

			Self::deposit_event(Event::OpinionStatedDfs { record: id, opinion, author: who });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn post(
			origin: OriginFor<T>,
			id: T::Hash,
			authors: BoundedVec<T::AccountId, MaxAuthorCount>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!ScientificRecord::<T>::contains_key(id), Error::<T>::DuplicateRecord,);

			let coauthors = authors
				.iter()
				.map(|a| Authorship::<T> { author: a.clone(), confirmed: false })
				.collect::<Vec<Authorship<T>>>()
				.try_into()
				.or(Err(Error::<T>::TooManyAuthors))?;

			ScientificRecord::<T>::insert(
				id.clone(),
				RecordContent::<T> { manager: who.clone(), coauthors },
			);

			Self::deposit_event(Event::ScientificRecordMade { record: id, author: who });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn reffer_to_doi(origin: OriginFor<T>, newer: T::Hash, older: Doi) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(ScientificRecord::<T>::contains_key(newer), Error::<T>::RecordNotRegistered);

			let record = Self::checkout(newer).unwrap();

			let older_record = Record::<T>::Doi(older);

			ensure!(record.manager == who, Error::<T>::RecordAccessDenied);

			ensure!(
				RelationRecord::<T>::contains_key((&who, Record::<T>::Ipfs(newer), &older_record)),
				Error::<T>::DuplicateRelationRecord
			);

			RelationRecord::<T>::insert(
				(who, Record::<T>::Ipfs(newer), older_record),
				Relation::Reference,
			);

			Ok(())
		}
		/*
		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
				*/
	}
}
