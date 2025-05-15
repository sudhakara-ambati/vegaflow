mod models;
mod data_fetch;

use models::black_scholes::{black_scholes_call, black_scholes_put};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fred_api_key = ""; // fred API key

    let symbol = "AAPL"; // stock symbol
    let expiry = 1789401600; // expiry date
    let s = data_fetch::fetch_stock_price(symbol).await?; // fetch stock price
    let k = 95.0;       // strike price
    let t = 0.00821918; // time to maturity in years
    let r = data_fetch::fetch_risk_free_rate(fred_api_key) // risk-free rate
            .await
            .expect("Failed to fetch risk-free rate");
    let calculated_iv = data_fetch::predict_iv(symbol, k, expiry).await?;


    let call_price = black_scholes_call(s, k, t, r, sigma);
    let put_price = black_scholes_put(s, k, t, r, sigma);

    println!("Stock price: {}", s);
    println!("Average IV: {}", calculated_iv);
    println!("Black-Scholes Call Price: {:.4}", call_price);
    println!("Black-Scholes Put Price:  {:.4}", put_price);

    Ok(())
}

//todo
//add stochastic model to "predict_iv" stock price
//add fetching IV for put table
//possibly start on web frontend