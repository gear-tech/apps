use core::time::Duration;

use ft_io::*;
use gtest::{Program, System};
use gstd::{String, Encode};
use ico_io::*;

use ico_contract::constants::*;

fn init_fungible_token(sys: &System) {
    let ft = Program::from_file(
        &sys,
        "fungible-token/target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    let res = ft.send(
        OWNER_ID,
        InitConfig {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
        },
    );

    assert!(res.log().is_empty());

    mint_tokens(&ft);
}

fn mint_tokens(ft: &Program<'_>) {
    let res = ft.send(OWNER_ID, Action::Mint(TOKENS_CNT));
    assert!(!res.main_failed());

    let res = ft.send(
        OWNER_ID,
        Action::Approve {
            to: ICO_CONTRACT_ID.into(),
            amount: TOKENS_CNT,
        },  
    );
    assert!(!res.main_failed());
}

fn init_ico(sys: &System) {
    let ico = Program::current(&sys);

    let res = ico.send(
        OWNER_ID,
        IcoInit { 
            tokens_goal: TOKENS_CNT, 
            token_id: TOKEN_ID.into(), 
            owner: OWNER_ID.into(), 
            start_price: START_PRICE, 
            price_increase_step: PRICE_INCREASE_STEP, 
            time_increase_step: TIME_INCREASE_STEP, 
        },
    );
    assert!(res.log().is_empty());
}   

pub fn init(sys: &System) {
    sys.init_logger();

    init_fungible_token(&sys);
    init_ico(&sys);
}

pub fn start_sale(ico: &Program, ico_duration: u64) {
    let duration = Duration::from_secs(ico_duration).as_millis() as u64;
    let res = ico.send(OWNER_ID, IcoAction::StartSale(duration));

    assert!(res.contains(&(OWNER_ID, IcoEvent::SaleStarted(duration).encode())));
}

pub fn end_sale(ico: &Program) {
    let res = ico.send(OWNER_ID, IcoAction::EndSale);
    assert!(res.contains(&(OWNER_ID, IcoEvent::SaleEnded.encode())));
}

pub fn buy_tokens(ico: &Program, amount: u128, price: u128) {
    let res = ico.send_with_value(USER_ID, IcoAction::Buy(amount), price);
    assert!(res.contains(&(USER_ID, (IcoEvent::Bought { buyer: USER_ID.into(), amount }).encode())));
}

pub fn balance_of(ico: &Program, amount: u128) {
    let res = ico.send(OWNER_ID, IcoAction::BalanceOf(USER_ID.into()));
    assert!(res.contains(&(OWNER_ID, (IcoEvent::BalanceOf { address: USER_ID.into() , balance: amount }).encode())));
}