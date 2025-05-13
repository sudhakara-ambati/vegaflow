use statrs::function::erf::erfc;

fn norm_cdf(x: f64) -> f64 {
    0.5 * erfc(-x / (2.0_f64).sqrt())
}

pub fn black_scholes_call(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    let d1 = (f64::ln(s / k) + (r + 0.5 * sigma * sigma) * t) / (sigma * f64::sqrt(t));
    let d2 = d1 - sigma * f64::sqrt(t);
    s * norm_cdf(d1) - k * f64::exp(-r * t) * norm_cdf(d2)
}

pub fn black_scholes_put(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    let d1 = (f64::ln(s / k) + (r + 0.5 * sigma * sigma) * t) / (sigma * f64::sqrt(t));
    let d2 = d1 - sigma * f64::sqrt(t);
    k * f64::exp(-r * t) * norm_cdf(-d2) - s * norm_cdf(-d1)
}
