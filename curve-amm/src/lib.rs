// CurveAmm Dex
// Implementation based on https://github.com/equilibrium-eosdt/equilibrium-curve-amm/blob/master/pallets/equilibrium-curve-amm/src/lib.rs
// For more details read
//      https://miguelmota.com/blog/understanding-stableswap-curve/
//      https://curve.fi/files/stableswap-paper.pdf
//      https://github.com/equilibrium-eosdt/equilibrium-curve-amm/blob/master/docs/deducing-get_y-formulas.pdf

#![no_std]
#![feature(const_btree_new)]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::{vec, vec::Vec};
use codec::{Decode, Encode};
use core::num::ParseIntError;
use fungible_token_messages::{
    Action, BurnInput, Event, MintInput, TransferFromInput, TransferInput,
};
use gstd::{errors::ContractError, exec, lock::mutex::Mutex, msg, prelude::*, ActorId, ToString};
use scale_info::TypeInfo;
use sp_arithmetic::{
    fixed_point::FixedU128,
    per_things::Permill,
    traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Zero},
    FixedPointNumber,
};

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

/// Return Err of the expression: `return Err($expression);`.
///
/// Used as `fail!(expression)`.
#[macro_export]
macro_rules! fail {
    ( $y:expr ) => {{
        return Err($y.into());
    }};
}

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            $crate::fail!($y);
        }
    }};
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmInitConfig {
    /// token accounts
    token_accounts: Vec<u8>,
    /// amplification_coefficient
    amplification_coefficient: u128,
    /// fee
    fee: u32,
    /// admin fee
    admin_fee: u32,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmInitReply {
    pool_id: u32,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmAddLiquidity {
    /// PoolId
    pool_id: u32,
    /// amounts
    amounts: Vec<u128>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmRemoveLiquidity {
    /// PoolId
    pool_id: u32,
    /// amount
    amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmExchange {
    /// PoolId
    pool_id: u32,
    /// i
    i: u32,
    /// j
    j: u32,
    /// dx amounts
    dx_amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
enum CurveAmmAction {
    AddLiquidity(CurveAmmAddLiquidity),
    RemoveLiquidity(CurveAmmRemoveLiquidity),
    Exchange(CurveAmmExchange),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmAddLiquidityReply {
    /// who
    who: ActorId,
    /// PoolId
    pool_id: u32,
    /// mint_amount
    mint_amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmRemoveLiquidityReply {
    /// who
    who: ActorId,
    /// PoolId
    pool_id: u32,
    /// amounts
    amounts: Vec<u128>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmExchangeReply {
    /// who
    who: ActorId,
    /// PoolId
    pool_id: u32,
    /// i
    i: u32,
    /// j
    j: u32,
    /// dy amounts
    dy_amount: u128,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
enum CurveAmmReply {
    AddLiquidity(CurveAmmAddLiquidityReply),
    RemoveLiquidity(CurveAmmRemoveLiquidityReply),
    Exchange(CurveAmmExchangeReply),
}

gstd::metadata! {
    title : "CurveAmm",
        init:
            input : CurveAmmInitConfig,
        handle:
            input: CurveAmmAction,
            output: CurveAmmReply,
}

/// Type that represents pool id
pub type PoolId = u32;

#[derive(Debug)]
pub enum CurveAmmError {
    /// Could not create new asset
    AssetNotCreated,
    /// Values in the storage are inconsistent
    InconsistentStorage,
    /// Not enough assets provided
    NotEnoughAssets,
    /// Some provided assets are not unique
    DuplicateAssets,
    /// Pool with specified id is not found
    PoolNotFound,
    /// Error occurred while performing math calculations
    Math,
    /// Specified asset amount is wrong
    WrongAssetAmount,
    /// Required amount of some token did not reached during adding or removing liquidity
    RequiredAmountNotReached,
    /// Source does not have required amount of coins to complete operation
    InsufficientFunds,
    /// Specified index is out of range
    IndexOutOfRange,
    /// The `AssetChecker` can use this error in case it can't provide better error
    ExternalAssetCheckFailed,
}

/// Storage record type for a pool
#[derive(Debug)]
pub struct PoolInfo {
    /// Owner of pool
    pub owner: ActorId,
    /// LP multiasset
    pub pool_asset: ActorId,
    /// List of multiassets supported by the pool
    pub assets: Vec<ActorId>,
    /// Initial amplification coefficient (leverage)
    pub amplification_coefficient: FixedU128,
    /// Amount of the fee pool charges for the exchange
    pub fee: Permill,
    /// Amount of the admin fee pool charges for the exchange
    pub admin_fee: Permill,
}

struct CurveAmm {
    /// Current number of pools (also ID for the next created pool)
    pool_count: PoolId,
    /// Existing pools
    pools: BTreeMap<PoolId, PoolInfo>,
}

impl CurveAmm {
    #[allow(dead_code)]
    pub fn get_precision(&self) -> FixedU128 {
        FixedU128::saturating_from_rational(1u32, 100_000_000u32)
    }

    /// Find `ann = amp * n^n` where `amp` - amplification coefficient,
    /// `n` - number of coins.
    #[allow(dead_code)]
    pub fn get_ann(&self, amp: FixedU128, n: usize) -> Option<FixedU128> {
        let n_coins = FixedU128::saturating_from_integer(n as u128);
        let mut ann = amp;
        for _ in 0..n {
            ann = ann.checked_mul(&n_coins)?;
        }
        Some(ann)
    }

    /// Find `d` preserving StableSwap invariant.
    /// Here `d` - total amount of coins when they have an equal price,
    /// `xp` - coin amounts, `ann` is amplification coefficient multiplied by `n^n`,
    /// where `n` is number of coins.
    ///
    /// # Notes
    ///
    /// D invariant calculation in non-overflowing integer operations iteratively
    ///
    /// ```pseudocode
    ///  A * sum(x_i) * n^n + D = A * D * n^n + D^(n+1) / (n^n * prod(x_i))
    /// ```
    ///
    /// Converging solution:
    ///
    /// ```pseudocode
    /// D[j + 1] = (A * n^n * sum(x_i) - D[j]^(n+1) / (n^n * prod(x_i))) / (A * n^n - 1)
    /// ```
    #[allow(dead_code)]
    pub fn get_d(&self, xp_f: &[FixedU128], ann_f: FixedU128) -> Option<FixedU128> {
        let zero = FixedU128::zero();
        let one = FixedU128::one();
        let n = FixedU128::saturating_from_integer(u128::try_from(xp_f.len()).ok()?);
        let sum = xp_f.iter().try_fold(zero, |s, x| s.checked_add(x))?;
        if sum == zero {
            return Some(zero);
        }
        let mut d = sum;

        for _ in 0..255 {
            let mut d_p = d;
            for x in xp_f.iter() {
                // d_p = d_p * d / (x * n)
                d_p = d_p.checked_mul(&d)?.checked_div(&x.checked_mul(&n)?)?;
            }
            let d_prev = d;

            // d = (ann * sum + d_p * n) * d / ((ann - 1) * d + (n + 1) * d_p)
            d = ann_f
                .checked_mul(&sum)?
                .checked_add(&d_p.checked_mul(&n)?)?
                .checked_mul(&d)?
                .checked_div(
                    &ann_f
                        .checked_sub(&one)?
                        .checked_mul(&d)?
                        .checked_add(&n.checked_add(&one)?.checked_mul(&d_p)?)?,
                )?;

            if d > d_prev {
                if d.checked_sub(&d_prev)? <= self.get_precision() {
                    return Some(d);
                }
            } else if d_prev.checked_sub(&d)? <= self.get_precision() {
                return Some(d);
            }
        }
        None
    }
    /// Here `xp` - coin amounts, `ann` is amplification coefficient multiplied by `n^n`, where
    /// `n` is number of coins.
    ///
    /// See https://github.com/equilibrium-eosdt/equilibrium-curve-amm/blob/master/docs/deducing-get_y-formulas.pdf
    /// for detailed explanation about formulas this function uses.
    ///
    /// # Notes
    ///
    /// Done by solving quadratic equation iteratively.
    ///
    /// ```pseudocode
    /// x_1^2 + x_1 * (sum' - (A * n^n - 1) * D / (A * n^n)) = D^(n+1) / (n^2n * prod' * A)
    /// x_1^2 + b * x_1 = c
    ///
    /// x_1 = (x_1^2 + c) / (2 * x_1 + b)
    /// ```
    pub fn get_y(
        &self,
        i: usize,
        j: usize,
        x_f: FixedU128,
        xp_f: &[FixedU128],
        ann_f: FixedU128,
    ) -> Option<FixedU128> {
        let zero = FixedU128::zero();
        let two = FixedU128::saturating_from_integer(2u8);
        let n = FixedU128::try_from(xp_f.len() as u128).ok()?;

        // Same coin
        if i == j {
            return None;
        }
        // j above n
        if j >= xp_f.len() {
            return None;
        }
        if i >= xp_f.len() {
            return None;
        }
        let d_f = self.get_d(xp_f, ann_f)?;
        let mut c = d_f;
        let mut s = zero;

        // Calculate s and c
        // p is implicitly calculated as part of c
        // note that loop makes n - 1 iterations
        for (k, xp_k) in xp_f.iter().enumerate() {
            let x_k: FixedU128;
            if k == i {
                x_k = x_f;
            } else if k != j {
                x_k = *xp_k;
            } else {
                continue;
            }
            // s = s + x_k
            s = s.checked_add(&x_k)?;
            // c = c * d / (x_k * n)
            c = c.checked_mul(&d_f)?.checked_div(&x_k.checked_mul(&n)?)?;
        }
        // c = c * d / (ann * n)
        // At this step we have d^n in the numerator of c
        // and n^(n-1) in its denominator.
        // So we multiplying it by remaining d/n
        c = c.checked_mul(&d_f)?.checked_div(&ann_f.checked_mul(&n)?)?;

        // b = s + d / ann
        // We subtract d later
        let b = s.checked_add(&d_f.checked_div(&ann_f)?)?;
        let mut y = d_f;

        for _ in 0..255 {
            let y_prev = y;
            // y = (y^2 + c) / (2 * y + b - d)
            // Subtract d to calculate b finally
            y = y
                .checked_mul(&y)?
                .checked_add(&c)?
                .checked_div(&two.checked_mul(&y)?.checked_add(&b)?.checked_sub(&d_f)?)?;

            // Equality with the specified precision
            if y > y_prev {
                if y.checked_sub(&y_prev)? <= self.get_precision() {
                    return Some(y);
                }
            } else if y_prev.checked_sub(&y)? <= self.get_precision() {
                return Some(y);
            }
        }

        None
    }

    fn create_pool(
        &mut self,
        who: &ActorId,
        assets: Vec<ActorId>,
        pool_asset: &ActorId,
        amplification_coefficient: FixedU128,
        fee: Permill,
        admin_fee: Permill,
    ) -> Option<PoolId> {
        // Assets related checks
        if assets.len() < 2 {
            panic!("create_pool : please provide atleast two assets");
        }
        let unique_assets = BTreeSet::<ActorId>::from_iter(assets.iter().copied());
        if unique_assets.len() != assets.len() {
            panic!("create_pool : duplicate assets found");
        }

        // Add new pool
        let pool_id = self.pool_count;

        // We expect that PoolInfos have sequential keys.
        // No PoolInfo can have key greater or equal to PoolCount
        if self.pools.get(&pool_id).is_some() {
            panic!("create_pool : inconsistent storage");
        }

        let pool_info = PoolInfo {
            owner: *who,
            pool_asset: *pool_asset,
            assets,
            amplification_coefficient,
            fee,
            admin_fee,
        };
        self.pools.insert(pool_id, pool_info);

        self.pool_count = pool_id.checked_add(1).expect("inconsistent storage");

        Some(pool_id)
    }

    fn fixed_to_u128(value: &FixedU128) -> u128 {
        value.into_inner() / FixedU128::DIV
    }

    pub async fn get_pool_balances(assets: &[ActorId]) -> Vec<FixedU128> {
        let mut balances = Vec::new();
        for asset in assets {
            let reply: Event = msg::send_and_wait_for_reply(
                *asset,
                &Action::BalanceOf(exec::program_id()),
                100_000_000_000,
                0,
            )
            .await
            .expect("Error in async message");
            let asset_balance = match reply {
                Event::Balance(bal) => FixedU128::saturating_from_integer(bal),
                _ => {
                    panic!("could not decode TotalIssuance reply");
                }
            };
            balances.push(asset_balance);
        }
        balances
    }

    async fn transfer_funds_to_pool(who: &ActorId, amounts: Vec<FixedU128>, assets: &[ActorId]) {
        let zero = FixedU128::zero();
        for (i, amount) in amounts.iter().enumerate() {
            if amount > &zero {
                let amount_u = amount.into_inner() / FixedU128::DIV;
                let _reply: Event = msg::send_and_wait_for_reply(
                    assets[i],
                    &Action::TransferFrom(TransferFromInput {
                        owner: *who,
                        to: exec::program_id(),
                        amount: amount_u,
                    }),
                    100_000_000_000,
                    0,
                )
                .await
                .expect("could not decode Transfer reply");
            }
        }
    }

    async fn transfer_funds_from_pool(who: &ActorId, amounts: Vec<FixedU128>, assets: &[ActorId]) {
        let zero = FixedU128::zero();
        for (i, amount) in amounts.iter().enumerate() {
            if amount > &zero {
                let amount: u128 = Self::fixed_to_u128(amount);
                let _reply: Event = msg::send_and_wait_for_reply(
                    assets[i],
                    &Action::Transfer(TransferInput { to: *who, amount }),
                    100_000_000_000,
                    0,
                )
                .await
                .expect("could not decode Transfer reply");
            }
        }
    }

    pub fn get_pool(&self, pool_id: &PoolId) -> &PoolInfo {
        self.pools.get(pool_id).expect("get_pool: Pool Not Found")
    }

    pub async fn get_lp_token_suppy(&self, pool_id: &PoolId) -> FixedU128 {
        let pool = self.get_pool(pool_id);
        let reply: Event =
            msg::send_and_wait_for_reply(pool.pool_asset, &Action::TotalSupply, 100_000_000_000, 0)
                .await
                .expect("Error in async message");
        let token_supply = match reply {
            Event::TotalSupply(bal) => FixedU128::saturating_from_integer(bal),
            _ => {
                panic!("could not decode TotalIssuance reply");
            }
        };
        token_supply
    }

    #[allow(dead_code)]
    async fn add_liquidity(
        &mut self,
        who: &ActorId,
        pool_id: PoolId,
        amounts: Vec<FixedU128>,
        min_mint_amount: FixedU128,
    ) {
        let lock = MUTEX.lock().await;
        let zero = FixedU128::zero();
        if !amounts.iter().all(|&x| x > zero) {
            gstd::mem::drop(lock);
            panic!("add_liquidity amounts: each amount must be grater than zero");
        }
        let pool = self.get_pool(&pool_id);
        let n_coins = pool.assets.len();
        if n_coins != pool.assets.len() {
            gstd::mem::drop(lock);
            panic!("add_liquidity number of coins and pool assets count mismatch");
        }
        if n_coins != amounts.len() {
            gstd::mem::drop(lock);
            panic!("add_liquidity number of coins and amouts count mismatch");
        }
        let ann = match self.get_ann(pool.amplification_coefficient, n_coins) {
            Some(v) => v,
            None => {
                gstd::mem::drop(lock);
                panic!("add_liquidity: math error");
            }
        };
        let old_balances = Self::get_pool_balances(&pool.assets).await;
        let d0 = match self.get_d(&old_balances, ann) {
            Some(v) => v,
            None => {
                gstd::mem::drop(lock);
                panic!("add_liquidity: math error");
            }
        };
        let token_supply = self.get_lp_token_suppy(&pool_id).await;
        let mut new_balances = old_balances.clone();
        for i in 0..n_coins {
            new_balances[i] = match new_balances[i].checked_add(&amounts[i]) {
                Some(v) => v,
                None => {
                    gstd::mem::drop(lock);
                    panic!("add_liquidity: math error");
                }
            };
        }
        let d1 = match self.get_d(&new_balances, ann) {
            Some(v) => v,
            None => {
                gstd::mem::drop(lock);
                panic!("add_liquidity: math error");
            }
        };
        if d1 <= d0 {
            gstd::mem::drop(lock);
            panic!("add_liquidity: d1 must be grater than d0");
        }
        let mint_amount;
        let mut fees = vec![FixedU128::zero(); n_coins];
        // Only account for fees if we are not the first to deposit
        if token_supply > zero {
            // Deposit x + withdraw y would chargVe about same
            // fees as a swap. Otherwise, one could exchange w/o paying fees.
            // And this formula leads to exactly that equality
            // fee = pool.fee * n_coins / (4 * (n_coins - 1))
            let one = FixedU128::saturating_from_integer(1u8);
            let four = FixedU128::saturating_from_integer(4u8);
            let n_coins_f = FixedU128::saturating_from_integer(n_coins as u128);
            let fee_f: FixedU128 = pool.fee.into();
            let n_coins_1 = match n_coins_f.checked_sub(&one) {
                Some(v) => v,
                None => {
                    gstd::mem::drop(lock);
                    panic!("add_liquidity: math error");
                }
            };
            let four_n_coins_1 = match four.checked_mul(&n_coins_1) {
                Some(v) => v,
                None => {
                    gstd::mem::drop(lock);
                    panic!("add_liquidity: math error");
                }
            };
            let pool_fees_n_coins = match fee_f.checked_mul(&n_coins_f) {
                Some(v) => v,
                None => {
                    gstd::mem::drop(lock);
                    panic!("add_liquidity: math error");
                }
            };
            let fee_f = match pool_fees_n_coins.checked_div(&four_n_coins_1) {
                Some(v) => v,
                None => {
                    gstd::mem::drop(lock);
                    panic!("add_liquidity: math error");
                }
            };
            // let admin_fee_f: FixedU128 = pool.admin_fee.into();
            for i in 0..n_coins {
                // ideal_balance = d1 * old_balances[i] / d0
                let ideal_balance = match (|| d1.checked_mul(&old_balances[i])?.checked_div(&d0))()
                {
                    Some(v) => v,
                    None => {
                        gstd::mem::drop(lock);
                        panic!("add_liquidity: math error");
                    }
                };

                let new_balance = new_balances[i];
                // difference = abs(ideal_balance - new_balance)
                let difference = match if ideal_balance > new_balance {
                    ideal_balance.checked_sub(&new_balance)
                } else {
                    new_balance.checked_sub(&ideal_balance)
                } {
                    Some(v) => v,
                    None => {
                        gstd::mem::drop(lock);
                        panic!("add_liquidity: math error");
                    }
                };

                fees[i] = match fee_f.checked_mul(&difference) {
                    Some(v) => v,
                    None => {
                        gstd::mem::drop(lock);
                        panic!("add_liquidity: math error");
                    }
                };
                new_balances[i] = match new_balances[i].checked_sub(&fees[i]) {
                    Some(v) => v,
                    None => {
                        gstd::mem::drop(lock);
                        panic!("add_liquidity: math error");
                    }
                };
            }
            let d2 = match self.get_d(&new_balances, ann) {
                Some(v) => v,
                None => {
                    gstd::mem::drop(lock);
                    panic!("add_liquidity: math error");
                }
            };

            // mint_amount = token_supply * (d2 - d0) / d0
            mint_amount = match (|| {
                token_supply
                    .checked_mul(&d2.checked_sub(&d0)?)?
                    .checked_div(&d0)
            })() {
                Some(v) => v,
                None => {
                    gstd::mem::drop(lock);
                    panic!("add_liquidity: math error");
                }
            };
        } else {
            mint_amount = d1;
        }
        if mint_amount < min_mint_amount {
            gstd::mem::drop(lock);
            panic!("add_liquidity: required mint amount not reached");
        }

        let _new_token_supply = match token_supply.checked_add(&mint_amount) {
            Some(v) => v,
            None => {
                gstd::mem::drop(lock);
                panic!("add_liquidity: math error");
            }
        };

        // Ensure that for all tokens user has sufficient amount
        for (i, amount) in amounts.iter().enumerate() {
            let reply: Event = msg::send_and_wait_for_reply(
                pool.assets[i],
                &Action::BalanceOf(*who),
                100_000_000_000,
                0,
            )
            .await
            .expect("Error in async message");
            let balance = match reply {
                Event::Balance(bal) => bal,
                _ => {
                    gstd::mem::drop(lock);
                    panic!("could not decode BalanceOf message");
                }
            };
            let balance: FixedU128 = FixedU128::saturating_from_integer(balance);
            if balance < *amount {
                gstd::mem::drop(lock);
                panic!("add_liquidity: insufficient funds");
            }
        }
        // Transfer funds to pool
        Self::transfer_funds_to_pool(who, amounts, &pool.assets).await;
        let mint_amount: u128 = Self::fixed_to_u128(&mint_amount);

        let reply: Result<Event, ContractError> = msg::send_and_wait_for_reply(
            pool.pool_asset,
            &Action::Mint(MintInput {
                account: *who,
                amount: mint_amount,
            }),
            100_000_000_000,
            0,
        )
        .await;
        match reply {
            Ok(_) => {}
            Err(e) => {
                gstd::mem::drop(lock);
                panic!("mint failed {:?}", e);
            }
        }

        let add_liquidity_reply = CurveAmmAddLiquidityReply {
            who: *who,
            pool_id,
            mint_amount,
        };

        msg::reply(CurveAmmReply::AddLiquidity(add_liquidity_reply), 0, 0);
    }

    #[allow(dead_code)]
    async fn remove_liquidity(&mut self, who: &ActorId, pool_id: PoolId, amount: FixedU128) {
        let lock = MUTEX.lock().await;
        let zero = FixedU128::zero();
        if amount <= zero {
            gstd::mem::drop(lock);
            panic!("remove_liquidity amounts: amount must be grater than zero");
        }
        let pool = self.get_pool(&pool_id);
        let n_coins = pool.assets.len();

        let token_supply = self.get_lp_token_suppy(&pool_id).await;
        let old_balances = Self::get_pool_balances(&pool.assets).await;
        let mut n_amounts = vec![FixedU128::zero(); n_coins];
        for (i, n_amount) in n_amounts.iter_mut().enumerate().take(n_coins) {
            let old_balance = old_balances[i];
            // value = old_balance * n_amount / token_supply
            let value = match (|| old_balance.checked_mul(&amount)?.checked_div(&token_supply))() {
                Some(v) => v,
                None => {
                    gstd::mem::drop(lock);
                    panic!("add_liquidity: math error");
                }
            };
            *n_amount = value;
        }
        let burn_amount: u128 = Self::fixed_to_u128(&amount);
        let reply: Result<Event, ContractError> = msg::send_and_wait_for_reply(
            pool.pool_asset,
            &Action::Burn(BurnInput {
                account: *who,
                amount: burn_amount,
            }),
            100_000_000_000,
            0,
        )
        .await;
        match reply {
            Ok(_) => {}
            Err(e) => {
                gstd::mem::drop(lock);
                panic!("burn failed {:?}", e);
            }
        }
        for (i, n_amount) in n_amounts.iter_mut().enumerate().take(n_coins) {
            let reply: Result<Event, ContractError> = msg::send_and_wait_for_reply(
                pool.assets[i],
                &Action::BalanceOf(*who),
                100_000_000_000,
                0,
            )
            .await;
            let reply = match reply {
                Ok(r) => r,
                Err(e) => {
                    gstd::mem::drop(lock);
                    panic!("balance failed {:?}", e);
                }
            };
            let balance = match reply {
                Event::Balance(bal) => bal,
                _ => {
                    gstd::mem::drop(lock);
                    panic!("could not decode BalanceOf message");
                }
            };
            let balance: FixedU128 = FixedU128::saturating_from_integer(balance);
            if balance < *n_amount {
                gstd::mem::drop(lock);
                panic!("remove_liquidity: insufficient fund");
            }
        }
        // Transfer funds from pool
        let amounts = n_amounts.iter().map(Self::fixed_to_u128).collect();
        Self::transfer_funds_from_pool(who, n_amounts, &pool.assets).await;

        let remove_liquidity_reply = CurveAmmRemoveLiquidityReply {
            who: *who,
            pool_id,
            amounts,
        };

        msg::reply(CurveAmmReply::RemoveLiquidity(remove_liquidity_reply), 0, 0);
    }

    #[allow(dead_code)]
    async fn exchange(
        &mut self,
        who: &ActorId,
        pool_id: PoolId,
        i_u: u32,
        j_u: u32,
        dx: FixedU128,
    ) {
        let lock = MUTEX.lock().await;
        let i = i_u as usize;
        let j = j_u as usize;
        let prec = self.get_precision();
        let zero = FixedU128::zero();
        if dx < zero {
            gstd::mem::drop(lock);
            panic!("exchange: dx amount shold be a positive value");
        }
        let pool = self.get_pool(&pool_id);
        let amp_coeff = pool.amplification_coefficient;
        let n_coins = pool.assets.len();
        if i >= n_coins && j >= n_coins {
            gstd::mem::drop(lock);
            panic!("exchange: i j indices should be smaller than n_coins");
        }
        let old_balances = Self::get_pool_balances(&pool.assets).await;
        let xp = old_balances.clone();
        let x = match xp[i].checked_add(&dx) {
            Some(v) => v,
            None => {
                gstd::mem::drop(lock);
                panic!("exchange: math error");
            }
        };
        let ann = match self.get_ann(amp_coeff, n_coins) {
            Some(v) => v,
            None => {
                gstd::mem::drop(lock);
                panic!("exchange: math error");
            }
        };
        let y = match self.get_y(i, j, x, &xp, ann) {
            Some(v) => v,
            None => {
                gstd::mem::drop(lock);
                panic!("exchange: math error");
            }
        };
        let dy = match (|| xp[j].checked_sub(&y)?.checked_sub(&prec))() {
            Some(v) => v,
            None => {
                gstd::mem::drop(lock);
                panic!("exchange: math error");
            }
        };

        let pool = self.get_pool(&pool_id);
        let reply: Result<Event, ContractError> = msg::send_and_wait_for_reply(
            pool.assets[i],
            &Action::BalanceOf(*who),
            100_000_000_000,
            0,
        )
        .await;
        let reply = match reply {
            Ok(r) => r,
            Err(e) => {
                gstd::mem::drop(lock);
                panic!("balance failed {:?}", e);
            }
        };
        let balance = match reply {
            Event::Balance(bal) => bal,
            _ => {
                panic!("could not decode BalanceOf message");
            }
        };
        let balance: FixedU128 = FixedU128::saturating_from_integer(balance);
        if balance < dx {
            gstd::mem::drop(lock);
            panic!("exchange: insufficient balance to exchange dx value");
        }
        let reply: Result<Event, ContractError> = msg::send_and_wait_for_reply(
            pool.assets[j],
            &Action::BalanceOf(exec::program_id()),
            100_000_000_000,
            0,
        )
        .await;
        let reply = match reply {
            Ok(r) => r,
            Err(e) => {
                gstd::mem::drop(lock);
                panic!("balance failed {:?}", e);
            }
        };
        let balance = match reply {
            Event::Balance(bal) => bal,
            _ => {
                panic!("could not decode BalanceOf message");
            }
        };
        let balance: FixedU128 = FixedU128::saturating_from_integer(balance);
        let amount: u128 = Self::fixed_to_u128(&dx);
        if balance < dy {
            gstd::mem::drop(lock);
            panic!("exchange: insufficient balance to exchange dy value");
        }
        let reply: Result<Event, ContractError> = msg::send_and_wait_for_reply(
            pool.assets[i],
            &Action::TransferFrom(TransferFromInput {
                owner: *who,
                to: exec::program_id(),
                amount,
            }),
            100_000_000_000,
            0,
        )
        .await;
        let _reply = match reply {
            Ok(r) => r,
            Err(e) => {
                gstd::mem::drop(lock);
                panic!("transfer failed {:?}", e);
            }
        };
        let amount: u128 = Self::fixed_to_u128(&dy);
        let reply: Result<Event, ContractError> = msg::send_and_wait_for_reply(
            pool.assets[j],
            &Action::Transfer(TransferInput { to: *who, amount }),
            100_000_000_000,
            0,
        )
        .await;
        let _reply = match reply {
            Ok(r) => r,
            Err(e) => {
                gstd::mem::drop(lock);
                panic!("transfer failed {:?}", e);
            }
        };
        let exchange_reply = CurveAmmExchangeReply {
            who: *who,
            pool_id,
            i: i_u,
            j: j_u,
            dy_amount: amount,
        };

        msg::reply(CurveAmmReply::Exchange(exchange_reply), 0, 0);
    }
}

static MUTEX: Mutex<u32> = Mutex::new(0);

static mut CURVE_AMM: CurveAmm = CurveAmm {
    pool_count: 0,
    pools: BTreeMap::new(),
};

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: CurveAmmInitConfig = msg::load().expect("Unable to decode InitConfig");
    let owner = msg::source();
    let input = String::from_utf8(config.token_accounts).expect("Invalid message: should be utf-8");
    let dests: Vec<&str> = input.split(',').collect();
    if dests.len() != 3 {
        panic!("Invalid input, should be three IDs separated by comma");
    }
    let x_token = ActorId::from_slice(
        &decode_hex(dests[0]).expect("INTIALIZATION FAILED: INVALID PROGRAM ID"),
    )
    .expect("Unable to create ActorId");
    let y_token = ActorId::from_slice(
        &decode_hex(dests[1]).expect("INTIALIZATION FAILED: INVALID PROGRAM ID"),
    )
    .expect("Unable to create ActorId");
    let lp_token = ActorId::from_slice(
        &decode_hex(dests[2]).expect("INTIALIZATION FAILED: INVALID PROGRAM ID"),
    )
    .expect("Unable to create ActorId");

    let assets = vec![x_token, y_token];
    let amplification_coefficient =
        FixedU128::saturating_from_integer(config.amplification_coefficient);
    let fee = Permill::from_percent(config.fee);
    let admin_fee = Permill::from_percent(config.admin_fee);
    let _res = CURVE_AMM
        .create_pool(
            &owner,
            assets,
            &lp_token,
            amplification_coefficient,
            fee,
            admin_fee,
        )
        .expect("Pool creation failed");
}

#[gstd::async_main]
async fn main() {
    let action: CurveAmmAction = msg::load().expect("Could not load Action");
    match action {
        CurveAmmAction::AddLiquidity(add_liquidity) => {
            let sender = msg::source();
            let pool_id: PoolId = add_liquidity.pool_id;
            let mut amounts_f = Vec::new();
            for amount in add_liquidity.amounts {
                amounts_f.push(FixedU128::saturating_from_integer(amount));
            }
            unsafe {
                let _res = CURVE_AMM
                    .add_liquidity(&sender, pool_id, amounts_f, FixedU128::zero())
                    .await;
            }
        }
        CurveAmmAction::RemoveLiquidity(remove_liquidity) => {
            let sender = msg::source();
            let pool_id: PoolId = remove_liquidity.pool_id;
            let amount_f = FixedU128::saturating_from_integer(remove_liquidity.amount);
            unsafe {
                let _res = CURVE_AMM.remove_liquidity(&sender, pool_id, amount_f).await;
            }
        }
        CurveAmmAction::Exchange(exchange) => {
            let sender = msg::source();
            let pool_id: PoolId = exchange.pool_id;
            let i = exchange.i;
            let j = exchange.j;
            let dx_amount_f = FixedU128::saturating_from_integer(exchange.dx_amount);
            unsafe {
                let _res = CURVE_AMM
                    .exchange(&sender, pool_id, i, j, dx_amount_f)
                    .await;
            }
        }
    }
}
