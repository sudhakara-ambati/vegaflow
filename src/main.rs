mod models;
mod data_fetch;
mod visualisations;
use data_fetch::{fetch_stock_price, fetch_risk_free_rate, predict_iv};
use models::black_scholes::{black_scholes_call, black_scholes_put};
use models::monte_carlo::monte_carlo_option_price;
use std::time::{SystemTime, UNIX_EPOCH};

fn time_to_maturity_in_years(expiry_unix: u64) -> f64 {
    let now_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let seconds_to_expiry = if expiry_unix > now_unix {
        expiry_unix - now_unix
    } else {
        0
    };
    seconds_to_expiry as f64 / (365.25 * 24.0 * 60.0 * 60.0)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fred_api_key = ""; // fred API key

    let option_type = "call"; // option type
    let symbol = "AAPL"; // stock symbol
    let expiry = 1750118762; // expiry date
    let current_stock = fetch_stock_price(symbol).await?; // fetch stock price
    let k = 95.0;       // strike price
    let t = time_to_maturity_in_years(expiry); // time to maturity in years
    let r = fetch_risk_free_rate(fred_api_key) // risk-free rate
            .await
            .expect("Failed to fetch risk-free rate");
    let calculated_iv = predict_iv(symbol, k, expiry, option_type).await?;

    if option_type == "call" {
        let call_price_black_scholes = black_scholes_call(current_stock, k, t, r, calculated_iv);
        let call_price_monte_carlo = monte_carlo_option_price(current_stock, k, t, r, calculated_iv, option_type, 5000000);
        println!("Black-Scholes Call Price: {:.9}", call_price_black_scholes);
        println!("Monte-Carlo Call Price: {:.9}", call_price_monte_carlo);
    }
    if option_type == "put" {
        let put_price_black_scholes = black_scholes_put(current_stock, k, t, r, calculated_iv);
        let put_price_monte_carlo = monte_carlo_option_price(current_stock, k, t, r, calculated_iv, option_type, 5000000);
        println!("Black-Scholes Put Price: {:.9}", put_price_black_scholes);
        println!("Monte-Carlo Put Price: {:.9}", put_price_monte_carlo);
    }
    Ok(())
}

//todo
//possibly start on web frontend
