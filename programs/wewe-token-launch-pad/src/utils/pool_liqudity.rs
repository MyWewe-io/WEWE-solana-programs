use anchor_lang::prelude::*;
use damm_v2_cpi::safe_math::SafeMath;
use damm_v2_cpi::PoolError;
use ruint::aliases::{U256, U512};

// Δa = L * (1 / √P_lower - 1 / √P_upper) => L = Δa / (1 / √P_lower - 1 / √P_upper)
pub fn get_initial_liquidity_from_delta_base(
    base_amount: u64,
    sqrt_price: u128,
    sqrt_max_price: u128,
) -> Result<U512> {
    let price_delta = U512::from(sqrt_max_price.safe_sub(sqrt_price)?);
    let prod = U512::from(base_amount)
        .safe_mul(U512::from(sqrt_price))?
        .safe_mul(U512::from(sqrt_max_price))?;
    let liquidity = prod.safe_div(price_delta)?; // round down
    Ok(liquidity)
}

// Δb = L (√P_upper - √P_lower) => L = Δb / (√P_upper - √P_lower)
pub fn get_initial_liquidity_from_delta_quote(
    quote_amount: u64,
    sqrt_min_price: u128,
    sqrt_price: u128,
) -> Result<u128> {
    let price_delta = U256::from(sqrt_price.safe_sub(sqrt_min_price)?);
    let quote_amount = U256::from(quote_amount).safe_shl(128)?;
    let liquidity = quote_amount.safe_div(price_delta)?; // round down
    return Ok(liquidity
        .try_into()
        .map_err(|_| PoolError::TypeCastFailed)?);
}

pub fn get_liquidity_delta(
    base_amount: u64,
    quote_amount: u64,
    sqrt_price: u128,
    min_sqrt_price: u128,
    max_sqrt_price: u128,
) -> Result<u128> {
    msg!("base_amount: {}", base_amount);
    msg!("quote_amount: {}", quote_amount);
    msg!("sqrt_price: {}", sqrt_price);
    msg!("min_sqrt_price: {}", min_sqrt_price);
    msg!("max_sqrt_price: {}", max_sqrt_price);

    let liquidity_from_base =
        get_initial_liquidity_from_delta_base(base_amount, sqrt_price, max_sqrt_price)?;

    let liquidity_from_quote =
        get_initial_liquidity_from_delta_quote(quote_amount, min_sqrt_price, sqrt_price)?;

    msg!("liquidity_from_base: {}", liquidity_from_base);
    msg!("liquidity_from_quote: {}", liquidity_from_quote);

    Ok(std::cmp::min(
        liquidity_from_base
            .try_into()
            .map_err(|_| PoolError::TypeCastFailed)?,
        liquidity_from_quote
            .try_into()
            .map_err(|_| PoolError::TypeCastFailed)?,
    ))
}
