use anchor_lang::prelude::*;
use damm_v2_cpi::curve::{get_delta_amount_a_unsigned, get_delta_amount_b_unsigned};
use damm_v2_cpi::u128x128_math::Rounding;


/// Gets the delta amount_a for given liquidity and price range
///
/// # Formula
///
/// * `Δa = L * (1 / √P_lower - 1 / √P_upper)`
/// * i.e. `L * (√P_upper - √P_lower) / (√P_upper * √P_lower)`


/// Gets the delta amount_b for given liquidity and price range
/// Δb = L * (√P_upper - √P_lower)
 

pub fn get_liquidity_for_adding_liquidity(
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
        get_delta_amount_a_unsigned(sqrt_price, max_sqrt_price, base_amount as u128, Rounding::Up)?;
    let liquidity_from_quote =
        get_delta_amount_b_unsigned(min_sqrt_price, sqrt_price, quote_amount as u128, Rounding::Up)?;

    msg!("liquidity_from_base: {}", liquidity_from_base);
    msg!("liquidity_from_quote: {}", liquidity_from_quote);
    
    Ok(std::cmp::min(liquidity_from_base as u128, liquidity_from_quote as u128))
}