use plotters::prelude::*;
use statrs::distribution::{Normal, ContinuousCDF, Continuous};
use crate::models::black_scholes::{black_scholes_call, black_scholes_put};
use rand::prelude::*;
use std::ops::Range;

pub fn plot_pnl_distribution(
    s0: f64,
    k: f64,
    t: f64,
    r: f64,
    sigma: f64,
    option_type: &str,
    num_simulations: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("pnl_distribution.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let dt = t / 252.0;
    let drift = r - 0.5 * sigma * sigma;
    let vol = sigma * dt.sqrt();

    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = rand::rng();

    let initial_price = match option_type {
        "call" => black_scholes_call(s0, k, t, r, sigma),
        _ => black_scholes_put(s0, k, t, r, sigma),
    };

    let mut pnls = Vec::with_capacity(num_simulations);

    for _ in 0..num_simulations {
        let mut price = s0;
        for _ in 0..252 {
            let u: f64 = rng.random::<f64>();
            let z = normal.inverse_cdf(u);
            price *= (drift * dt + vol * z).exp();
        }
        let payoff = match option_type {
            "call" => (price - k).max(0.0),
            _ => (k - price).max(0.0),
        };
        let pnl = payoff - initial_price;
        pnls.push(pnl);
    }

    let min_pnl = pnls.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_pnl = pnls.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let num_bins = 50;
    let bin_width = (max_pnl - min_pnl) / num_bins as f64;
    let mut histogram = vec![0; num_bins];
    for pnl in &pnls {
        let bin = ((pnl - min_pnl) / bin_width).floor() as usize;
        let bin = bin.min(num_bins - 1);
        histogram[bin] += 1;
    }

    let prob_profit = pnls.iter().filter(|&&x| x > 0.0).count() as f64 / pnls.len() as f64;
    let avg_pnl = pnls.iter().sum::<f64>() / pnls.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("{} Option PnL Distribution (K=${:.2})",
                        if option_type == "call" { "Call" } else { "Put" }, k),
                ("sans-serif", 30))
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(min_pnl..max_pnl, 0..(*histogram.iter().max().unwrap() + 1))?;

    chart.configure_mesh()
        .x_desc("Profit/Loss ($)")
        .y_desc("Frequency")
        .draw()?;

    let bars: Vec<(Range<f64>, u32)> = histogram.iter().enumerate()
        .map(|(i, &count)| {
            let left = min_pnl + i as f64 * bin_width;
            let right = left + bin_width;
            (left..right, count)
        })
        .collect();

    chart.draw_series(
        bars.iter().map(|(range, count)| {
            let color = if range.start < 0.0 && range.end <= 0.0 {
                RED.filled()
            } else if range.start >= 0.0 && range.end > 0.0 {
                GREEN.filled()
            } else {
                BLUE.filled()
            };
            Rectangle::new([(range.start, 0), (range.end, *count)], color)
        })
    )?;

    chart.draw_series(LineSeries::new(
        vec![(0.0, 0), (0.0, *histogram.iter().max().unwrap())],
        BLACK.stroke_width(2),
    ))?;

    chart.draw_series(std::iter::once(Text::new(
        format!("Probability of Profit: {:.1}%", prob_profit * 100.0),
        (min_pnl + (max_pnl - min_pnl) * 0.1, *histogram.iter().max().unwrap() as u32 * 9 / 10),
        ("sans-serif", 20).into_font().color(&BLACK),
    )))?;

    chart.draw_series(std::iter::once(Text::new(
        format!("Average P&L: ${:.2}", avg_pnl),
        (min_pnl + (max_pnl - min_pnl) * 0.1, *histogram.iter().max().unwrap() as u32 * 8 / 10),
        ("sans-serif", 20).into_font().color(&BLACK),
    )))?;

    println!("PnL distribution saved as pnl_distribution.png");
    Ok(())
}

pub fn plot_time_decay(
    s0: f64,
    k: f64,
    r: f64,
    sigma: f64,
    max_days: usize,
    option_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let max_days = max_days.min(60);
    
    let root = BitMapBackend::new("time_decay_curve.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut option_prices: Vec<(f64, f64)> = Vec::with_capacity(max_days + 1);
    let mut intrinsic_values: Vec<(f64, f64)> = Vec::with_capacity(max_days + 1);
    let mut time_values: Vec<(f64, f64)> = Vec::with_capacity(max_days + 1);
    
    for day in 0..=max_days {
        let actual_day = max_days - day;
        let t = actual_day as f64 / 365.0;
        
        let price = match option_type {
            "call" => black_scholes_call(s0, k, t, r, sigma),
            _ => black_scholes_put(s0, k, t, r, sigma),
        };
        
        let intrinsic = match option_type {
            "call" => (s0 - k).max(0.0),
            _ => (k - s0).max(0.0),
        };
        
        let time_value = price - intrinsic;
        
        option_prices.push((day as f64, price));
        intrinsic_values.push((day as f64, intrinsic));
        time_values.push((day as f64, time_value));
    }

    let max_price = option_prices.iter().map(|(_, p)| *p).fold(0.0, f64::max);
    let min_price = intrinsic_values.iter().map(|(_, p)| *p).fold(f64::INFINITY, f64::min);
    let price_range = max_price - min_price;
    
    let y_min = (min_price - price_range * 0.05).max(0.0);
    let y_max = max_price + price_range * 0.05;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("{} Option Time Decay (K=${:.2})", 
                        if option_type == "call" { "Call" } else { "Put" }, k), 
                ("sans-serif", 30))
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..(max_days as f64), y_min..y_max)?;

    chart.configure_mesh()
        .x_desc("Days Remaining Until Expiry")
        .y_desc("Option Price ($)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        intrinsic_values,
        BLUE.mix(0.5).stroke_width(1),
    ))?
    .label("Intrinsic Value")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.mix(0.5).stroke_width(1)));

    chart.draw_series(LineSeries::new(
        time_values,
        GREEN.stroke_width(2),
    ))?
    .label("Time Value")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN.stroke_width(2)));

    chart.draw_series(LineSeries::new(
        option_prices.clone(),
        RED.stroke_width(2),
    ))?
    .label("Option Price")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2)));

    let markers = [30, 20, 10];
    for &days in markers.iter() {
        if days <= max_days {
            let day_idx = max_days - days;
            if let Some((day, price)) = option_prices.get(day_idx) {
                chart.draw_series(std::iter::once(Circle::new(
                    (*day, *price),
                    5,
                    BLACK.filled(),
                )))?;
                
                chart.draw_series(std::iter::once(Text::new(
                    format!("{} days: ${:.2}", days, *price),
                    (*day + 2.0, *price),
                    ("sans-serif", 15).into_font().color(&BLACK),
                )))?;
            }
        }
    }

    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    println!("Time decay curve saved as time_decay_curve.png");
    Ok(())
}

pub fn plot_volatility_smile(
    strikes: Vec<f64>,
    ivs: Vec<f64>,
    current_stock: f64,
    k: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("volatility_smile.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_strike = *strikes.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max_strike = *strikes.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let min_iv = *ivs.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max_iv = *ivs.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Volatility Smile", ("sans-serif", 30))
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            (min_strike * 0.95)..(max_strike * 1.05),
            (min_iv * 0.95)..(max_iv * 1.05),
        )?;

    chart.configure_mesh()
        .x_desc("Strike Price ($)")
        .y_desc("Implied Volatility")
        .draw()?;

    chart.draw_series(
        strikes.iter().zip(ivs.iter()).map(|(&k, &iv)| {
            Circle::new((k, iv), 5, BLUE.filled())
        })
    )?;

    chart.draw_series(LineSeries::new(
        strikes.iter().zip(ivs.iter()).map(|(&k, &iv)| (k, iv)),
        &BLUE,
    ))?;

    chart.draw_series(LineSeries::new(
        vec![(current_stock, min_iv * 0.95), (current_stock, max_iv * 1.05)],
        RED.mix(0.5).stroke_width(2),
    ))?
    .label("Current Stock Price")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.mix(0.5).stroke_width(2)));

    chart.draw_series(LineSeries::new(
        vec![(k, min_iv * 0.95), (k, max_iv * 1.05)],
        GREEN.mix(0.5).stroke_width(2),
    ))?
    .label("Selected Strike")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN.mix(0.5).stroke_width(2)));

    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    println!("Volatility smile saved as volatility_smile.png");
    Ok(())
}

pub fn plot_greeks(
    s0: f64,
    k: f64,
    t: f64,
    r: f64,
    sigma: f64,
    option_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*;
    let root = BitMapBackend::new("option_greeks.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let price_min = (s0.min(k)) * 0.8;
    let price_max = (s0.max(k)) * 1.2;
    let steps = 100;

    let mut delta_points = Vec::with_capacity(steps);
    let mut gamma_points = Vec::with_capacity(steps);
    let mut theta_points = Vec::with_capacity(steps);
    let mut vega_points = Vec::with_capacity(steps);

    for i in 0..steps {
        let price = price_min + (price_max - price_min) * i as f64 / (steps as f64 - 1.0);
        delta_points.push((price, calculate_delta(price, k, t, r, sigma, option_type)));
        gamma_points.push((price, calculate_gamma(price, k, t, r, sigma)));
        theta_points.push((price, calculate_theta(price, k, t, r, sigma, option_type)));
        vega_points.push((price, calculate_vega(price, k, t, r, sigma)));
    }

    let mut chart = ChartBuilder::on(&root)
        .caption("Option Greeks", ("sans-serif", 30))
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(price_min..price_max, -1.0..1.0)?;

    chart.configure_mesh()
        .x_desc("Stock Price ($)")
        .y_desc("Greek Value (normalized)")
        .draw()?;

    chart.draw_series(LineSeries::new(delta_points, &BLUE))?
        .label("Delta")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    chart.draw_series(LineSeries::new(gamma_points, &GREEN))?
        .label("Gamma")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));
    chart.draw_series(LineSeries::new(theta_points, &RED))?
        .label("Theta")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    chart.draw_series(LineSeries::new(vega_points, &MAGENTA))?
        .label("Vega")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &MAGENTA));

    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    println!("Option Greeks chart saved as option_greeks.png");
    Ok(())
}

pub fn plot_stock_paths(s0: f64,
    k: f64,
    t: f64,
    paths: Vec<Vec<f64>>
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("stock_price_paths.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let num_steps = paths[0].len() - 1;
    
    let mut min_price = s0;
    let mut max_price = s0;
    
    for path in &paths {
        for &price in path {
            min_price = min_price.min(price);
            max_price = max_price.max(price);
        }
    }

    let margin = (max_price - min_price) * 0.1;
    min_price -= margin;
    max_price += margin;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Monte Carlo Stock Price Paths (K={})", k), ("sans-serif", 30))
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..t, min_price..max_price)?;

    chart.configure_mesh()
        .x_desc("Time (years)")
        .y_desc("Stock Price")
        .draw()?;
        
    chart.draw_series(LineSeries::new(
        vec![(0.0, k), (t, k)],
        RED.mix(0.5).stroke_width(2),
    ))?
    .label("Strike Price")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.mix(0.5).stroke_width(2)));

    for (i, path) in paths.iter().enumerate() {
        let color = Palette99::pick(i).mix(0.5);
        let points: Vec<(f64, f64)> = path.iter().enumerate()
            .map(|(step, &price)| (step as f64 * t / num_steps as f64, price))
            .collect();
            
        chart.draw_series(LineSeries::new(points, color.stroke_width(1)))?;
    }
    
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
        
    println!("Stock price paths saved to stock_price_paths.png");
    
    Ok(())
}

pub fn plot_iv_curve_reciprocal(
    expiries: Vec<u64>,
    expiry_iv_pairs: Vec<(f64, f64)>, 
    predict_expiry: u64,
    predicted_iv: f64,
    x_mean: f64,
    x_std: f64,
    a: f64,
    b: f64,
    c: f64
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("iv_reciprocal.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_expiry = *expiries.iter().min().unwrap() as f64;
    let max_expiry = *expiries.iter().max().unwrap() as f64;

    let min_iv = expiry_iv_pairs.iter().map(|(_, iv)| *iv).fold(f64::INFINITY, f64::min);
    let max_iv = expiry_iv_pairs.iter().map(|(_, iv)| *iv).fold(f64::NEG_INFINITY, f64::max);

    let rec_eval = |x: f64| -> f64 {
        let x_norm = (x - x_mean) / x_std;
        let iv = a + b / (x_norm + c);
        iv.max(0.0)
    };

    let rec_min = rec_eval(min_expiry);
    let rec_max = rec_eval(max_expiry);
    let rec_mid = rec_eval((min_expiry + max_expiry) / 2.0);

    let min_iv = min_iv.min(rec_min).min(rec_mid).min(predicted_iv);
    let max_iv = max_iv.max(rec_max).max(rec_mid).max(predicted_iv);

    let padding = (max_iv - min_iv) * 0.1;
    let min_iv = min_iv - padding;
    let max_iv = max_iv + padding;

    let mut chart = ChartBuilder::on(&root)
        .caption("IV Reciprocal Model", ("sans-serif", 30))
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(min_expiry..max_expiry, min_iv..max_iv)?;

    chart.configure_mesh()
        .x_desc("Expiry (timestamp)")
        .y_desc("Implied Volatility")
        .draw()?;

    chart.draw_series(
        expiry_iv_pairs.iter().map(|(e, iv)| Circle::new((*e, *iv), 5, RED.filled()))
    )?;

    let steps = 200;
    let step = (max_expiry - min_expiry) / steps as f64;
    let curve_points: Vec<(f64, f64)> = (0..=steps)
        .map(|i| {
            let x = min_expiry + step * i as f64;
            (x, rec_eval(x))
        })
        .collect();

    chart.draw_series(LineSeries::new(curve_points, &BLUE))?;

    chart.draw_series(std::iter::once(Circle::new(
        (predict_expiry as f64, predicted_iv),
        8,
        GREEN.filled(),
    )))?;

    println!("Plot saved as iv_reciprocal.png");

    Ok(())
}

fn d1(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    ((s / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * t.sqrt())
}

fn d2(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    d1(s, k, t, r, sigma) - sigma * t.sqrt()
}

pub fn calculate_delta(s: f64, k: f64, t: f64, r: f64, sigma: f64, option_type: &str) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    match option_type {
        "call" => normal.cdf(d1(s, k, t, r, sigma)),
        _ => normal.cdf(d1(s, k, t, r, sigma)) - 1.0,
    }
}

pub fn calculate_gamma(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    normal.pdf(d1(s, k, t, r, sigma)) / (s * sigma * t.sqrt())
}

pub fn calculate_theta(s: f64, k: f64, t: f64, r: f64, sigma: f64, option_type: &str) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let d1 = d1(s, k, t, r, sigma);
    let d2 = d2(s, k, t, r, sigma);
    let first = - (s * normal.pdf(d1) * sigma) / (2.0 * t.sqrt());
    let second = match option_type {
        "call" => -r * k * (-r * t).exp() * normal.cdf(d2),
        _ => r * k * (-r * t).exp() * normal.cdf(-d2),
    };
    (first + second) / 365.0
}

pub fn calculate_vega(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    s * normal.pdf(d1(s, k, t, r, sigma)) * t.sqrt() / 100.0
}