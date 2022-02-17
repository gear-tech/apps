extern crate std;
use std::fmt::Write;

use codec::Encode;
use fungible_token_messages::*;
use gstd::vec;
use gstd::{ActorId, String, Vec};
use gtest::{Program, RunResult, System};

use crate::{
    CurveAmmAction, CurveAmmAddLiquidity, CurveAmmAddLiquidityReply, CurveAmmExchange,
    CurveAmmExchangeReply, CurveAmmRemoveLiquidity, CurveAmmRemoveLiquidityReply, CurveAmmReply,
};
const USERS: &'static [u64] = &[5, 6, 7];

fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).expect("Format failed")
    }
    s
}

fn init(sys: &System) {
    sys.init_logger();

    let usdc_token = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );
    let usdt_token = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );
    let lp_token = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );
    let curve_amm = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/curve_amm.wasm",
    );

    let res = usdc_token.send(
        USERS[0],
        InitConfig {
            name: String::from("USDC Stable Coin"),
            symbol: String::from("USDC"),
        },
    );
    assert!(res.log().is_empty());
    let res = usdt_token.send(
        USERS[0],
        InitConfig {
            name: String::from("USDT Stable Coin"),
            symbol: String::from("USDT"),
        },
    );
    assert!(res.log().is_empty());
    let res = lp_token.send(
        USERS[0],
        InitConfig {
            name: String::from("LP Token"),
            symbol: String::from("LPT"),
        },
    );
    assert!(res.log().is_empty());

    let usdc_id = encode_hex(usdc_token.id().as_slice());
    let usdt_id = encode_hex(usdt_token.id().as_slice());
    let lp_token_id = encode_hex(lp_token.id().as_slice());
    let token_accounts = usdc_id + "," + &usdt_id + "," + &lp_token_id;

    let res = curve_amm.send(
        USERS[0],
        crate::CurveAmmInitConfig {
            token_accounts: token_accounts.as_bytes().to_vec(),
            amplification_coefficient: 10000,
            fee: 0,
            admin_fee: 0,
        },
    );

    assert!(res.log().is_empty());
}

fn add_admin(ft: &Program, admin: &ActorId, owner: u64) -> RunResult {
    return ft.send(owner, fungible_token_messages::Action::AddAdmin(*admin));
}

fn mint(ft: &Program, creator: u64, account: &ActorId, amount: u128) -> RunResult {
    return ft.send(
        creator,
        fungible_token_messages::Action::Mint(MintInput {
            account: *account,
            amount,
        }),
    );
}

fn approve(ft: &Program, owner: u64, spender: &ActorId, amount: u128) -> RunResult {
    return ft.send(
        owner,
        fungible_token_messages::Action::Approve(ApproveInput {
            spender: *spender,
            amount,
        }),
    );
}

fn balance_of(ft: &Program, account: &ActorId) -> RunResult {
    return ft.send(100001, fungible_token_messages::Action::BalanceOf(*account));
}

fn add_liquidity(curve_amm: &Program, from: u64, pool_id: u32, amounts: Vec<u128>) -> RunResult {
    return curve_amm.send(
        from,
        CurveAmmAction::AddLiquidity(CurveAmmAddLiquidity { pool_id, amounts }),
    );
}

fn remove_liquidity(curve_amm: &Program, from: u64, pool_id: u32, lp_token: u128) -> RunResult {
    return curve_amm.send(
        from,
        CurveAmmAction::RemoveLiquidity(CurveAmmRemoveLiquidity {
            pool_id,
            amount: lp_token,
        }),
    );
}

fn exchange(
    curve_amm: &Program,
    from: u64,
    pool_id: u32,
    i: u32,
    j: u32,
    dx_amount: u128,
) -> RunResult {
    return curve_amm.send(
        from,
        CurveAmmAction::Exchange(CurveAmmExchange {
            pool_id,
            i,
            j,
            dx_amount,
        }),
    );
}

#[test]
fn dex_demo() {
    let sys = System::new();
    init(&sys);

    let usdc_program = sys.get_program(1);
    let usdt_program = sys.get_program(2);
    let lp_token_program = sys.get_program(3);
    let curve_amm_program = sys.get_program(4);

    // add curve_amm program as admin to lp-token program
    let curve_amm_actor_id = ActorId::new(
        curve_amm_program
            .id()
            .as_slice()
            .try_into()
            .expect("slice with incorrect length"),
    );
    let alice: ActorId = USERS[0].into();
    let bob: ActorId = USERS[1].into();
    let charlie: ActorId = USERS[2].into();
    let dex: ActorId = 4_u64.into();
    let zero: ActorId = 0_u64.into();
    let res = add_admin(&lp_token_program, &curve_amm_actor_id, USERS[0]);
    assert!(res.contains(&(USERS[0], Event::AdminAdded(curve_amm_actor_id).encode())));

    // add 100000 USDC to account ALICE
    let res = mint(&usdc_program, USERS[0], &alice, 100000);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: alice,
            amount: 100000
        })
        .encode()
    )));

    // add 100000 USDC to account BOB
    let res = mint(&usdc_program, USERS[0], &bob, 100000);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: bob,
            amount: 100000
        })
        .encode()
    )));

    // add 100000 USDT to account ALICE
    let res = mint(&usdt_program, USERS[0], &alice, 100000);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: alice,
            amount: 100000
        })
        .encode()
    )));

    // add 100000 USDT to account BOB
    let res = mint(&usdt_program, USERS[0], &bob, 100000);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: bob,
            amount: 100000
        })
        .encode()
    )));
    // add 5500 USDC to account CHARLIE
    let res = mint(&usdc_program, USERS[0], &charlie, 5500);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: charlie,
            amount: 5500
        })
        .encode()
    )));

    // Approve DEX to transfer USDC on behalf of ALICE
    let res = approve(&usdc_program, USERS[0], &dex, 2000000_u128);
    assert!(res.contains(&(
        USERS[0],
        Event::Approval(ApproveReply {
            owner: alice,
            spender: dex,
            amount: 2000000_u128
        })
        .encode()
    )));
    // Approve DEX to transfer USDT on behalf of ALICE
    let res = approve(&usdt_program, USERS[0], &dex, 2000000_u128);
    assert!(res.contains(&(
        USERS[0],
        Event::Approval(ApproveReply {
            owner: alice,
            spender: dex,
            amount: 2000000_u128
        })
        .encode()
    )));
    // Approve DEX to transfer USDC on behalf of BOB
    let res = approve(&usdc_program, USERS[1], &dex, 2000000_u128);
    assert!(res.contains(&(
        USERS[1],
        Event::Approval(ApproveReply {
            owner: bob,
            spender: dex,
            amount: 2000000_u128
        })
        .encode()
    )));
    // Approve DEX to transfer USDT on behalf of BOB
    let res = approve(&usdt_program, USERS[1], &dex, 2000000_u128);
    assert!(res.contains(&(
        USERS[1],
        Event::Approval(ApproveReply {
            owner: bob,
            spender: dex,
            amount: 2000000_u128
        })
        .encode()
    )));
    // Approve DEX to transfer USDC on behalf of CHARLIE
    let res = approve(&usdc_program, USERS[2], &dex, 2000000_u128);
    assert!(res.contains(&(
        USERS[2],
        Event::Approval(ApproveReply {
            owner: charlie,
            spender: dex,
            amount: 2000000_u128
        })
        .encode()
    )));
    // Approve DEX to transfer USDT on behalf of CHARLIE
    let res = approve(&usdt_program, USERS[2], &dex, 2000000_u128);
    assert!(res.contains(&(
        USERS[2],
        Event::Approval(ApproveReply {
            owner: charlie,
            spender: dex,
            amount: 2000000_u128
        })
        .encode()
    )));

    // check lp_token balance of ALICE
    let res = balance_of(&lp_token_program, &alice);
    assert!(res.contains(&(100001, Event::Balance(0_u128).encode())));
    // check lp_token balance of BOB
    let res = balance_of(&lp_token_program, &bob);
    assert!(res.contains(&(100001, Event::Balance(0_u128).encode())));

    // add USDC, USDT liquidity from ALICE to pool.
    let amounts = vec![10000, 10000];
    let res = add_liquidity(&curve_amm_program, USERS[0], 0_u32, amounts);
    assert!(res.contains(&(
        USERS[0],
        CurveAmmReply::AddLiquidity(CurveAmmAddLiquidityReply {
            who: alice,
            pool_id: 0_u32,
            mint_amount: 20000_u128
        })
        .encode()
    )));
    // add USDC, USDT liquidity from BOB to pool.
    let amounts = vec![10000, 10000];
    let res = add_liquidity(&curve_amm_program, USERS[1], 0_u32, amounts);
    assert!(res.contains(&(
        USERS[1],
        CurveAmmReply::AddLiquidity(CurveAmmAddLiquidityReply {
            who: bob,
            pool_id: 0_u32,
            mint_amount: 20000_u128
        })
        .encode()
    )));

    // CHARLIE exchanges 100 USDC for USDT
    let res = exchange(&curve_amm_program, USERS[2], 0_u32, 0, 1, 100_u128);
    assert!(res.contains(&(
        USERS[2],
        CurveAmmReply::Exchange(CurveAmmExchangeReply {
            who: charlie,
            pool_id: 0_u32,
            i: 0_u32,
            j: 1_u32,
            dy_amount: 100_u128
        })
        .encode()
    )));

    // redeem some LP tokens from ALICE
    let res = remove_liquidity(&curve_amm_program, USERS[0], 0_u32, 200_u128);
    let expected_amounts = vec![101, 100];
    assert!(res.contains(&(
        USERS[0],
        CurveAmmReply::RemoveLiquidity(CurveAmmRemoveLiquidityReply {
            who: alice,
            pool_id: 0_u32,
            amounts: expected_amounts
        })
        .encode()
    )));
    // redeem some LP tokesn from BOB
    let res = remove_liquidity(&curve_amm_program, USERS[1], 0_u32, 200_u128);
    let expected_amounts = vec![100, 99];
    assert!(res.contains(&(
        USERS[1],
        CurveAmmReply::RemoveLiquidity(CurveAmmRemoveLiquidityReply {
            who: bob,
            pool_id: 0_u32,
            amounts: expected_amounts
        })
        .encode()
    )));
}
