use std::collections::HashMap;

use anyhow::Result;
use solana_transaction_status::{
    TransactionTokenBalance, UiTransactionTokenBalance,
};

use crate::constants::WSOL_MINT_KEY_STR;

pub trait TokenBalanceInfo {
    fn get_mint(&self) -> &str;
    fn get_ui_amount(&self) -> Option<f64>;
    fn get_owner(&self) -> &str;
}

impl TokenBalanceInfo for TransactionTokenBalance {
    fn get_mint(&self) -> &str {
        &self.mint
    }

    fn get_ui_amount(&self) -> Option<f64> {
        self.ui_token_amount.ui_amount
    }

    fn get_owner(&self) -> &str {
        &self.owner
    }
}

impl TokenBalanceInfo for UiTransactionTokenBalance {
    fn get_mint(&self) -> &str {
        &self.mint
    }

    fn get_ui_amount(&self) -> Option<f64> {
        self.ui_token_amount.ui_amount
    }

    fn get_owner(&self) -> &str {
        self.owner.as_ref().map(|s| s.as_str()).unwrap_or_default()
    }
}

pub fn process_diffs(
    diffs: &Vec<Diff>,
    sol_price: f64,
) -> Result<(f64, f64, String)> {
    let (token0, token1) = (&diffs[0], &diffs[1]);

    let amount0 = token0.diff.abs();
    let amount1 = token1.diff.abs();

    let (sol_amount, token_amount, coin_mint) =
        match (token0.mint.as_str(), token1.mint.as_str()) {
            (WSOL_MINT_KEY_STR, other_mint) => (amount0, amount1, other_mint),
            (other_mint, WSOL_MINT_KEY_STR) => (amount1, amount0, other_mint),
            _ => return Err(anyhow::anyhow!("Non-WSOL swap")),
        };

    let price = (sol_amount.abs() / token_amount.abs()) * sol_price;
    let swap_amount = sol_amount * sol_price;

    Ok((price, swap_amount, coin_mint.to_string()))
}
#[derive(Debug)]
pub struct Diff {
    pub mint: String,
    pub pre_amount: f64,
    pub post_amount: f64,
    pub diff: f64,
    pub owner: String,
}

pub fn get_token_balance_diff<T: TokenBalanceInfo + std::fmt::Debug>(
    pre_balances: &[T],
    post_balances: &[T],
) -> Vec<Diff> {
    let mut diffs = Vec::new();
    let mut pre_balances_map = HashMap::new();
    let mut post_balances_map = HashMap::new();

    println!("pre_balances: {:#?}", pre_balances);
    println!("post_balances: {:#?}", post_balances);

    for balance in pre_balances {
        if let Some(amount) = balance.get_ui_amount() {
            pre_balances_map.insert(
                balance.get_mint().to_string(),
                (amount, balance.get_owner().to_string()),
            );
        }
    }

    for balance in post_balances {
        if let Some(amount) = balance.get_ui_amount() {
            post_balances_map.insert(
                balance.get_mint().to_string(),
                (amount, balance.get_owner().to_string()),
            );
        }
    }

    for (mint, (pre_amount, owner)) in pre_balances_map {
        if let Some((post_amount, _)) = post_balances_map.get(&mint) {
            let diff = post_amount - pre_amount;
            diffs.push(Diff {
                mint,
                pre_amount,
                post_amount: *post_amount,
                diff,
                owner,
            });
        }
    }

    diffs
}
