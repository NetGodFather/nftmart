#![cfg_attr(not(feature = "std"), no_std)]

use enumflags2::BitFlags;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency, ExistenceRequirement::KeepAlive},
	transactional, dispatch::DispatchResult
};
use sp_std::vec::Vec;
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
pub use sp_core::constants_types::{Balance, ACCURACY, NATIVE_CURRENCY_ID};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{CheckedAdd, Bounded, CheckedSub,
			 AccountIdConversion, StaticLookup, Zero, One, AtLeast32BitUnsigned},
	ModuleId, RuntimeDebug, SaturatedConversion,
};
use codec::FullCodec;

// mod mock;
// mod tests;

pub use module::*;

#[repr(u8)]
#[derive(Encode, Decode, Clone, Copy, BitFlags, RuntimeDebug, PartialEq, Eq)]
pub enum ClassProperty {
	/// Token can be transferred
	Transferable = 0b00000001,
	/// Token can be burned
	Burnable = 0b00000010,
	/// Need to charge royalties when orders are completed.
	RoyaltiesChargeable = 0b00000100,
}

#[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Properties(pub BitFlags<ClassProperty>);

impl Eq for Properties {}
impl Encode for Properties {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl Decode for Properties {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u8::decode(input)?;
		Ok(Self(
			<BitFlags<ClassProperty>>::from_bits(field as u8).map_err(|_| "invalid value")?,
		))
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData<BlockNumber> {
	/// The minimum balance to create class
	#[codec(compact)]
	pub deposit: Balance,
	/// Property of all tokens in this class.
	pub properties: Properties,
	/// Name of class.
	pub name: Vec<u8>,
	/// Description of class.
	pub description: Vec<u8>,
	#[codec(compact)]
	pub create_block: BlockNumber,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData<AccountId, BlockNumber> {
	/// The minimum balance to create token
	#[codec(compact)]
	pub deposit: Balance,
	#[codec(compact)]
	pub create_block: BlockNumber,
	/// Charge royalty
	pub royalty: bool,
	/// The token's creator
	pub creator: AccountId,
	/// Royalty beneficiary
	pub royalty_beneficiary: AccountId,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CategoryData {
	/// The category metadata.
	pub metadata: NFTMetadata,
	/// The number of NFTs in this category.
	#[codec(compact)]
	pub nft_count: Balance,
}

pub type NFTMetadata = Vec<u8>;
pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
pub type CategoryIdOf<T> = <T as Config>::CategoryId;
pub type BalanceOf<T> = <<T as module::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CurrencyIdOf<T> = <<T as module::Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
	V1_0_0,
	V2_0_0,
}

impl Default for Releases {
	fn default() -> Self {
		Releases::V2_0_0
	}
}

pub mod migrations {
	use super::*;

	#[derive(Decode)]
	pub struct OldClassData {
		#[codec(compact)]
		pub deposit: Balance,
		pub properties: Properties,
		pub name: Vec<u8>,
		pub description: Vec<u8>,
	}

	#[derive(Decode)]
	pub struct OldTokenData {
		#[codec(compact)]
		pub deposit: Balance,
	}

	impl OldClassData {
		fn upgraded<T>(self) -> ClassData<T> where T: AtLeast32BitUnsigned + Bounded + Copy + From<u32> {
			let create_block: T = One::one();
			ClassData {
				create_block: create_block * 2u32.into(),
				deposit: self.deposit,
				properties: self.properties,
				name: self.name,
				description: self.description,
			}
		}
	}

	impl OldTokenData {
		fn upgraded<AccountId: Clone, T>(self, who: AccountId) -> TokenData<AccountId, T> where T: AtLeast32BitUnsigned + Bounded + Copy + From<u32> {
			let create_block: T = One::one();
			TokenData {
				create_block: create_block * 3u32.into(),
				deposit: self.deposit,
				royalty: false,
				creator: who.clone(),
				royalty_beneficiary: who,
			}
		}
	}

	pub fn do_migrate<T: Config>() -> Weight {
		// type OldClass<T> = orml_nft::ClassInfo<TokenIdOf<T>, <T as frame_system::Config>::AccountId, OldClassData>;
		// type NewClass<T> = orml_nft::ClassInfo<TokenIdOf<T>, <T as frame_system::Config>::AccountId, ClassData<BlockNumberOf<T>>>;
		// orml_nft::Classes::<T>::translate::<OldClass<T>, _>(|_, p: OldClass<T>| {
		// 	let new_data: NewClass<T> = NewClass::<T> {
		// 		 metadata: p.metadata,
		// 		 total_issuance: p.total_issuance,
		// 		 owner: p.owner,
		// 		 data: p.data.upgraded::<BlockNumberOf<T>>(),
		// 	};
		// 	Some(new_data)
		// });
		// type OldToken<T> = orml_nft::TokenInfo<<T as frame_system::Config>::AccountId, OldTokenData>;
		// type NewToken<T> = orml_nft::TokenInfo<<T as frame_system::Config>::AccountId, TokenData<<T as frame_system::Config>::AccountId, BlockNumberOf<T>>>;
		// orml_nft::Tokens::<T>::translate::<OldToken<T>, _>(|_, _, p: OldToken<T>| {
		// 	let new_data: NewToken<T> = NewToken::<T> {
		// 		metadata: p.metadata,
		// 		owner: p.owner.clone(),
		// 		data: p.data.upgraded::<<T as frame_system::Config>::AccountId, BlockNumberOf<T>>(p.owner),
		// 	};
		// 	Some(new_data)
		// });
		T::BlockWeights::get().max_block
	}
}

#[frame_support::pallet]
pub mod module {
	use super::*;
	use orml_nft::{TokenInfoOf, ClassInfoOf};
	use sp_runtime::{PerU16};

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config<ClassData = ClassData<BlockNumberOf<Self>>, TokenData = TokenData<<Self as frame_system::Config>::AccountId, BlockNumberOf<Self>>> + pallet_proxy::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The minimum balance to create class
		#[pallet::constant]
		type CreateClassDeposit: Get<Balance>;

		/// The amount of balance that must be deposited per byte of metadata.
		#[pallet::constant]
		type MetaDataByteDeposit: Get<Balance>;

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateTokenDeposit: Get<Balance>;

		/// The NFT's module id
		#[pallet::constant]
		type ModuleId: Get<ModuleId>;

		/// MultiCurrency type for trading
		type MultiCurrency: MultiReservableCurrency<Self::AccountId, Balance = Balance>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The Category ID type
		type CategoryId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded + FullCodec;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// ClassId not found
		ClassIdNotFound,
		/// TokenId not found
		TokenIdNotFound,
		/// Category not found
		CategoryNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Quantity is invalid. need >= 1
		InvalidQuantity,
		/// Invalid deadline
		InvalidDeadline,
		/// Invalid deposit
		InvalidDeposit,
		/// Property of class don't support transfer
		NonTransferable,
		/// Property of class don't support burn
		NonBurnable,
		/// Can not destroy class Total issuance is not 0
		CannotDestroyClass,
		/// No available category ID
		NoAvailableCategoryId,
		/// NameTooLong
		NameTooLong,
		/// DescriptionTooLong
		DescriptionTooLong,
		/// Not supported for now
		NotSupportedForNow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Created NFT class. \[owner, class_id\]
		CreatedClass(T::AccountId, ClassIdOf<T>),
		/// Minted NFT token. \[from, to, class_id, quantity\]
		MintedToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Transferred NFT token. \[from, to, class_id, token_id, quantity\]
		TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>),
		/// Burned NFT token. \[owner, class_id, token_id, quantity, unreserved\]
		BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>, Balance),
		/// Destroyed NFT class. \[owner, class_id, dest\]
		DestroyedClass(T::AccountId, ClassIdOf<T>, T::AccountId),
		/// Created NFT common category. \[category_id\]
		CreatedCategory(CategoryIdOf<T>),
		/// Updated NFT common category. \[category_id\]
		UpdatedCategory(CategoryIdOf<T>),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::<T>::get() == Releases::V1_0_0 {
				StorageVersion::<T>::put(Releases::V2_0_0);
				migrations::do_migrate::<T>()
			} else {
				0
			}
		}

		fn integrity_test () {}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		platform_fee_rate: PerU16,
		_phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				platform_fee_rate: PerU16::from_rational(1u32, 10000u32),
				_phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<StorageVersion<T>>::put(Releases::default());
			PlatformFeeRate::<T>::put(self.platform_fee_rate);
		}
	}

	/// Storage version of the pallet.
	#[pallet::storage]
	pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

	/// Next available common category ID.
	#[pallet::storage]
	#[pallet::getter(fn next_category_id)]
	pub type NextCategoryId<T: Config> = StorageValue<_, T::CategoryId, ValueQuery>;

	/// The storage of categories.
	#[pallet::storage]
	#[pallet::getter(fn categories)]
	pub type Categories<T: Config> = StorageMap<_, Identity, T::CategoryId, CategoryData>;

	/// Royalties rate, which can be set by council or sudo.
	#[pallet::storage]
	#[pallet::getter(fn royalties_rate)]
	pub type RoyaltiesRate<T: Config> = StorageValue<_, PerU16, ValueQuery>;

	/// platform fee rate
	#[pallet::storage]
	#[pallet::getter(fn platform_fee_rate)]
	pub type PlatformFeeRate<T: Config> = StorageValue<_, PerU16, ValueQuery>;

	/// MaxDistributionReward
	#[pallet::storage]
	#[pallet::getter(fn max_distribution_reward)]
	pub type MaxDistributionReward<T: Config> = StorageValue<_, PerU16, ValueQuery>;

	/// MinReferenceDeposit
	#[pallet::storage]
	#[pallet::getter(fn min_reference_deposit)]
	pub type MinReferenceDeposit<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Create a common category for trading NFT.
		/// A Selling NFT should belong to a category.
		///
		/// - `metadata`: metadata
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn create_category(origin: OriginFor<T>, metadata: NFTMetadata) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let category_id = NextCategoryId::<T>::try_mutate(|id| -> Result<T::CategoryId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableCategoryId)?;
				Ok(current_id)
			})?;

			let info = CategoryData {
				metadata,
				nft_count: Zero::zero(),
			};
			Categories::<T>::insert(category_id, info);

			Self::deposit_event(Event::CreatedCategory(category_id));
			Ok(().into())
		}

		/// Update a common category.
		///
		/// - `category_id`: category ID
		/// - `metadata`: metadata
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn update_category(origin: OriginFor<T>, category_id: CategoryIdOf<T>, metadata: NFTMetadata) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			if let Some(category) = Self::categories(category_id) {
				let info = CategoryData {
					metadata,
					nft_count: category.nft_count,
				};
				Categories::<T>::insert(category_id, info);
				Self::deposit_event(Event::UpdatedCategory(category_id));
			}
			Ok(().into())
		}

		/// Create NFT class, tokens belong to the class.
		///
		/// - `metadata`: external metadata
		/// - `properties`: class property, include `Transferable` `Burnable`
		/// - `name`: class name, with len limitation.
		/// - `description`: class description, with len limitation.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn create_class(origin: OriginFor<T>, metadata: NFTMetadata, name: Vec<u8>, description: Vec<u8>, properties: Properties) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(name.len() <= 20, Error::<T>::NameTooLong);// TODO: pass configurations from runtime configuration.
			ensure!(description.len() <= 256, Error::<T>::DescriptionTooLong);// TODO: pass configurations from runtime configuration.

			let next_id = orml_nft::Pallet::<T>::next_class_id();
			let owner: T::AccountId = T::ModuleId::get().into_sub_account(next_id);
			let (deposit, all_deposit) = Self::create_class_deposit(
				metadata.len().saturated_into(),
				name.len().saturated_into(),
				description.len().saturated_into(),
			);

			<T as Config>::Currency::transfer(&who, &owner, all_deposit.saturated_into(), KeepAlive)?;
			<T as Config>::Currency::reserve(&owner, deposit.saturated_into())?;
			// owner add proxy delegate to origin
			<pallet_proxy::Pallet<T>>::add_proxy_delegate(&owner, who, Default::default(), Zero::zero())?;

			let data: ClassData<BlockNumberOf<T>> = ClassData {
				deposit,
				properties,
				name,
				description,
				create_block: <frame_system::Pallet<T>>::block_number(),
			};
			orml_nft::Pallet::<T>::create_class(&owner, metadata, data)?;

			Self::deposit_event(Event::CreatedClass(owner, next_id));
			Ok(().into())
		}

		/// Update token royalty.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn update_token_royalty(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			charge_royalty: Option<bool>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			orml_nft::Tokens::<T>::try_mutate(class_id, token_id, |maybe_token| -> DispatchResultWithPostInfo {
				let token_info: &mut TokenInfoOf<T> = maybe_token.as_mut().ok_or(Error::<T>::TokenIdNotFound)?;
				ensure!(who == token_info.data.royalty_beneficiary, Error::<T>::NoPermission);

				// TODO: Get ride of this limitation.
				ensure!(token_info.quantity == One::one(), Error::<T>::NotSupportedForNow);

				token_info.data.royalty = charge_royalty.ok_or_else(|| -> Result<bool,DispatchError> {
					let class_info: ClassInfoOf<T> = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
					let data: ClassData<T::BlockNumber> = class_info.data;
					Ok(data.properties.0.contains(ClassProperty::RoyaltiesChargeable))
				}).or_else(core::convert::identity)?;
				Ok(().into())
			})
		}

		/// Update token royalty beneficiary.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn update_token_royalty_beneficiary(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			to: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			orml_nft::Tokens::<T>::try_mutate(class_id, token_id, |maybe_token| -> DispatchResultWithPostInfo {
				let token_info: &mut TokenInfoOf<T> = maybe_token.as_mut().ok_or(Error::<T>::TokenIdNotFound)?;
				ensure!(who == token_info.data.royalty_beneficiary, Error::<T>::NoPermission);
				let to = T::Lookup::lookup(to)?;
				token_info.data.royalty_beneficiary = to;
				Ok(().into())
			})
		}

		/// Mint NFT token
		///
		/// - `to`: the token owner's account
		/// - `class_id`: token belong to the class id
		/// - `metadata`: external metadata
		/// - `quantity`: token quantity
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] class_id: ClassIdOf<T>,
			metadata: NFTMetadata,
			#[pallet::compact] quantity: TokenIdOf<T>,
			charge_royalty: Option<bool>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			ensure!(quantity >= One::one(), Error::<T>::InvalidQuantity);
			let class_info: ClassInfoOf<T> = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(who == class_info.owner, Error::<T>::NoPermission);
			let deposit = Self::mint_token_deposit(metadata.len().saturated_into());

			<T as Config>::Currency::reserve(&class_info.owner, deposit.saturated_into())?;
			let data: TokenData<T::AccountId, BlockNumberOf<T>> = TokenData {
				deposit,
				create_block: <frame_system::Pallet<T>>::block_number(),
				royalty: charge_royalty.unwrap_or_else(|| class_info.data.properties.0.contains(ClassProperty::RoyaltiesChargeable)),
				creator: to.clone(),
				royalty_beneficiary: to.clone(),
			};

			// TODO: Get ride of this limitation.
			if quantity > One::one() {
				ensure!(!data.royalty, Error::<T>::NotSupportedForNow);
			}

			orml_nft::Pallet::<T>::mint(&to, class_id, metadata.clone(), data.clone(), quantity)?;

			Self::deposit_event(Event::MintedToken(who, to, class_id, quantity));
			Ok(().into())
		}

		/// Transfer NFT tokens to another account
		///
		/// - `to`: the token owner's account
		/// - `class_id`: class id
		/// - `token_id`: token id
		/// - `quantity`: quantity
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			#[pallet::compact] quantity: TokenIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			ensure!(quantity >= One::one(), Error::<T>::InvalidQuantity);
			Self::do_transfer(&who, &to, class_id, token_id, quantity)?;
			Ok(().into())
		}

		/// Burn NFT token
		///
		/// - `class_id`: class id
		/// - `token_id`: token id
		/// - `quantity`: quantity
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn burn(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			#[pallet::compact] quantity: TokenIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_burnable(class_id)?, Error::<T>::NonBurnable);
			ensure!(quantity >= One::one(), Error::<T>::InvalidQuantity);

			if let Some(token_info) = orml_nft::Pallet::<T>::burn(&who, (class_id, token_id), quantity)? {
				if token_info.quantity.is_zero() {
					let class_owner: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
					let data: TokenData<T::AccountId, T::BlockNumber> = token_info.data;
					// `repatriate_reserved` will check `to` account exist and return `DeadAccount`.
					// `transfer` not do this check.
					<T as Config>::Currency::unreserve(&class_owner, data.deposit.saturated_into());
					<T as Config>::Currency::transfer(&class_owner, &who, data.deposit.saturated_into(), KeepAlive)?;
					Self::deposit_event(Event::BurnedToken(who, class_id, token_id, quantity, data.deposit));
				} else {
					Self::deposit_event(Event::BurnedToken(who, class_id, token_id, quantity, 0));
				}
			}
			Ok(().into())
		}

		/// Destroy NFT class
		///
		/// - `class_id`: destroy class id
		/// - `dest`: transfer reserve balance from sub_account to dest
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn destroy_class(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(who == class_info.owner, Error::<T>::NoPermission);
			ensure!(
				class_info.total_issuance == Zero::zero(),
				Error::<T>::CannotDestroyClass
			);

			let owner: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
			let data = class_info.data;
			// `repatriate_reserved` will check `to` account exist and return `DeadAccount`.
			// `transfer` not do this check.
			<T as Config>::Currency::unreserve(&owner, data.deposit.saturated_into());
			// At least there is one admin at this point.
			<T as Config>::Currency::transfer(&owner, &dest, data.deposit.saturated_into(), KeepAlive)?;

			// transfer all free from origin to dest
			orml_nft::Pallet::<T>::destroy_class(&who, class_id)?;

			Self::deposit_event(Event::DestroyedClass(who, class_id, dest));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {

	fn is_burnable(class_id: ClassIdOf<T>) -> Result<bool, DispatchError> {
		let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		Ok(data.properties.0.contains(ClassProperty::Burnable))
	}

	fn is_transferable(class_id: ClassIdOf<T>) -> Result<bool, DispatchError> {
		let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		Ok(data.properties.0.contains(ClassProperty::Transferable))
	}

	// fn delete_order(class_id: ClassIdOf<T>, token_id: TokenIdOf<T>, who: &T::AccountId) -> DispatchResult {
	// 	Orders::<T>::try_mutate_exists((class_id, token_id), who, |maybe_order| {
	// 		let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
	//
	// 		let mut deposit: Balance = Zero::zero();
	// 		if !order.by_token_owner {
	// 			// todo: emit an event for `order.currency_id`.
	// 			let d = T::MultiCurrency::unreserve(order.currency_id, &who, order.price.saturated_into());
	// 			deposit = deposit.saturating_add(order.price).saturating_sub(d);
	// 		}
	//
	// 		Categories::<T>::try_mutate(order.category_id, |category| -> DispatchResult {
	// 			category.as_mut().map(|cate| cate.nft_count = cate.nft_count.saturating_sub(One::one()) );
	// 			Ok(())
	// 		})?;
	//
	// 		let deposit = {
	// 			let d = <T as Config>::Currency::unreserve(&who, order.deposit.saturated_into());
	// 			deposit.saturating_add(order.deposit).saturating_sub(d.saturated_into())
	// 		};
	// 		Self::deposit_event(Event::RemovedOrder(class_id, token_id, who.clone(), deposit.saturated_into()));
	// 		*maybe_order = None;
	// 		Ok(())
	// 	})
	// }

	// fn try_delete_order(class_id: ClassIdOf<T>, token_id: TokenIdOf<T>, who: &T::AccountId) {
	// 	let _ = Self::delete_order(class_id, token_id, who);
	// }

	fn do_transfer(from: &T::AccountId, to: &T::AccountId, class_id: ClassIdOf<T>, token_id: TokenIdOf<T>, quantity: TokenIdOf<T>) -> DispatchResult {
		ensure!(Self::is_transferable(class_id)?, Error::<T>::NonTransferable);

		orml_nft::Pallet::<T>::transfer(from, to, (class_id, token_id), quantity)?;

		Self::deposit_event(Event::TransferredToken(from.clone(), to.clone(), class_id, token_id, quantity));
		Ok(())
	}

	pub fn add_class_admin_deposit(admin_count: u32) -> Balance {
		let proxy_deposit_before: Balance = <pallet_proxy::Pallet<T>>::deposit(1).saturated_into();
		let proxy_deposit_after: Balance = <pallet_proxy::Pallet<T>>::deposit(admin_count.saturating_add(1)).saturated_into();
		proxy_deposit_after.saturating_sub(proxy_deposit_before)
	}

	pub fn mint_token_deposit(metadata_len: u32) -> Balance {
		T::CreateTokenDeposit::get().saturating_add((metadata_len as Balance).saturating_mul(T::MetaDataByteDeposit::get()))
	}

	pub fn create_class_deposit(metadata_len: u32, name_len: u32, description_len: u32) -> (Balance, Balance) {
		let deposit: Balance = {
			let total_bytes = metadata_len.saturating_add(name_len).saturating_add(description_len);
			T::CreateClassDeposit::get().saturating_add(
				(total_bytes as Balance).saturating_mul(T::MetaDataByteDeposit::get())
			)
		};
		let proxy_deposit: Balance = <pallet_proxy::Pallet<T>>::deposit(1).saturated_into();
		(deposit, deposit.saturating_add(proxy_deposit))
	}
}