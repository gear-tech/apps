// CurveAmm Dex
// Implementation based on https://github.com/equilibrium-eosdt/equilibrium-curve-amm/blob/master/pallets/equilibrium-curve-amm/src/lib.rs
// For more details read
//      https://miguelmota.com/blog/understanding-stableswap-curve/
//      https://curve.fi/files/stableswap-paper.pdf
//      https://github.com/equilibrium-eosdt/equilibrium-curve-amm/blob/master/docs/deducing-get_y-formulas.pdf

#![no_std]
#![feature(const_btree_new)]

#[cfg(test)]
mod tests;

mod math;

extern crate alloc;

use crate::math::*;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::{vec, vec::Vec};
use codec::{Decode, Encode};
// use core::num::ParseIntError;
use fungible_token_messages::{
    Action, BurnInput, Event, MintInput, TransferFromInput, TransferInput,
};
use futures::future;
use gstd::exec::program_id;
use gstd::{errors::ContractError, exec, lock::mutex::Mutex, msg, prelude::*, ActorId, ToString};
use scale_info::TypeInfo;
use sp_arithmetic::{
    fixed_point::FixedU128,
    per_things::Permill,
    traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero},
    FixedPointNumber,
};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct CurveAmmInitConfig {
    /// program id of fungible token X.
    token_x_id: ActorId,
    /// program id of fungible token Y.
    token_y_id: ActorId,
    /// program id of fungible token LP.
    token_lp_id: ActorId,
    /// amp_coeff is configuration parameter used in stableswap algorithm.
    amplification_coefficient: u128,
    /// fees charged for any operation which changes pool's balances in imbalanced way.
    fee: u32,
    /// fee charged by administrator. (This can be 0 too)
    admin_fee: u32,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmInitReply {
    /// id of newly created pool.
    pool_id: u32,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmAddLiquidity {
    /// id of pool to which liquidity should be added.
    pool_id: u32,
    /// amounts of x and y tokens to be added.
    amounts: Vec<u128>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmRemoveLiquidity {
    /// id of the pool from which liquidity should be removed.
    pool_id: u32,
    /// amount of lp-tokens which are to be redeemed.
    amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmExchange {
    /// id of the pool on which exchange is performed.
    pool_id: u32,
    /// index of token supplied as input.
    i: u32,
    /// index of token expected as output.
    j: u32,
    /// amount of input token.
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
    /// user who executed add liquidity.
    who: ActorId,
    /// PoolId on which add liquidity is performed.
    pool_id: u32,
    /// amount of lp token minted for added liquidity.
    mint_amount: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmRemoveLiquidityReply {
    /// user who executed remove liquidity.
    who: ActorId,
    /// PoolId on which remove liquidity is performed.
    pool_id: u32,
    /// amounts of tokens removed from pool.
    amounts: Vec<u128>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
struct CurveAmmExchangeReply {
    /// user who executed exchange.
    who: ActorId,
    /// PoolId on which exchange is performed.
    pool_id: u32,
    /// i, index of tokens sent to pool.
    i: u32,
    /// j, index of tokens exchanged from pool.
    j: u32,
    /// amount of tokens received by user.
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
    /// FungibleToken Burn failed  
    BurnFailed(ContractError),
    /// FungibleToken BalanceOf failed
    BalaceOfFailed(ContractError),
    /// FungibleToken TransferFrom failed
    TransferFromFailed(ContractError),
    /// FungibleToken Transfer failed
    TransferFailed(ContractError),
    /// FungibleToken Mint failed
    MintFailed(ContractError),
    /// FungibleToken TotalSupply failed
    TotalSupplyFailed(ContractError),
    /// Error while decoding reply from FungibleToken
    DecodeError,
    /// Error while create FixedU128 from u128
    ConversionError,
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
    fn create_pool(
        &mut self,
        who: &ActorId,
        assets: Vec<ActorId>,
        pool_asset: &ActorId,
        amplification_coefficient: FixedU128,
        fee: Permill,
        admin_fee: Permill,
    ) -> Result<PoolId, CurveAmmError> {
        // Assets related checks
        if assets.len() < 2 {
            return Err(CurveAmmError::NotEnoughAssets);
        }
        let unique_assets = BTreeSet::<ActorId>::from_iter(assets.iter().copied());
        if unique_assets.len() != assets.len() {
            return Err(CurveAmmError::DuplicateAssets);
        }

        // Add new pool
        let pool_id = self.pool_count;

        // We expect that PoolInfos have sequential keys.
        // No PoolInfo can have key greater or equal to PoolCount
        if self.pools.get(&pool_id).is_some() {
            return Err(CurveAmmError::InconsistentStorage);
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

        self.pool_count = pool_id
            .checked_add(1)
            .ok_or(CurveAmmError::InconsistentStorage)?;

        Ok(pool_id)
    }

    fn fixed_to_u128(value: &FixedU128) -> u128 {
        value.round().into_inner() / FixedU128::DIV
    }

    fn u128_to_fixed(value: u128) -> Result<FixedU128, CurveAmmError> {
        FixedU128::checked_from_integer(value).ok_or(CurveAmmError::ConversionError)
    }

    pub async fn get_pool_balances(assets: &[ActorId]) -> Result<Vec<FixedU128>, CurveAmmError> {
        let mut balances = Vec::new();
        let program_id = program_id();
        let mut requests: Vec<_> = assets
            .iter()
            .map(|asset| {
                msg::send_and_wait_for_reply(
                    *asset,
                    &Action::BalanceOf(program_id),
                    100_000_000_000,
                    0,
                )
            })
            .collect();
        while !requests.is_empty() {
            let (result, _, remaining) = future::select_all(requests).await;
            let reply = result.map_err(CurveAmmError::BalaceOfFailed)?;
            let asset_balance = match reply {
                Event::Balance(bal) => Self::u128_to_fixed(bal)?,
                _ => {
                    return Err(CurveAmmError::DecodeError);
                }
            };
            balances.push(asset_balance);
            requests = remaining;
        }
        Ok(balances)
    }

    async fn transfer_funds_to_pool(
        who: &ActorId,
        amounts: Vec<FixedU128>,
        assets: &[ActorId],
    ) -> Result<(), CurveAmmError> {
        let zero = FixedU128::zero();
        for (i, amount) in amounts.iter().enumerate() {
            if amount > &zero {
                let amount_u = Self::fixed_to_u128(amount);
                let _lock = MUTEX.lock().await; // will be dropped automatically on function return
                msg::send_and_wait_for_reply(
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
                .map_err(CurveAmmError::TransferFailed)?;
            }
        }
        Ok(())
    }

    async fn transfer_funds_from_pool(
        who: &ActorId,
        amounts: Vec<FixedU128>,
        assets: &[ActorId],
    ) -> Result<(), CurveAmmError> {
        let zero = FixedU128::zero();
        for (i, amount) in amounts.iter().enumerate() {
            if amount > &zero {
                let amount = Self::fixed_to_u128(amount);
                let _lock = MUTEX.lock().await; // will be dropped automatically on function return
                msg::send_and_wait_for_reply(
                    assets[i],
                    &Action::Transfer(TransferInput { to: *who, amount }),
                    100_000_000_000,
                    0,
                )
                .await
                .map_err(CurveAmmError::TransferFailed)?;
            }
        }
        Ok(())
    }

    pub fn get_pool(&self, pool_id: &PoolId) -> Result<&PoolInfo, CurveAmmError> {
        self.pools.get(pool_id).ok_or(CurveAmmError::PoolNotFound)
    }

    pub async fn get_lp_token_suppy(&self, pool_id: &PoolId) -> Result<FixedU128, CurveAmmError> {
        let pool = self.get_pool(pool_id)?;
        let _lock = MUTEX.lock().await; // will be dropped automatically on function return
        let reply =
            msg::send_and_wait_for_reply(pool.pool_asset, &Action::TotalSupply, 100_000_000_000, 0)
                .await
                .map_err(CurveAmmError::TotalSupplyFailed)?;
        let token_supply = match reply {
            Event::TotalSupply(bal) => Self::u128_to_fixed(bal)?,
            _ => {
                return Err(CurveAmmError::DecodeError);
            }
        };
        Ok(token_supply)
    }

    #[allow(dead_code)]
    async fn add_liquidity(
        &mut self,
        who: &ActorId,
        pool_id: PoolId,
        amounts: Vec<FixedU128>,
        min_mint_amount: FixedU128,
    ) -> Result<(), CurveAmmError> {
        let zero = FixedU128::zero();
        if !amounts.iter().all(|&x| x > zero) {
            return Err(CurveAmmError::WrongAssetAmount);
        }
        let pool = self.get_pool(&pool_id)?;
        let n_coins = pool.assets.len();
        if n_coins != amounts.len() {
            return Err(CurveAmmError::NotEnoughAssets);
        }
        let ann = get_ann(pool.amplification_coefficient, n_coins).ok_or(CurveAmmError::Math)?;
        let old_balances = Self::get_pool_balances(&pool.assets).await?;
        let d0 = get_d(&old_balances, ann).ok_or(CurveAmmError::Math)?;
        let token_supply = self.get_lp_token_suppy(&pool_id).await?;
        let mut new_balances = old_balances.clone();
        for i in 0..n_coins {
            new_balances[i] = new_balances[i]
                .checked_add(&amounts[i])
                .ok_or(CurveAmmError::Math)?;
        }
        let d1 = get_d(&new_balances, ann).ok_or(CurveAmmError::Math)?;
        if d1 <= d0 {
            return Err(CurveAmmError::WrongAssetAmount);
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
            let n_coins_f = Self::u128_to_fixed(n_coins as u128)?;
            let fee_f: FixedU128 = pool.fee.into();
            let n_coins_1 = n_coins_f.checked_sub(&one).ok_or(CurveAmmError::Math)?;
            let four_n_coins_1 = four.checked_mul(&n_coins_1).ok_or(CurveAmmError::Math)?;
            let pool_fees_n_coins = fee_f.checked_mul(&n_coins_f).ok_or(CurveAmmError::Math)?;
            let fee_f = pool_fees_n_coins
                .checked_div(&four_n_coins_1)
                .ok_or(CurveAmmError::Math)?;
            // let admin_fee_f: FixedU128 = pool.admin_fee.into();
            for i in 0..n_coins {
                let ideal_balance = d1
                    .checked_mul(&old_balances[i])
                    .and_then(|v| v.checked_div(&d0))
                    .ok_or(CurveAmmError::Math)?;

                let new_balance = new_balances[i];
                // difference = abs(ideal_balance - new_balance)
                let difference = if ideal_balance > new_balance {
                    ideal_balance
                        .checked_sub(&new_balance)
                        .ok_or(CurveAmmError::Math)?
                } else {
                    new_balance
                        .checked_sub(&ideal_balance)
                        .ok_or(CurveAmmError::Math)?
                };
                fees[i] = fee_f.checked_mul(&difference).ok_or(CurveAmmError::Math)?;
                new_balances[i] = new_balances[i]
                    .checked_sub(&fees[i])
                    .ok_or(CurveAmmError::Math)?;
            }
            let d2 = get_d(&new_balances, ann).ok_or(CurveAmmError::Math)?;

            mint_amount = (|| {
                token_supply
                    .checked_mul(&d2.checked_sub(&d0)?)?
                    .checked_div(&d0)
            })()
            .ok_or(CurveAmmError::Math)?;
        } else {
            mint_amount = d1;
        }
        if mint_amount < min_mint_amount {
            return Err(CurveAmmError::RequiredAmountNotReached);
        }

        {
            let _lock = MUTEX.lock().await;
            // Ensure that for all tokens user has sufficient amount
            for (i, amount) in amounts.iter().enumerate() {
                let reply = msg::send_and_wait_for_reply(
                    pool.assets[i],
                    &Action::BalanceOf(*who),
                    100_000_000_000,
                    0,
                )
                .await
                .map_err(CurveAmmError::BalaceOfFailed)?;
                let balance = match reply {
                    Event::Balance(bal) => bal,
                    _ => {
                        return Err(CurveAmmError::DecodeError);
                    }
                };
                let balance = Self::u128_to_fixed(balance)?;
                if balance < *amount {
                    return Err(CurveAmmError::InsufficientFunds);
                }
            }
        } // lock will be dropped here or on return

        // Transfer funds to pool
        Self::transfer_funds_to_pool(who, amounts, &pool.assets).await?;
        let mint_amount = Self::fixed_to_u128(&mint_amount);

        let _lock = MUTEX.lock().await;
        msg::send_and_wait_for_reply(
            pool.pool_asset,
            &Action::Mint(MintInput {
                account: *who,
                amount: mint_amount,
            }),
            100_000_000_000,
            0,
        )
        .await
        .map_err(CurveAmmError::MintFailed)?;

        let add_liquidity_reply = CurveAmmAddLiquidityReply {
            who: *who,
            pool_id,
            mint_amount,
        };

        msg::reply(CurveAmmReply::AddLiquidity(add_liquidity_reply), 0, 0);
        Ok(())
    }

    #[allow(dead_code)]
    async fn remove_liquidity(
        &mut self,
        who: &ActorId,
        pool_id: PoolId,
        amount: FixedU128,
    ) -> Result<(), CurveAmmError> {
        let zero = FixedU128::zero();
        if amount <= zero {
            return Err(CurveAmmError::WrongAssetAmount);
        }
        let pool = self.get_pool(&pool_id)?;
        let n_coins = pool.assets.len();

        let token_supply = self.get_lp_token_suppy(&pool_id).await?;
        let old_balances = Self::get_pool_balances(&pool.assets).await?;
        let mut n_amounts = vec![FixedU128::zero(); n_coins];
        for (i, n_amount) in n_amounts.iter_mut().enumerate().take(n_coins) {
            let old_balance = old_balances[i];
            // value = old_balance * n_amount / token_supply
            let value = old_balance
                .checked_mul(&amount)
                .and_then(|v| v.checked_div(&token_supply))
                .ok_or(CurveAmmError::Math)?;
            *n_amount = value;
        }
        let burn_amount = Self::fixed_to_u128(&amount);
        {
            let _lock = MUTEX.lock().await;
            msg::send_and_wait_for_reply(
                pool.pool_asset,
                &Action::Burn(BurnInput {
                    account: *who,
                    amount: burn_amount,
                }),
                100_000_000_000,
                0,
            )
            .await
            .map_err(CurveAmmError::BurnFailed)?;

            for (i, n_amount) in n_amounts.iter_mut().enumerate().take(n_coins) {
                let reply = msg::send_and_wait_for_reply(
                    pool.assets[i],
                    &Action::BalanceOf(*who),
                    100_000_000_000,
                    0,
                )
                .await
                .map_err(CurveAmmError::BalaceOfFailed)?;
                let balance = match reply {
                    Event::Balance(bal) => bal,
                    _ => {
                        return Err(CurveAmmError::DecodeError);
                    }
                };
                let balance = Self::u128_to_fixed(balance)?;
                if balance < *n_amount {
                    return Err(CurveAmmError::InsufficientFunds);
                }
            }
        } // lock will be dropped here or on return

        // Transfer funds from pool
        let amounts = n_amounts.iter().map(Self::fixed_to_u128).collect();
        Self::transfer_funds_from_pool(who, n_amounts, &pool.assets).await?;

        let remove_liquidity_reply = CurveAmmRemoveLiquidityReply {
            who: *who,
            pool_id,
            amounts,
        };

        msg::reply(CurveAmmReply::RemoveLiquidity(remove_liquidity_reply), 0, 0);
        Ok(())
    }

    #[allow(dead_code)]
    async fn exchange(
        &mut self,
        who: &ActorId,
        pool_id: PoolId,
        i_u: u32,
        j_u: u32,
        dx: FixedU128,
    ) -> Result<(), CurveAmmError> {
        let i = i_u as usize;
        let j = j_u as usize;
        let prec = get_precision();
        let zero = FixedU128::zero();
        if dx < zero {
            return Err(CurveAmmError::WrongAssetAmount);
        }
        let pool = self.get_pool(&pool_id)?;
        let amp_coeff = pool.amplification_coefficient;
        let n_coins = pool.assets.len();
        if i >= n_coins && j >= n_coins {
            return Err(CurveAmmError::IndexOutOfRange);
        }
        let old_balances = Self::get_pool_balances(&pool.assets).await?;
        let xp = old_balances.clone();
        let x = xp[i].checked_add(&dx).ok_or(CurveAmmError::Math)?;
        let ann = get_ann(amp_coeff, n_coins).ok_or(CurveAmmError::Math)?;
        let y = get_y(i, j, x, &xp, ann).ok_or(CurveAmmError::Math)?;
        let dy = xp[j]
            .checked_sub(&y)
            .and_then(|v| v.checked_sub(&prec))
            .ok_or(CurveAmmError::Math)?;

        let pool = self.get_pool(&pool_id)?;

        let _lock = MUTEX.lock().await; // will be dropped on function return
        let reply = msg::send_and_wait_for_reply(
            pool.assets[i],
            &Action::BalanceOf(*who),
            100_000_000_000,
            0,
        )
        .await
        .map_err(CurveAmmError::BalaceOfFailed)?;
        let balance = match reply {
            Event::Balance(bal) => bal,
            _ => {
                return Err(CurveAmmError::DecodeError);
            }
        };
        let balance = Self::u128_to_fixed(balance)?;
        if balance < dx {
            return Err(CurveAmmError::InsufficientFunds);
        }
        let reply = msg::send_and_wait_for_reply(
            pool.assets[j],
            &Action::BalanceOf(exec::program_id()),
            100_000_000_000,
            0,
        )
        .await
        .map_err(CurveAmmError::BalaceOfFailed)?;
        let balance = match reply {
            Event::Balance(bal) => bal,
            _ => {
                return Err(CurveAmmError::DecodeError);
            }
        };
        let balance = Self::u128_to_fixed(balance)?;
        let amount = Self::fixed_to_u128(&dx);
        if balance < dy {
            return Err(CurveAmmError::InsufficientFunds);
        }

        msg::send_and_wait_for_reply(
            pool.assets[i],
            &Action::TransferFrom(TransferFromInput {
                owner: *who,
                to: exec::program_id(),
                amount,
            }),
            100_000_000_000,
            0,
        )
        .await
        .map_err(CurveAmmError::TransferFromFailed)?;
        let amount = Self::fixed_to_u128(&dy);
        msg::send_and_wait_for_reply(
            pool.assets[j],
            &Action::Transfer(TransferInput { to: *who, amount }),
            100_000_000_000,
            0,
        )
        .await
        .map_err(CurveAmmError::TransferFailed)?;
        let exchange_reply = CurveAmmExchangeReply {
            who: *who,
            pool_id,
            i: i_u,
            j: j_u,
            dy_amount: amount,
        };

        msg::reply(CurveAmmReply::Exchange(exchange_reply), 0, 0);
        Ok(())
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
    let assets = vec![config.token_x_id, config.token_y_id];
    let amplification_coefficient =
        FixedU128::checked_from_integer(config.amplification_coefficient)
            .expect("conversion error");
    let fee = Permill::from_percent(config.fee);
    let admin_fee = Permill::from_percent(config.admin_fee);
    let _res = CURVE_AMM
        .create_pool(
            &owner,
            assets,
            &config.token_lp_id,
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
                amounts_f.push(FixedU128::checked_from_integer(amount).expect("conversion error"));
            }
            unsafe {
                let res = CURVE_AMM
                    .add_liquidity(&sender, pool_id, amounts_f, FixedU128::zero())
                    .await;
                if let Err(e) = res {
                    panic!("add_liquidity failed with {:?}", e);
                }
            }
        }
        CurveAmmAction::RemoveLiquidity(remove_liquidity) => {
            let sender = msg::source();
            let pool_id: PoolId = remove_liquidity.pool_id;
            let amount_f =
                FixedU128::checked_from_integer(remove_liquidity.amount).expect("conversion error");
            unsafe {
                let res = CURVE_AMM.remove_liquidity(&sender, pool_id, amount_f).await;
                if let Err(e) = res {
                    panic!("remove_liquidity failed with {:?}", e);
                }
            }
        }
        CurveAmmAction::Exchange(exchange) => {
            let sender = msg::source();
            let pool_id: PoolId = exchange.pool_id;
            let i = exchange.i;
            let j = exchange.j;
            let dx_amount_f =
                FixedU128::checked_from_integer(exchange.dx_amount).expect("conversion error");
            unsafe {
                let res = CURVE_AMM
                    .exchange(&sender, pool_id, i, j, dx_amount_f)
                    .await;
                if let Err(e) = res {
                    panic!("exchange failed with {:?}", e);
                }
            }
        }
    }
}
