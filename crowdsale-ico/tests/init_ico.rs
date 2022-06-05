use core::time::Duration;

use gtest::{Program, System};
use gstd::{String, Encode};

use ft_io::*;
use ico_io::*;

use gstd::ActorId;

pub const TOKEN_ID: u64 = 1;
pub const ICO_CONTRACT_ID: u64 = 2;
pub const OWNER_ID: u64 = 100001;
pub const USER_ID: u64 = 12345;

pub const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

pub const TOKENS_CNT: u128 = 100;
pub const START_PRICE: u128 = 1000;
pub const PRICE_INCREASE_STEP: u128 = 100;
pub const TIME_INCREASE_STEP: u128 = 1000;


fn init_fungible_token(sys: &System) {
    let ft = Program::from_file(
        &sys,
        "../target/wasm32-unknown-unknown/release/fungible_token.wasm",
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
    let res = ft.send(OWNER_ID, FTAction::Mint(TOKENS_CNT));
    assert!(!res.main_failed());

    // let res = ft.send(
    //     OWNER_ID,
    //     FTAction::Approve {
    //         to: ICO_CONTRACT_ID.into(),
    //         amount: TOKENS_CNT,
    //     },  
    // );
    // assert!(!res.main_failed());
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
    assert!(res.contains(&(USER_ID, (IcoEvent::Bought { buyer: USER_ID.into(), amount, change: 0 }).encode())));
}

pub fn balance_of(ico: &Program, amount: u128) {
    let res = ico.send(OWNER_ID, IcoAction::BalanceOf(USER_ID.into()));
    assert!(res.contains(&(OWNER_ID, (IcoEvent::BalanceOf { address: USER_ID.into() , balance: amount }).encode())));
}