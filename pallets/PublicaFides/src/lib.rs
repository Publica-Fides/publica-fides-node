#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod helper;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::{DispatchResult, EncodeLike},
		pallet_prelude::*,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, One};
	use substrate_fixed::types::U32F32;
	pub use crate::helper::score_claims;
	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Id of content stored in the system
		type ContentId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
	}
	
	/// Id of claims made in the system.
	type ClaimId = u32;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	
	
	#[derive(Encode, Decode, Default, Clone, Eq, PartialEq, RuntimeDebug)]
	/// Represents content in the system.
	pub struct Content {
		/// The URL designated for accessing the Content
		url: Vec<u8>,
		/// Claims raised in the Content and their vote result. Max 10
		claims: Vec<Claim>,
		/// Number in range of 0-1 representing the calculated score for each piece of content
		score: U32F32
	}

	#[pallet::storage]
	#[pallet::getter(fn get_content)]
	pub type ContentStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ContentId, Content, ValueQuery>;

	/// Next available class ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextContentId<T: Config> = StorageValue<_, T::ContentId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_claim_id)]
	pub type NextClaimId<T: Config> = StorageValue<_, ClaimId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_resolved_claim_id)]
	pub type NextResolvedClaimID<T: Config> = StorageValue<_, ClaimId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_final_claims)]
	pub type FinalClaimStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ContentId, ResolvedClaims, ValueQuery>;

	#[derive(Encode, Decode, Default, Clone, Eq, PartialEq, RuntimeDebug)]
	/// Claims made in proposed content. Proposers can introduce claims as accepted or rejected to reflect the veracity of the content.
	pub struct Claim {
		/// the IPFS CID of the text that contains the objective claim statement.
		pub claim_text_cid: Vec<u8>,
		/// Whether the claim is determined to be accepted or rejected by the Collective instance. Using an affirmative for wider understanding..
		pub is_accepted: bool,
	}

	#[derive(Encode, Decode, Default, Clone, Eq, PartialEq, RuntimeDebug)]
	// Claims that have been verified as objective and judged to be true or false
	pub struct ResolvedClaims {
		pub claims: Vec<Claim>
	}

	#[pallet::storage]
	#[pallet::getter(fn get_claims)]
	// Double Storage map that maps claims to the articles they originated from
	pub type ClaimsToContent<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ClaimId,
		Blake2_128Concat,
		T::ContentId,
		Claim,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ContentStored(T::ContentId),
		ClaimStored(ClaimId),
		ScoreStored(u8),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoAvailableContentId,
		NoAvailableClaimId,
		NonExistentContent
	}

pub fn truth_from_content<T: Config>(content_id: T::ContentId) {
	// Get mutable stored content by its id
	ContentStorage::<T>::try_mutate_exists(content_id, |query_result| -> DispatchResult {
		let content = query_result.as_mut().ok_or(Error::<T>::NonExistentContent).unwrap();
		// get claims for the given piece of content
		let claims = FinalClaimStorage::<T>::get(content_id);
		// update score of that piece of content with the score
		content.score = score_claims(claims);
		// Todo: decide whether we want to send an event i.e. alert the sender about the result
		// Pallet::<T>::deposit_event(Event::ScoreStored(yourscorehere));
		Ok(())
	});
}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		/// Stores an article in the system to initiate the claims-voting process
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the request.
		/// * `url` - Url of the article. Displayed for the purpose of allowing voters to find and read the content.
		pub fn store_content(
			origin: OriginFor<T>,
			url: Vec<u8>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let class_id =
				NextContentId::<T>::try_mutate(|id| -> Result<T::ContentId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableContentId)?;
					Ok(current_id)
				})?;

			let content = Content { url, claims: [].to_vec(), score: U32F32::from_num(0) };
			ContentStorage::<T>::insert(class_id.clone(), content);
			Self::deposit_event(Event::ContentStored(class_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		/// Stores a claim for given content. This should be called by the collective propose extrinsic.
		///
		/// # Arguments
		///
		/// * `origin` - Origin of the request
		/// * `claim_statement` - IPFS CID of a stored string that contains an objective claim. This claim will be voted on for veracity.
		/// * `content_id` - Id of the content the claim was discovered in.
		/// * `is_accepted` - Designates whether the claim is accepted, or rejected, by the council.
		pub fn store_claim_for_content(
			origin: OriginFor<T>,
			claim_statement: Vec<u8>,
			content_id: T::ContentId,
			is_accepted: bool,
		) -> DispatchResult {
			// TODO: Find way to ensure this was called by the `propose` extrinsic. This fails with BadOrigin when called by the `propose` pallet.
			ensure_signed(origin)?;
			// Ensure that the article exists
			ensure!(ContentStorage::<T>::contains_key(content_id), Error::<T>::NonExistentContent);

			let new_claim_id =
				NextClaimId::<T>::try_mutate(|claim_id| -> Result<ClaimId, DispatchError> {
					let current_id = *claim_id;
					*claim_id =
						claim_id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableClaimId)?;
					Ok(current_id)
				})?;
			
			let new_resolved_claim_id =
				NextResolvedClaimID::<T>::try_mutate(|claim_id| -> Result<ClaimId, DispatchError> {
					let current_id = *claim_id;
					*claim_id =
						claim_id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableClaimId)?;
					Ok(current_id)
				})?;

			let this_claim = Claim { claim_text_cid: claim_statement, is_accepted };

			ClaimsToContent::<T>::insert(
				new_claim_id,
				content_id.clone(),
                this_claim.clone(),
			);

			ContentStorage::<T>::try_mutate_exists(content_id.clone(), |val| -> DispatchResult {
				// add claim id to content for future reference
				let content = val.as_mut().ok_or(Error::<T>::NonExistentContent).unwrap();
				content.claims.push(this_claim);
				Self::deposit_event(Event::ClaimStored(new_claim_id));
				Ok(())
			});

			Self::deposit_event(Event::ClaimStored(new_claim_id));
			Ok(())
		}
	}
}
