#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_core::constants_types::{Balance};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use enumflags2::BitFlags;

pub trait NftmartConfig<AccountId> {
	fn is_in_whitelist(_who: &AccountId) -> bool;
}

impl<AccountId> NftmartConfig<AccountId> for () {
	fn is_in_whitelist(_who: &AccountId) -> bool { true }
}

pub type NFTMetadata = Vec<u8>;

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
