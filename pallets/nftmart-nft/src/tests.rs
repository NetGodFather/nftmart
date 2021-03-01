#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

// fn free_balance(who: &AccountId) -> Balance {
// 	<Runtime as Config>::Currency::free_balance(who)
// }

fn reserved_balance(who: &AccountId) -> Balance {
	<Runtime as Config>::Currency::reserved_balance(who)
}

fn class_id_account() -> AccountId {
	<Runtime as Config>::ModuleId::get().into_sub_account(CLASS_ID)
}

#[test]
fn create_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		let name = vec![1];
		let description = vec![1];
		assert_ok!(Nftmart::create_class(Origin::signed(ALICE), metadata.clone(), name.clone(), description.clone(), Default::default()));

		let event = Event::nftmart_nft(crate::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		let total_bytes: Balance = metadata.len().saturating_add(name.len()).saturating_add(description.len()) as Balance;
		let reserved = <Runtime as Config>::MetaDataByteDeposit::get().saturating_mul(total_bytes).saturating_add(Proxy::deposit(1u32));
		let reserved = <Runtime as Config>::CreateClassDeposit::get().saturating_add(reserved);

		assert_eq!(reserved_balance(&class_id_account()), reserved);
	});
}

#[test]
fn create_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Nftmart::create_class(
				Origin::signed(BOB),
				vec![1], vec![1], vec![1],
				Properties(ClassProperty::Transferable | ClassProperty::Burnable)
			),
			pallet_balances::Error::<Runtime, _>::InsufficientBalance
		);
	});
}

#[test]
fn mint_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let (metadata, reserved) = {
			let metadata = vec![1];
			let name = vec![1];
			let description = vec![1];
			assert_ok!(Nftmart::create_class(
				Origin::signed(ALICE),
				metadata.clone(), name.clone(), description.clone(),
				Properties(ClassProperty::Transferable | ClassProperty::Burnable)
			));
			let event = Event::nftmart_nft(crate::Event::CreatedClass(class_id_account(), CLASS_ID));
			assert_eq!(last_event(), event);

			let total_bytes: Balance = metadata.len().saturating_add(name.len()).saturating_add(description.len()) as Balance;
			let reserved = <Runtime as Config>::MetaDataByteDeposit::get().saturating_mul(total_bytes).saturating_add(Proxy::deposit(1u32));
			let reserved = <Runtime as Config>::CreateClassDeposit::get().saturating_add(reserved);

			(metadata, reserved)
		};

		let count: Balance = 2;
		let reserved = {
			let total_bytes: Balance = metadata.len() as Balance;
			let deposit = <Runtime as Config>::CreateTokenDeposit::get().saturating_add(
				total_bytes.saturating_mul(<Runtime as Config>::MetaDataByteDeposit::get())
			).saturating_mul(count);
			assert_eq!(
				Balances::deposit_into_existing(&class_id_account(), deposit).is_ok(),
				true
			);
			deposit.saturating_add(reserved)
		};

		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			count as u32
		));
		let event = Event::nftmart_nft(crate::Event::MintedToken(class_id_account(), BOB, CLASS_ID, count as u32));
		assert_eq!(last_event(), event);

		assert_eq!(reserved_balance(&class_id_account()), reserved);
	});
}

#[test]
fn mint_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			vec![1], vec![1], vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_noop!(
			Nftmart::mint(Origin::signed(ALICE), BOB, CLASS_ID_NOT_EXIST, vec![1], 2),
			Error::<Runtime>::ClassIdNotFound
		);

		assert_noop!(
			Nftmart::mint(Origin::signed(BOB), BOB, CLASS_ID, vec![1], 0),
			Error::<Runtime>::InvalidQuantity
		);

		assert_noop!(
			Nftmart::mint(Origin::signed(BOB), BOB, CLASS_ID, vec![1], 2),
			Error::<Runtime>::NoPermission
		);

		orml_nft::NextTokenId::<Runtime>::mutate(CLASS_ID, |id| {
			*id = <Runtime as orml_nft::Config>::TokenId::max_value()
		});
		{
			let total_bytes: Balance = vec![1].len() as Balance;
			let deposit = <Runtime as Config>::CreateTokenDeposit::get().saturating_add(
				total_bytes.saturating_mul(<Runtime as Config>::MetaDataByteDeposit::get())
			).saturating_mul(2);
			assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit).is_ok(), true);
		}
		assert_noop!(
			Nftmart::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 2),
			orml_nft::Error::<Runtime>::NoAvailableTokenId
		);
	});
}

// #[test]
// fn transfer_should_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(NFTModule::create_class(
// 			Origin::signed(ALICE),
// 			vec![1],
// 			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
// 		));
// 		assert_eq!(
// 			Balances::deposit_into_existing(&class_id_account(), 2 * <Runtime as Config>::CreateTokenDeposit::get())
// 				.is_ok(),
// 			true
// 		);
// 		assert_ok!(NFTModule::mint(
// 			Origin::signed(class_id_account()),
// 			BOB,
// 			CLASS_ID,
// 			vec![1],
// 			2
// 		));
//
// 		assert_ok!(NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)));
// 		let event = Event::nft(crate::Event::TransferredToken(BOB, ALICE, CLASS_ID, TOKEN_ID));
// 		assert_eq!(last_event(), event);
//
// 		assert_ok!(NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)));
// 		let event = Event::nft(crate::Event::TransferredToken(ALICE, BOB, CLASS_ID, TOKEN_ID));
// 		assert_eq!(last_event(), event);
// 	});
// }
//
// #[test]
// fn transfer_should_fail() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(NFTModule::create_class(
// 			Origin::signed(ALICE),
// 			vec![1],
// 			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
// 		));
// 		assert_eq!(
// 			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
// 				.is_ok(),
// 			true
// 		);
// 		assert_ok!(NFTModule::mint(
// 			Origin::signed(class_id_account()),
// 			BOB,
// 			CLASS_ID,
// 			vec![1],
// 			1
// 		));
// 		assert_noop!(
// 			NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID_NOT_EXIST, TOKEN_ID)),
// 			Error::<Runtime>::ClassIdNotFound
// 		);
// 		assert_noop!(
// 			NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID_NOT_EXIST)),
// 			Error::<Runtime>::TokenIdNotFound
// 		);
// 		assert_noop!(
// 			NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)),
// 			Error::<Runtime>::NoPermission
// 		);
// 	});
//
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(NFTModule::create_class(
// 			Origin::signed(ALICE),
// 			vec![1],
// 			Default::default()
// 		));
// 		assert_eq!(
// 			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
// 				.is_ok(),
// 			true
// 		);
// 		assert_ok!(NFTModule::mint(
// 			Origin::signed(class_id_account()),
// 			BOB,
// 			CLASS_ID,
// 			vec![1],
// 			1
// 		));
// 		assert_noop!(
// 			NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)),
// 			Error::<Runtime>::NonTransferable
// 		);
// 	});
// }
//
// #[test]
// fn burn_should_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(NFTModule::create_class(
// 			Origin::signed(ALICE),
// 			vec![1],
// 			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
// 		));
// 		assert_eq!(
// 			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
// 				.is_ok(),
// 			true
// 		);
// 		assert_ok!(NFTModule::mint(
// 			Origin::signed(class_id_account()),
// 			BOB,
// 			CLASS_ID,
// 			vec![1],
// 			1
// 		));
// 		assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
// 		let event = Event::nft(crate::Event::BurnedToken(BOB, CLASS_ID, TOKEN_ID));
// 		assert_eq!(last_event(), event);
//
// 		assert_eq!(
// 			reserved_balance(&class_id_account()),
// 			<Runtime as Config>::CreateClassDeposit::get() + Proxy::deposit(1u32)
// 		);
// 	});
// }
//
// #[test]
// fn burn_should_fail() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(NFTModule::create_class(
// 			Origin::signed(ALICE),
// 			vec![1],
// 			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
// 		));
// 		assert_eq!(
// 			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
// 				.is_ok(),
// 			true
// 		);
// 		assert_ok!(NFTModule::mint(
// 			Origin::signed(class_id_account()),
// 			BOB,
// 			CLASS_ID,
// 			vec![1],
// 			1
// 		));
// 		assert_noop!(
// 			NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID_NOT_EXIST)),
// 			Error::<Runtime>::TokenIdNotFound
// 		);
//
// 		assert_noop!(
// 			NFTModule::burn(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
// 			Error::<Runtime>::NoPermission
// 		);
//
// 		orml_nft::Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
// 			class_info.as_mut().unwrap().total_issuance = 0;
// 		});
// 		assert_noop!(
// 			NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
// 			orml_nft::Error::<Runtime>::NumOverflow
// 		);
// 	});
//
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(NFTModule::create_class(
// 			Origin::signed(ALICE),
// 			vec![1],
// 			Default::default()
// 		));
// 		assert_eq!(
// 			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
// 				.is_ok(),
// 			true
// 		);
// 		assert_ok!(NFTModule::mint(
// 			Origin::signed(class_id_account()),
// 			BOB,
// 			CLASS_ID,
// 			vec![1],
// 			1
// 		));
// 		assert_noop!(
// 			NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
// 			Error::<Runtime>::NonBurnable
// 		);
// 	});
// }
//
// #[test]
// fn destroy_class_should_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(NFTModule::create_class(
// 			Origin::signed(ALICE),
// 			vec![1],
// 			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
// 		));
// 		assert_eq!(
// 			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
// 				.is_ok(),
// 			true
// 		); // + 100
// 		assert_ok!(NFTModule::mint(
// 			Origin::signed(class_id_account()),
// 			BOB,
// 			CLASS_ID,
// 			vec![1],
// 			1
// 		));
// 		assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
// 		assert_ok!(NFTModule::destroy_class(
// 			Origin::signed(class_id_account()),
// 			CLASS_ID,
// 			BOB
// 		));
// 		let event = Event::nft(crate::Event::DestroyedClass(class_id_account(), CLASS_ID, BOB));
// 		assert_eq!(last_event(), event);
//
// 		assert_eq!(reserved_balance(&class_id_account()), 2);
// 		assert_eq!(free_balance(&ALICE), 99700 + 100 - 2);
// 		assert_eq!(free_balance(&BOB), 300);
// 	});
// }
//
// #[test]
// fn destroy_class_should_fail() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_ok!(NFTModule::create_class(
// 			Origin::signed(ALICE),
// 			vec![1],
// 			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
// 		));
// 		assert_eq!(
// 			Balances::deposit_into_existing(&class_id_account(), 1 * <Runtime as Config>::CreateTokenDeposit::get())
// 				.is_ok(),
// 			true
// 		);
// 		assert_ok!(NFTModule::mint(
// 			Origin::signed(class_id_account()),
// 			BOB,
// 			CLASS_ID,
// 			vec![1],
// 			1
// 		));
// 		assert_noop!(
// 			NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID_NOT_EXIST, BOB),
// 			Error::<Runtime>::ClassIdNotFound
// 		);
//
// 		assert_noop!(
// 			NFTModule::destroy_class(Origin::signed(BOB), CLASS_ID, BOB),
// 			Error::<Runtime>::NoPermission
// 		);
//
// 		assert_noop!(
// 			NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID, BOB),
// 			Error::<Runtime>::CannotDestroyClass
// 		);
//
// 		assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
// 		assert_ok!(NFTModule::destroy_class(
// 			Origin::signed(class_id_account()),
// 			CLASS_ID,
// 			BOB
// 		));
// 	});
// }
