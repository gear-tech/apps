use gtest::{Program, System};
use ico_io::*;

mod init_ico;
use init_ico::*;

#[test]
fn test_init() {
    let sys = System::new();
    init(&sys);
}

#[test]
#[should_panic]
fn zero_owner_id_init() {
    let sys = System::new();
    sys.init_logger();

    let ico = Program::current(&sys);

    let res = ico.send(
        OWNER_ID,
        IcoInit { 
            tokens_goal: TOKENS_CNT, 
            token_id: TOKEN_ID.into(), 
            owner: ZERO_ID, 
            start_price: START_PRICE, 
            price_increase_step: PRICE_INCREASE_STEP, 
            time_increase_step: TIME_INCREASE_STEP, 
        },
    );

    assert!(res.log().is_empty());
}

#[test]
#[should_panic]
fn zero_token_id_init() {
    let sys = System::new();
    sys.init_logger();

    let ico = Program::current(&sys);

    let res = ico.send(
        OWNER_ID,
        IcoInit { 
            tokens_goal: TOKENS_CNT, 
            token_id: ZERO_ID,
            owner: OWNER_ID.into(), 
            start_price: START_PRICE, 
            price_increase_step: PRICE_INCREASE_STEP, 
            time_increase_step: TIME_INCREASE_STEP, 
        },
    );

    assert!(res.log().is_empty());
}

#[test]
#[should_panic]
fn zero_tokens_goal_init() {
    let sys = System::new();
    sys.init_logger();

    let ico = Program::current(&sys);

    let res = ico.send(
        OWNER_ID,
        IcoInit { 
            tokens_goal: 0, 
            token_id: TOKEN_ID.into(),
            owner: OWNER_ID.into(), 
            start_price: START_PRICE, 
            price_increase_step: PRICE_INCREASE_STEP, 
            time_increase_step: TIME_INCREASE_STEP, 
        },
    );

    assert!(res.log().is_empty());
}

#[test]
#[should_panic]
fn zero_start_price_init() {
    let sys = System::new();
    sys.init_logger();

    let ico = Program::current(&sys);

    let res = ico.send(
        OWNER_ID,
        IcoInit { 
            tokens_goal: TOKENS_CNT, 
            token_id: TOKEN_ID.into(),
            owner: OWNER_ID.into(), 
            start_price: 0, 
            price_increase_step: PRICE_INCREASE_STEP, 
            time_increase_step: TIME_INCREASE_STEP, 
        },
    );

    assert!(res.log().is_empty());
}