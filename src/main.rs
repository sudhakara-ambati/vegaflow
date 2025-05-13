mod models;
mod data_fetch;

use models::black_scholes::{black_scholes_call, black_scholes_put};

fn main() {
    let s = 212.27;     // spot price
    let k = 95.0;     // strike price
    let t = 0.00821918;       // time to maturity in years
    let r = 0.05;      // risk-free rate
    let sigma = 3.1797;   // volatility

    let call_price = black_scholes_call(s, k, t, r, sigma);
    let put_price = black_scholes_put(s, k, t, r, sigma);

    println!("Black-Scholes Call Price: {:.4}", call_price);
    println!("Black-Scholes Put Price:  {:.4}", put_price);
}
