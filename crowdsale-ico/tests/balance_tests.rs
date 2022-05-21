use gtest::System;
use gstd::Encode;
use ico_io::*;

use ico_contract::constants::*;

mod init_ico;
use init_ico::*;

#[test]
fn balance_after_two_purchases() {
    let sys = System::new();
    init(&sys);

    let ico = sys.get_program(2);

    start_sale(&ico, 2);

    balance_of(&ico, 0);

    let amount: u128 = 5;
    buy_tokens(&ico, amount, amount * START_PRICE);

    balance_of(&ico, amount);

    sys.spend_blocks((TIME_INCREASE_STEP + 1).try_into().unwrap());

    buy_tokens(&ico, amount, amount * (START_PRICE + PRICE_INCREASE_STEP));

    balance_of(&ico, amount * 2);
}

#[test]
#[should_panic]
fn not_owner_balance() {
    let sys = System::new();
    init(&sys);

    let ico = sys.get_program(2);

    start_sale(&ico, 1);

    let res = ico.send(USER_ID, IcoAction::BalanceOf(USER_ID.into()));
    assert!(res.contains(&(USER_ID, (IcoEvent::BalanceOf { address: USER_ID.into() , balance: 0 }).encode())));
}