mod models;
mod data_fetch;

use models::black_scholes::{black_scholes_call, black_scholes_put};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fred_api_key = ""; // fred API key

    let s = 212.27;     // spot price
    let k = 95.0;       // strike price
    let t = 0.00821918; // time to maturity in years
    let r = data_fetch::fetch_risk_free_rate(fred_api_key) // risk-free rate
            .await
            .expect("Failed to fetch risk-free rate");
    let sigma = 3.1797; // volatility

    let call_price = black_scholes_call(s, k, t, r, sigma);
    let put_price = black_scholes_put(s, k, t, r, sigma);
    let avg_iv = data_fetch::print_all_expiry_ivs("AAPL", 140.0).await?;

    println!("Black-Scholes Call Price: {:.4}", call_price);
    println!("Black-Scholes Put Price:  {:.4}", put_price);

    Ok(())
}