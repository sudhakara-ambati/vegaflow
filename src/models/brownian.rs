use statrs::distribution::{Normal, ContinuousCDF};

pub fn gbm_price(s0: f64, mu: f64, sigma: f64, t: f64, quantile: f64) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let z = normal.inverse_cdf(quantile);
    s0 * ((mu - 0.5 * sigma * sigma) * t + sigma * t.sqrt() * z).exp()
}