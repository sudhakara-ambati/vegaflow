use statrs::distribution::{Normal, ContinuousCDF};
use rand::Rng;

pub fn monte_carlo_option_price(s0: f64, k: f64, t: f64, r: f64, sigma: f64, option_type: &str, n: usize,) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let drift = (r - 0.5 * sigma * sigma) * t;
    let diffusion = sigma * t.sqrt();
    let mut rng = rand::thread_rng();

    let payoffs: f64 = (0..n)
        .map(|_| {
            let quantile = rng.gen::<f64>();
            let z = normal.inverse_cdf(quantile);
            let st = s0 * (drift + diffusion * z).exp();
            match option_type {
                "call" => (st - k).max(0.0),
                "put" => (k - st).max(0.0),
                _ => 0.0,
            }
        })
        .sum();

    let expected_payoff = payoffs / n as f64;
    (-(r * t)).exp() * expected_payoff
}