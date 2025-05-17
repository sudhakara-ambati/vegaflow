use statrs::distribution::{Normal, ContinuousCDF};
use rand::prelude::*;
use crate::visualisations::visualisations::plot_stock_paths;

pub fn monte_carlo_option_price(
    s0: f64,
    k: f64,
    t: f64,
    r: f64,
    sigma: f64,
    option_type: &str,
    n: usize
) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let drift = (r - 0.5 * sigma * sigma) * t;
    let diffusion = sigma * t.sqrt();
    let mut rng = rand::rng();

    let num_paths_to_plot = 50.min(n);
    let num_steps = 100;
    let mut paths = Vec::with_capacity(num_paths_to_plot);
    let dt = t / num_steps as f64;
    
    for _ in 0..num_paths_to_plot {
        let mut path = Vec::with_capacity(num_steps + 1);
        path.push(s0);
        let mut current_price = s0;
        
        for _ in 0..num_steps {
            let z = normal.inverse_cdf(rng.random::<f64>());
            current_price *= ((r - 0.5 * sigma * sigma) * dt + sigma * z * dt.sqrt()).exp();
            path.push(current_price);
        }
        
        paths.push(path);
    }
    
    plot_stock_paths(s0, k, t, paths).expect("Failed to plot stock paths");

    let payoffs: f64 = (0..n)
        .map(|_| {
            let quantile = rng.random::<f64>();
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