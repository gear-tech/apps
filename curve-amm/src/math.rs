use primitive_types::U256;

#[allow(dead_code)]
pub fn get_precision() -> U256 {
    1.into()
}

/// Find `ann = amp * n^n` where `amp` - amplification coefficient,
/// `n` - number of coins.
#[allow(dead_code)]
pub fn get_ann(amp: U256, n: usize) -> Option<U256> {
    let n_coins = (n as u128).into();
    let mut ann = amp;
    for _ in 0..n {
        ann = ann.checked_mul(n_coins)?;
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
pub fn get_d(xp_f: &[U256], ann_f: U256) -> Option<U256> {
    let zero = U256::zero();
    let one = U256::one();
    let prec = get_precision();
    let n = (u128::try_from(xp_f.len()).ok()?).into();
    let sum = xp_f.iter().try_fold(zero, |s, x| s.checked_add(*x))?;
    if sum == zero {
        return Some(zero);
    }
    let mut d = sum;

    for _ in 0..255 {
        let mut d_p = d;
        for x in xp_f.iter() {
            // d_p = d_p * d / (x * n)
            d_p = d_p.checked_mul(d.checked_div(x.checked_mul(n)?)?)?;
        }
        let d_prev = d;

        // d = (ann * sum + d_p * n) * d / ((ann - 1) * d + (n + 1) * d_p)
        let term1 = ann_f.checked_mul(sum)?.checked_add(d_p.checked_mul(n)?)?;
        let term2 = ann_f
            .checked_sub(one)?
            .checked_mul(d)?
            .checked_add(n.checked_add(one)?.checked_mul(d_p)?)?;
        d = term1.checked_div(term2)?.checked_mul(d)?;
        if d > d_prev {
            let v = d.checked_sub(d_prev)?;
            if v <= prec {
                return Some(d);
            }
        } else {
            let v = d_prev.checked_sub(d)?;
            if v <= prec {
                return Some(d);
            }
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
pub fn get_y(i: usize, j: usize, x_f: U256, xp_f: &[U256], ann_f: U256) -> Option<U256> {
    let zero = U256::zero();
    let two: U256 = 2_u128.into();
    let n = (xp_f.len() as u128).into();

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
    let d_f = get_d(xp_f, ann_f)?;
    let mut c = d_f;
    let mut s = zero;

    // Calculate s and c
    // p is implicitly calculated as part of c
    // note that loop makes n - 1 iterations
    for (k, xp_k) in xp_f.iter().enumerate() {
        let x_k: U256;
        if k == i {
            x_k = x_f;
        } else if k != j {
            x_k = *xp_k;
        } else {
            continue;
        }
        // s = s + x_k
        s = s.checked_add(x_k)?;
        // c = c * d / (x_k * n)
        c = c.checked_mul(d_f)?.checked_div(x_k.checked_mul(n)?)?;
    }
    // c = c * d / (ann * n)
    // At this step we have d^n in the numerator of c
    // and n^(n-1) in its denominator.
    // So we multiplying it by remaining d/n
    c = c.checked_mul(d_f)?.checked_div(ann_f.checked_mul(n)?)?;

    // b = s + d / ann
    // We subtract d later
    let b = s.checked_add(d_f.checked_div(ann_f)?)?;
    let mut y = d_f;

    for _ in 0..255 {
        let y_prev = y;
        // y = (y^2 + c) / (2 * y + b - d)
        // Subtract d to calculate b finally
        y = y
            .checked_mul(y)?
            .checked_add(c)?
            .checked_div(two.checked_mul(y)?.checked_add(b)?.checked_sub(d_f)?)?;

        // Equality with the specified precision
        if y > y_prev {
            if y.checked_sub(y_prev)? <= get_precision() {
                return Some(y);
            }
        } else if y_prev.checked_sub(y)? <= get_precision() {
            return Some(y);
        }
    }

    None
}
