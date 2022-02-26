use codec::Encode;
use fungible_token_messages::*;
use gstd::vec;
use gstd::{ActorId, String, Vec};
use gtest::{Program, RunResult, System};
use primitive_types::U256;
use sp_arithmetic::{FixedPointNumber, FixedU128};

use crate::{
    CurveAmmAction, CurveAmmAddLiquidity, CurveAmmAddLiquidityReply, CurveAmmExchange,
    CurveAmmExchangeReply, CurveAmmRemoveLiquidity, CurveAmmRemoveLiquidityReply, CurveAmmReply,
};
use proptest::prelude::*;

const USERS: &'static [u64] = &[5, 6, 7];

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

    let res = curve_amm.send(
        USERS[0],
        crate::CurveAmmInitConfig {
            token_x_id: 1_u64.into(),
            token_y_id: 2_u64.into(),
            token_lp_id: 3_u64.into(),
            amplification_coefficient: 5,
            fee: 5, // 5%
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

/// Check that x/y ~ 1 up to a certain precision
pub fn acceptable_computation_error(x: u128, y: u128, precision: u128) -> Result<(), FixedU128> {
    let delta = i128::abs(x as i128 - y as i128);
    if delta > 1 {
        let epsilon: u128 = 1;
        let lower =
            FixedU128::saturating_from_rational(precision, precision.saturating_add(epsilon));
        let upper =
            FixedU128::saturating_from_rational(precision, precision.saturating_sub(epsilon));
        let q = FixedU128::checked_from_rational(x, y).expect("values too big; qed;");
        if lower <= q && q <= upper {
            Ok(())
        } else {
            Err(q)
        }
    } else {
        Ok(())
    }
}

#[test]
fn test_get_d() {
    // testing aginst some random value
    // NOTE: if values have too big difference then it does not pass with
    // `acceptable_computation_error`
    let unit = 10_000_000_000_u128;
    let balances: Vec<Vec<U256>> = vec![
        vec![
            (10 * unit).into(),      // 10
            (98 * unit / 10).into(), // 9.8
        ],
        vec![(1_000_000_000 * unit).into(), (1_000_000_000 * unit).into()],
        vec![
            (100_000_000_000_u128 * unit).into(),
            (100_000_000_000_u128 * unit).into(),
        ],
    ];
    let amp = 5_u128.into();
    let ann = crate::math::get_ann(amp, 2);
    assert!(ann.is_some());
    for xp in balances {
        let res = crate::math::get_d(&xp, ann.unwrap());
        assert!(res.is_some());
        let sum = xp[0].checked_add(xp[1]);
        assert!(sum.is_some());
        let res = res.unwrap().try_into().expect("could not fit into u128");
        let sum = sum.unwrap().try_into().expect("could not fit into u128");
        // when computing d first time it is equal to total number of coins,
        // hence compare against sum
        assert!(acceptable_computation_error(res, sum, 1000).is_ok());
    }

    // check for empty d
    let xp = vec![];
    let res = crate::math::get_d(&xp, ann.unwrap());
    assert_eq!(res, Some(U256::zero()));
}

#[test]
fn test_get_y() {
    let i = 0;
    let j = 1;
    let unit = 10_000_000_000_u128;
    let balances: Vec<Vec<U256>> = vec![
        vec![
            (10_u128 * unit).into(),           // 10
            (98_u128 * unit / 10_u128).into(), // 9.8
        ],
        vec![
            (1_000_000_000_u128 * unit).into(),
            (1_000_000_000_u128 * unit).into(),
        ],
        vec![
            (u64::MAX as u128 * unit).into(),
            (u64::MAX as u128 * unit).into(),
        ],
    ];
    let xs: Vec<U256> = vec![
        (55_u128 * unit / 10_u128).into(), // price of 5.5 tokens at index 0
        (1000_u128 * unit).into(),
        (1000_000_u128 * unit).into(),
    ];
    let amp = 5_u128.into();
    let ann = crate::math::get_ann(amp, 2);
    assert!(ann.is_some());
    for (xp, x) in balances.iter().zip(xs.iter()) {
        let xs = xp[i].checked_add(*x).expect("impossible. qed");
        let res = crate::math::get_y(i, j, xs, xp, ann.unwrap());
        assert!(res.is_some());
        let res = res.unwrap();
        let ys = xp[j].checked_sub(res).expect("impossible. qed");
        let y: u128 = ys.try_into().expect("could not fit into u128");
        let dy = y / unit;
        let dx: u128 = (*x).try_into().expect("could not fit into u128");
        let dx = dx / unit;
        assert!(acceptable_computation_error(dy, dx, 1000).is_ok());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn get_y_prop(
        dx in 0..u32::MAX,
    ) {
        let unit = 10_000_000_000_u128;
        let balances : Vec<U256> = vec![(u64::MAX as u128 * unit).into(), (u64::MAX as u128 * unit).into()];
        let dx : U256 = (dx as u128 * unit).into();
        let x = balances[0].checked_add(dx).expect("impossible. qed");
        let amp = 5_u128.into();
        let ann = crate::math::get_ann(amp, 2);
        assert!(ann.is_some());
        let ann = ann.unwrap();
        let res = crate::math::get_y(0, 1, x, &balances, ann);
        assert!(res.is_some());
        let res = res.unwrap();
        let dy = balances[1].checked_sub(res).expect("impossible. qed");
        let dy: u128 = dy.try_into().expect("could not fit into u128");
        let dy = dy / unit;
        let dx: u128 = dx.try_into().expect("could not fit into u128");
        let dx = dx / unit;
        assert!(acceptable_computation_error(dy, dx, 1000).is_ok());
    }
}

#[test]
fn dex_demo() {
    let sys = System::new();
    init(&sys);

    let unit = 10_000_000_000_u128;
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
    let res = mint(&usdc_program, USERS[0], &alice, 100000 * unit);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: alice,
            amount: 100000 * unit
        })
        .encode()
    )));

    // add 100000 USDC to account BOB
    let res = mint(&usdc_program, USERS[0], &bob, 100000 * unit);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: bob,
            amount: 100000 * unit
        })
        .encode()
    )));

    // add 100000 USDT to account ALICE
    let res = mint(&usdt_program, USERS[0], &alice, 100000 * unit);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: alice,
            amount: 100000 * unit
        })
        .encode()
    )));

    // add 100000 USDT to account BOB
    let res = mint(&usdt_program, USERS[0], &bob, 100000 * unit);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: bob,
            amount: 100000 * unit
        })
        .encode()
    )));
    // add 5500 USDC to account CHARLIE
    let res = mint(&usdc_program, USERS[0], &charlie, 5500 * unit);
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: zero,
            to: charlie,
            amount: 5500 * unit
        })
        .encode()
    )));

    // Approve DEX to transfer USDC on behalf of ALICE
    let res = approve(&usdc_program, USERS[0], &dex, 2000000_u128 * unit);
    assert!(res.contains(&(
        USERS[0],
        Event::Approval(ApproveReply {
            owner: alice,
            spender: dex,
            amount: 2000000_u128 * unit
        })
        .encode()
    )));
    // Approve DEX to transfer USDT on behalf of ALICE
    let res = approve(&usdt_program, USERS[0], &dex, 2000000_u128 * unit);
    assert!(res.contains(&(
        USERS[0],
        Event::Approval(ApproveReply {
            owner: alice,
            spender: dex,
            amount: 2000000_u128 * unit
        })
        .encode()
    )));
    // Approve DEX to transfer USDC on behalf of BOB
    let res = approve(&usdc_program, USERS[1], &dex, 2000000_u128 * unit);
    assert!(res.contains(&(
        USERS[1],
        Event::Approval(ApproveReply {
            owner: bob,
            spender: dex,
            amount: 2000000_u128 * unit
        })
        .encode()
    )));
    // Approve DEX to transfer USDT on behalf of BOB
    let res = approve(&usdt_program, USERS[1], &dex, 2000000_u128 * unit);
    assert!(res.contains(&(
        USERS[1],
        Event::Approval(ApproveReply {
            owner: bob,
            spender: dex,
            amount: 2000000_u128 * unit
        })
        .encode()
    )));
    // Approve DEX to transfer USDC on behalf of CHARLIE
    let res = approve(&usdc_program, USERS[2], &dex, 2000000_u128 * unit);
    assert!(res.contains(&(
        USERS[2],
        Event::Approval(ApproveReply {
            owner: charlie,
            spender: dex,
            amount: 2000000_u128 * unit
        })
        .encode()
    )));
    // Approve DEX to transfer USDT on behalf of CHARLIE
    let res = approve(&usdt_program, USERS[2], &dex, 2000000_u128 * unit);
    assert!(res.contains(&(
        USERS[2],
        Event::Approval(ApproveReply {
            owner: charlie,
            spender: dex,
            amount: 2000000_u128 * unit
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
    let amounts = vec![10000 * unit, 10000 * unit];
    let res = add_liquidity(&curve_amm_program, USERS[0], 0_u32, amounts);
    assert!(res.contains(&(
        USERS[0],
        CurveAmmReply::AddLiquidity(CurveAmmAddLiquidityReply {
            who: alice,
            pool_id: 0_u32,
            mint_amount: 20000_u128 * unit
        })
        .encode()
    )));
    // add USDC, USDT liquidity from BOB to pool.
    let amounts = vec![10000 * unit, 10000 * unit];
    let res = add_liquidity(&curve_amm_program, USERS[1], 0_u32, amounts);
    assert!(res.contains(&(
        USERS[1],
        CurveAmmReply::AddLiquidity(CurveAmmAddLiquidityReply {
            who: bob,
            pool_id: 0_u32,
            mint_amount: 20000_u128 * unit
        })
        .encode()
    )));

    // CHARLIE exchanges 100 USDC for USDT
    let res = exchange(&curve_amm_program, USERS[2], 0_u32, 0, 1, 100_u128 * unit);
    assert!(res.contains(&(
        USERS[2],
        CurveAmmReply::Exchange(CurveAmmExchangeReply {
            who: charlie,
            pool_id: 0_u32,
            i: 0_u32,
            j: 1_u32,
            dy_amount: 949568369100_u128 // 94.95 , 5% i.e 5 unit cut as fee
        })
        .encode()
    )));

    // redeem some LP tokens from ALICE
    let res = remove_liquidity(&curve_amm_program, USERS[0], 0_u32, 200_u128 * unit);
    let expected_amounts = vec![1005000000000 /* 100.5 */, 995252158154 /* 99.5 */];
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
    let res = remove_liquidity(&curve_amm_program, USERS[1], 0_u32, 200_u128 * unit);
    let expected_amounts = vec![1005000000000 /* 100.5 */, 995252158154 /* 99.5 */];
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
