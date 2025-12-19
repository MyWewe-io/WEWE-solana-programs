use anchor_lang::prelude::*;
use ruint::aliases::{U256, U512};

use crate::ProposalError;

// L = Δx * sqrt(P) * sqrt(P_upper) / (sqrt(P_upper) - sqrt(P))
fn get_initial_liquidity_from_delta_base(
    base_amount: u64,
    sqrt_max_price: u128,
    sqrt_price: u128,
) -> Result<U512> {
    let delta = sqrt_max_price
        .checked_sub(sqrt_price)
        .ok_or(ProposalError::NumericalOverflow)?;
    let price_delta = U512::from(delta);

    let base = U512::from(base_amount);
    let sqrt_price = U512::from(sqrt_price);
    let sqrt_max_price = U512::from(sqrt_max_price);

    let prod = base
        .checked_mul(sqrt_price)
        .ok_or(ProposalError::NumericalOverflow)?
        .checked_mul(sqrt_max_price)
        .ok_or(ProposalError::NumericalOverflow)?;

    let liquidity = prod
        .checked_div(price_delta)
        .ok_or(ProposalError::NumericalOverflow)?;

    Ok(liquidity)
}

// L = Δy * 2^128 / (sqrt(P) - sqrt(P_lower))
fn get_initial_liquidity_from_delta_quote(
    quote_amount: u64,
    sqrt_min_price: u128,
    sqrt_price: u128,
) -> Result<u128> {
    msg!("get_initial_liquidity_from_delta_quote: quote={}, sqrt_min={}, sqrt_price={}", 
         quote_amount, sqrt_min_price, sqrt_price);
    
    let delta = sqrt_price
        .checked_sub(sqrt_min_price)
        .ok_or(ProposalError::NumericalOverflow)?;
    msg!("delta (sqrt_price - sqrt_min_price): {}", delta);
    
    if delta == 0 {
        msg!("ERROR: delta is ZERO! sqrt_price ({}) == sqrt_min_price ({})", sqrt_price, sqrt_min_price);
        return Err(ProposalError::InvalidPriceRange.into());
    }
    
    let price_delta = U256::from(delta);

    let quote = U256::from(quote_amount);
    let quote_shifted = quote
        .checked_shl(128)
        .ok_or(ProposalError::NumericalOverflow)?;
    msg!("quote_shifted (quote << 128): {}", quote_shifted);

    let liquidity = quote_shifted
        .checked_div(price_delta)
        .ok_or(ProposalError::NumericalOverflow)?;
    msg!("liquidity (quote_shifted / price_delta): {}", liquidity);

    let result = liquidity.to::<u128>();
    msg!("liquidity as u128: {}", result);
    return Ok(result)
}

pub fn get_liquidity_for_adding_liquidity(
    base_amount: u64,
    quote_amount: u64,
    sqrt_price: u128,
    min_sqrt_price: u128,
    max_sqrt_price: u128,
) -> Result<u128> {
    msg!("get_liquidity_for_adding_liquidity: base={}, quote={}, sqrt_price={}, min={}, max={}", 
         base_amount, quote_amount, sqrt_price, min_sqrt_price, max_sqrt_price);
    
    let liquidity_from_base =
        get_initial_liquidity_from_delta_base(base_amount, max_sqrt_price, sqrt_price)?;
    msg!("liquidity_from_base (U512): {}", liquidity_from_base);
    
    let liquidity_from_quote =
        get_initial_liquidity_from_delta_quote(quote_amount, min_sqrt_price, sqrt_price)?;
    msg!("liquidity_from_quote (u128): {}", liquidity_from_quote);
    
    let result = if liquidity_from_base > U512::from(liquidity_from_quote) {
        msg!("Using liquidity_from_quote (smaller): {}", liquidity_from_quote);
        Ok(liquidity_from_quote)
    } else {
        let liquidity_u128 = liquidity_from_base
            .try_into()
            .map_err(|_| ProposalError::TypeCastFailed)?;
        msg!("Using liquidity_from_base (smaller): {}", liquidity_u128);
        Ok(liquidity_u128)
    };
    
    msg!("Final liquidity result: {}", result.as_ref().unwrap_or(&0));
    result
}