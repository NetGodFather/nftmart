#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use crate::mock::{Event, *};

#[test]
fn remove_order_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_category();
		ensure_min_order_deposit_a_unit();
		ensure_bob_balances(ACCURACY * 4);
		add_class(ALICE);
		add_token(BOB);
		add_token(ALICE);
		assert_ok!(Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID,
							  ACCURACY, CATEGORY_ID, CLASS_ID, TOKEN_ID, ACCURACY + 3, DEADLINE + 1));
		let order: OrderData<Runtime> = Nftmart::orders((CLASS_ID, TOKEN_ID), BOB).unwrap();
		assert_eq!(order.deposit, ACCURACY + 3);
		assert_eq!(order.price, ACCURACY);
		assert_ok!(Nftmart::remove_order(Origin::signed(BOB), CLASS_ID, TOKEN_ID));
		// check category
		assert_eq!(0, Nftmart::categories(CATEGORY_ID).unwrap().nft_count);
		assert!(Nftmart::categories(CATEGORY_ID + 1).is_none());
		assert_eq!(last_event(), Event::nftmart_nft(crate::Event::RemovedOrder(CLASS_ID, TOKEN_ID, BOB, ACCURACY + 3)));
		assert_ok!(Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID,
							  ACCURACY, CATEGORY_ID, CLASS_ID, TOKEN_ID + 1, ACCURACY + 3, DEADLINE + 1));
		assert_ok!(Nftmart::remove_order(Origin::signed(BOB), CLASS_ID, TOKEN_ID + 1));
		assert_eq!(last_event(), Event::nftmart_nft(crate::Event::RemovedOrder(CLASS_ID, TOKEN_ID + 1, BOB, ACCURACY * 2 + 3)));
		// check category
		assert_eq!(0, Nftmart::categories(CATEGORY_ID).unwrap().nft_count);
		assert!(Nftmart::categories(CATEGORY_ID + 1).is_none());
	});
}

#[test]
fn remove_order_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		add_category();
		ensure_min_order_deposit_a_unit();
		ensure_bob_balances(ACCURACY * 4);
		add_class(ALICE);
		add_token(BOB);
		add_token(ALICE);
		assert_ok!(Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID,
							  ACCURACY, CATEGORY_ID, CLASS_ID, TOKEN_ID, ACCURACY + 3, DEADLINE + 1));
		assert_noop!(
			Nftmart::remove_order(Origin::signed(ALICE), CLASS_ID, TOKEN_ID),
			Error::<Runtime>::OrderNotFound,
		);
		// check category
		assert_eq!(1, Nftmart::categories(CATEGORY_ID).unwrap().nft_count);
		assert!(Nftmart::categories(CATEGORY_ID + 1).is_none());
		assert_ok!(Nftmart::remove_order(Origin::signed(BOB), CLASS_ID, TOKEN_ID));
		assert_noop!(
			Nftmart::remove_order(Origin::signed(BOB), CLASS_ID, TOKEN_ID),
			Error::<Runtime>::OrderNotFound,
		);
		// check category
		assert_eq!(0, Nftmart::categories(CATEGORY_ID).unwrap().nft_count);
		assert!(Nftmart::categories(CATEGORY_ID + 1).is_none());
	});
}

#[test]
fn submit_order_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_category();
		ensure_min_order_deposit_a_unit();
		ensure_bob_balances(ACCURACY * 4);
		add_class(ALICE);
		add_token(BOB);
		add_token(ALICE);
		// Get the accounts of Alice & Bob.
		let alice_reserved = Currencies::reserved_balance(NATIVE_CURRENCY_ID, &ALICE);
		let bob_reserved = Currencies::reserved_balance(NATIVE_CURRENCY_ID, &BOB);
		assert_eq!(alice_reserved + bob_reserved, 0);
		// Bob submits his own token.
		assert_ok!(Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID,
							  ACCURACY, CATEGORY_ID, CLASS_ID, TOKEN_ID, ACCURACY + 3, DEADLINE + 1));
		assert_eq!(last_event(), Event::nftmart_nft(crate::Event::CreatedOrder(CLASS_ID, TOKEN_ID, BOB)));
		// check category
		assert_eq!(1, Nftmart::categories(CATEGORY_ID).unwrap().nft_count);
		assert!(Nftmart::categories(CATEGORY_ID + 1).is_none());
		let alice_reserved = Currencies::reserved_balance(NATIVE_CURRENCY_ID, &ALICE);
		let bob_reserved = Currencies::reserved_balance(NATIVE_CURRENCY_ID, &BOB);
		assert_eq!(alice_reserved, 0);
		assert_eq!(bob_reserved, ACCURACY + 3);
		assert!(Nftmart::orders((CLASS_ID, TOKEN_ID), BOB).unwrap().by_token_owner);
		// Bob submits an order to buy Alice's token.
		assert_ok!(Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID,
							  ACCURACY, CATEGORY_ID, CLASS_ID, TOKEN_ID + 1, ACCURACY + 3, DEADLINE + 1));
		assert_eq!(last_event(), Event::nftmart_nft(crate::Event::CreatedOrder(CLASS_ID, TOKEN_ID + 1, BOB)));
		let alice_reserved = Currencies::reserved_balance(NATIVE_CURRENCY_ID, &ALICE);
		let bob_reserved = Currencies::reserved_balance(NATIVE_CURRENCY_ID, &BOB);
		assert_eq!(alice_reserved, 0);
		assert_eq!(bob_reserved, ACCURACY + 3 + ACCURACY + ACCURACY + 3);
		assert!(!Nftmart::orders((CLASS_ID, TOKEN_ID + 1), BOB).unwrap().by_token_owner);
		assert_eq!(2, Nftmart::categories(CATEGORY_ID).unwrap().nft_count);
		assert!(Nftmart::categories(CATEGORY_ID + 1).is_none());
	});
}

#[test]
fn submit_order_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		add_category();
		ensure_min_order_deposit_a_unit();
		ensure_bob_balances(ACCURACY);
		add_class(ALICE);
		add_token(BOB);
		assert_noop!(
			Nftmart::submit_order(Origin::signed(BOB), 1, ACCURACY, CATEGORY_ID, CLASS_ID, TOKEN_ID, ACCURACY, DEADLINE),
			Error::<Runtime>::NativeCurrencyOnlyForNow,
		);
		assert_noop!(
			Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID,
				ACCURACY, CATEGORY_ID, CLASS_ID, TOKEN_ID_NOT_EXIST, ACCURACY, DEADLINE),
			Error::<Runtime>::TokenIdNotFound,
		);
		assert_noop!(
			Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID,
				ACCURACY, CATEGORY_ID, CLASS_ID, TOKEN_ID, ACCURACY, 1),
			Error::<Runtime>::InvalidDeadline,
		);
		assert_noop!(
			Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID,
				ACCURACY, CATEGORY_ID_NOT_EXIST, CLASS_ID, TOKEN_ID, ACCURACY, DEADLINE + 1),
			Error::<Runtime>::CategoryNotFound,
		);
		assert_noop!(
			Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID,
				ACCURACY, CATEGORY_ID, CLASS_ID, TOKEN_ID, ACCURACY - 1, DEADLINE + 1),
			Error::<Runtime>::InvalidDeposit,
		);
	});
}

#[test]
fn create_category_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!({ let id_expect: CategoryIdOf<Runtime> = Zero::zero(); id_expect }, Nftmart::next_category_id());
		assert_eq!(None, Nftmart::categories(CATEGORY_ID));

		let metadata = vec![1];
		assert_ok!(Nftmart::create_category(Origin::root(), metadata.clone()));

		let event = Event::nftmart_nft(crate::Event::CreatedCategory(CATEGORY_ID));
		assert_eq!(last_event(), event);
		assert_eq!({ let id_expect: CategoryIdOf<Runtime> = One::one(); id_expect }, Nftmart::next_category_id());
		assert_eq!(Some(CategoryData{ metadata, nft_count: 0 }), Nftmart::categories(CATEGORY_ID));
		assert_eq!(None, Nftmart::categories(CATEGORY_ID_NOT_EXIST));
	});
}

#[test]
fn create_category_should_fail() {
	let metadata = vec![1];
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Nftmart::create_category(Origin::signed(ALICE), metadata.clone()),
			DispatchError::BadOrigin,
		);
	});
	ExtBuilder::default().build().execute_with(|| {
		NextCategoryId::<Runtime>::set(<CategoryIdOf<Runtime>>::max_value());
		assert_noop!(
			Nftmart::create_category(Origin::root(), metadata.clone()),
			Error::<Runtime>::NoAvailableCategoryId,
		);
	});
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

		let reserved = Nftmart::create_class_deposit(metadata.len() as u32, name.len() as u32, description.len() as u32).1;
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

			let deposit = Nftmart::create_class_deposit(metadata.len() as u32, name.len() as u32, description.len() as u32).1;
			(metadata, deposit)
		};

		let count: Balance = 2;
		let reserved = {
			let deposit = Nftmart::mint_token_deposit(metadata.len() as u32, count as u32).1;
			assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit as Balance).is_ok(), true);
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
		let metadata = vec![1];
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata.clone(), vec![1], vec![1],
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
			let deposit = Nftmart::mint_token_deposit(metadata.len() as u32, 2).1;
			assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit).is_ok(), true);
		}
		assert_noop!(
			Nftmart::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 2),
			orml_nft::Error::<Runtime>::NoAvailableTokenId
		);
	});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata.clone(), vec![1], vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		let deposit = Nftmart::mint_token_deposit(metadata.len() as u32, 2).1;
		assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit).is_ok(), true);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			2
		));

		assert_ok!(Nftmart::transfer(Origin::signed(BOB), ALICE, CLASS_ID, TOKEN_ID));
		let event = Event::nftmart_nft(crate::Event::TransferredToken(BOB, ALICE, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);

		assert_ok!(Nftmart::transfer(Origin::signed(ALICE), BOB, CLASS_ID, TOKEN_ID));
		let event = Event::nftmart_nft(crate::Event::TransferredToken(ALICE, BOB, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn transfer_should_fail() {
	let metadata = vec![1];
	let deposit = Nftmart::mint_token_deposit(metadata.len() as u32, 1).1;
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata.clone(), vec![1], vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit).is_ok(), true);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			Nftmart::transfer(Origin::signed(BOB), ALICE, CLASS_ID_NOT_EXIST, TOKEN_ID),
			Error::<Runtime>::ClassIdNotFound
		);
		assert_noop!(
			Nftmart::transfer(Origin::signed(BOB), ALICE, CLASS_ID, TOKEN_ID_NOT_EXIST),
			Error::<Runtime>::TokenIdNotFound
		);
		assert_noop!(
			Nftmart::transfer(Origin::signed(ALICE), BOB, CLASS_ID, TOKEN_ID),
			Error::<Runtime>::NoPermission
		);
		// submit an order.
		assert_ok!(Nftmart::create_category(Origin::root(), vec![1]));
		assert_ok!(Nftmart::update_min_order_deposit(Origin::root(), 20));
		assert_ok!(Currencies::deposit(NATIVE_CURRENCY_ID, &BOB, ACCURACY));
		assert_ok!(Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID, ACCURACY, CATEGORY_ID,
			CLASS_ID, TOKEN_ID, ACCURACY, DEADLINE));
		// prevent from transferring the NFT.
		assert_noop!(
			Nftmart::transfer(Origin::signed(BOB), ALICE, CLASS_ID, TOKEN_ID),
			Error::<Runtime>::OrderExists
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata.clone(), vec![1], vec![1],
			Default::default()
		));
		assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit).is_ok(), true);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			Nftmart::transfer(Origin::signed(BOB), ALICE, CLASS_ID, TOKEN_ID),
			Error::<Runtime>::NonTransferable
		);
	});
}

#[test]
fn burn_should_work() {
	let metadata = vec![1];
	let name = vec![1];
	let description = vec![1];
	let deposit_token = Nftmart::mint_token_deposit(metadata.len() as u32, 1).1;
	let deposit_class = Nftmart::create_class_deposit(metadata.len() as u32, name.len() as u32, description.len() as u32).1;
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata, name, description,
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit_token).is_ok(), true);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_eq!(
			reserved_balance(&class_id_account()),
			deposit_class.saturating_add(deposit_token)
		);
		assert_ok!(Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID));
		let event = Event::nftmart_nft(crate::Event::BurnedToken(BOB, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);

		assert_eq!(
			reserved_balance(&class_id_account()),
			deposit_class
		);
	});
}

#[test]
fn burn_should_fail() {
	let metadata = vec![1];
	let name = vec![1];
	let description = vec![1];
	let deposit_token = Nftmart::mint_token_deposit(metadata.len() as u32, 1).1;
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata.clone(), name.clone(), description.clone(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit_token).is_ok(), true);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID_NOT_EXIST),
			Error::<Runtime>::TokenIdNotFound
		);

		assert_noop!(
			Nftmart::burn(Origin::signed(ALICE), CLASS_ID, TOKEN_ID),
			Error::<Runtime>::NoPermission
		);

		orml_nft::Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
			class_info.as_mut().unwrap().total_issuance = 0;
		});
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID),
			orml_nft::Error::<Runtime>::NumOverflow
		);

		// submit an order.
		assert_ok!(Nftmart::create_category(Origin::root(), vec![1]));
		assert_ok!(Nftmart::update_min_order_deposit(Origin::root(), 20));
		assert_ok!(Currencies::deposit(NATIVE_CURRENCY_ID, &BOB, ACCURACY));
		assert_ok!(Nftmart::submit_order(Origin::signed(BOB), NATIVE_CURRENCY_ID, ACCURACY, CATEGORY_ID,
			CLASS_ID, TOKEN_ID, ACCURACY, DEADLINE));
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID),
			Error::<Runtime>::OrderExists
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata.clone(), name.clone(), description.clone(),
			Default::default()
		));
		assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit_token).is_ok(), true);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID),
			Error::<Runtime>::NonBurnable
		);
	});
}

#[test]
fn destroy_class_should_work() {
	let metadata = vec![1];
	let name = vec![1];
	let description = vec![1];
	let deposit_token = Nftmart::mint_token_deposit(metadata.len() as u32, 1).1;
	let deposit_class = Nftmart::create_class_deposit(metadata.len() as u32, name.len() as u32, description.len() as u32).1;
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(reserved_balance(&class_id_account()), 0);
		assert_eq!(free_balance(&class_id_account()), 0);
		assert_eq!(free_balance(&ALICE), 100000);

		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata, name, description,
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(free_balance(&ALICE), 100000 - deposit_class);
		assert_eq!(free_balance(&class_id_account()), 0);
		assert_eq!(reserved_balance(&class_id_account()), deposit_class);
		assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit_token).is_ok(), true);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_eq!(free_balance(&class_id_account()), 0);
		assert_eq!(reserved_balance(&class_id_account()), deposit_class.saturating_add(deposit_token));
		assert_ok!(Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID));
		assert_eq!(reserved_balance(&class_id_account()), deposit_class);
		assert_eq!(free_balance(&class_id_account()), 0);
		assert_ok!(Nftmart::destroy_class(
			Origin::signed(class_id_account()),
			CLASS_ID,
			BOB
		));
		let event = Event::nftmart_nft(crate::Event::DestroyedClass(class_id_account(), CLASS_ID, BOB));
		assert_eq!(last_event(), event);
		assert_eq!(free_balance(&class_id_account()), 0);

		assert_eq!(reserved_balance(&class_id_account()), Proxy::deposit(1));

		let free_bob = deposit_class.saturating_add(deposit_token).saturating_sub(Proxy::deposit(1));
		assert_eq!(free_balance(&ALICE), 100000 - deposit_class);
		assert_eq!(free_balance(&BOB), free_bob);
	});
}

#[test]
fn destroy_class_should_fail() {
	let metadata = vec![1];
	let name = vec![1];
	let description = vec![1];
	let deposit_token = Nftmart::mint_token_deposit(metadata.len() as u32, 1).1;
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nftmart::create_class(
			Origin::signed(ALICE),
			metadata, name, description,
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(Balances::deposit_into_existing(&class_id_account(), deposit_token).is_ok(), true);
		assert_ok!(Nftmart::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![1],
			1
		));
		assert_noop!(
			Nftmart::destroy_class(Origin::signed(class_id_account()), CLASS_ID_NOT_EXIST, BOB),
			Error::<Runtime>::ClassIdNotFound
		);

		assert_noop!(
			Nftmart::destroy_class(Origin::signed(BOB), CLASS_ID, BOB),
			Error::<Runtime>::NoPermission
		);

		assert_noop!(
			Nftmart::destroy_class(Origin::signed(class_id_account()), CLASS_ID, BOB),
			Error::<Runtime>::CannotDestroyClass
		);

		assert_ok!(Nftmart::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID));
		assert_ok!(Nftmart::destroy_class(
			Origin::signed(class_id_account()),
			CLASS_ID,
			BOB
		));
	});
}