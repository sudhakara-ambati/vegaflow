use serde_json;
use plotters::prelude::*;
use scraper::{Selector};
use nalgebra::{DMatrix, DVector};

pub async fn fetch_risk_free_rate(api_key: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.stlouisfed.org/fred/series/observations?series_id=GS1&api_key={}&file_type=json&sort_order=desc&limit=1",
        api_key
    );

    let resp = reqwest::get(&url).await?.json::<serde_json::Value>().await?;

    if let Some(observations) = resp["observations"].as_array() {
        if let Some(latest) = observations.first() {
            if let Some(value_str) = latest["value"].as_str() {
                if let Ok(value) = value_str.parse::<f64>() {
                    return Ok(value / 100.0);
                }
            }
        }
    }

    Err("Failed to parse risk-free rate".into())
}

pub async fn predict_iv(
    symbol: &str,
    input_strike: f64,
    predict_expiry: u64,
    option_type: &str
) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!("https://finance.yahoo.com/quote/{}/options", symbol);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;
    let body = resp.text().await?;

    let document = scraper::Html::parse_document(&body);
    let script_selector = scraper::Selector::parse(r#"script[type="application/json"]"#).unwrap();

    let mut expiries = Vec::new();

    for script in document.select(&script_selector) {
        if let Some(json_text) = script.text().next() {
            if let Ok(outer_json) = serde_json::from_str::<serde_json::Value>(json_text) {
                if let Some(body_str) = outer_json.get("body").and_then(|b| b.as_str()) {
                    if let Ok(inner_json) = serde_json::from_str::<serde_json::Value>(body_str) {
                        if let Some(expiry_arr) = inner_json["optionChain"]["result"][0]["expirationDates"].as_array() {
                            for expiry in expiry_arr {
                                if let Some(ts) = expiry.as_u64() {
                                    expiries.push(ts);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if expiries.is_empty() {
        return Err("Could not find expiration dates".into());
    }

    let mut expiry_iv_pairs = Vec::new();

    for expiry in &expiries {
        match fetch_closest_iv_for_expiry(symbol, *expiry, input_strike, option_type).await {
            Ok(iv) => {
                println!("Expiry {}: Closest IV = {:.2}%", expiry, iv * 100.0);
                expiry_iv_pairs.push((*expiry as f64, iv));
            }
            Err(e) => println!("Expiry {}: Error fetching IV: {}", expiry, e),
        }
    }

    if expiry_iv_pairs.len() < 3 {
        println!("Not enough data for reciprocal regression (need at least 3 points).");
        return Err("Not enough data for reciprocal regression".into());
    }

    expiry_iv_pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let n = expiry_iv_pairs.len();
    let xs: Vec<f64> = expiry_iv_pairs.iter().map(|(e, _)| *e).collect();
    let ys: Vec<f64> = expiry_iv_pairs.iter().map(|(_, iv)| *iv).collect();

    let x_mean: f64 = xs.iter().sum::<f64>() / n as f64;
    let x_std: f64 = (xs.iter().map(|x| (x - x_mean).powi(2)).sum::<f64>() / n as f64).sqrt();
    let xs_norm: Vec<f64> = xs.iter().map(|x| (x - x_mean) / x_std).collect();

    let mut best_c = 1.0;
    let mut best_err = f64::INFINITY;
    let mut best_a = 0.0;
    let mut best_b = 0.0;

    for c_try in (1..1000).map(|i| i as f64 * 0.01) {
        let rec_terms: Vec<f64> = xs_norm.iter().map(|&x| 1.0 / (x + c_try)).collect();
        let mut a_mat = DMatrix::zeros(n, 2);
        for i in 0..n {
            a_mat[(i, 0)] = 1.0;
            a_mat[(i, 1)] = rec_terms[i];
        }
        let b_vec = DVector::from_iterator(n, ys.iter().cloned());
        let lhs = a_mat.transpose() * &a_mat;
        let rhs = a_mat.transpose() * &b_vec;
        if let Some(coeffs) = lhs.lu().solve(&rhs) {
            let a = coeffs[0];
            let b = coeffs[1];
            let err = ys.iter().enumerate().map(|(i, &y)| {
                let y_pred = a + b * rec_terms[i];
                (y - y_pred).powi(2)
            }).sum::<f64>();
            if err < best_err {
                best_err = err;
                best_c = c_try;
                best_a = a;
                best_b = b;
            }
        }
    }

    let rec_eval = |x: f64| {
        let x_norm = (x - x_mean) / x_std;
        let iv = best_a + best_b / (x_norm + best_c);
        iv.max(0.0)
    };

    let predicted_iv = rec_eval(predict_expiry as f64);

    println!(
        "Predicted IV at expiry {} (reciprocal): {:.2}%",
        predict_expiry,
        predicted_iv * 100.0
    );

    plot_iv_curve_reciprocal(
        expiries,
        expiry_iv_pairs,
        predict_expiry,
        predicted_iv,
        x_mean,
        x_std,
        best_a,
        best_b,
        best_c,
    )?;

    Ok(predicted_iv)
}

fn plot_iv_curve_reciprocal(
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

pub async fn fetch_closest_iv_for_expiry(
    symbol: &str,
    expiry: u64,
    input_strike: f64,
    option_type: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!("https://finance.yahoo.com/quote/{}/options?date={}", symbol, expiry);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;
    let body = resp.text().await?;

    let document = scraper::Html::parse_document(&body);

    let table_selector = Selector::parse("table.yf-wurt5d").unwrap();
    let tables: Vec<_> = document.select(&table_selector).collect();

    let table_index = match option_type {
        "put" => 1,
        _ => 0,
    };

    let table = tables.get(table_index)
        .ok_or_else(|| format!("Could not find {} table", option_type))?;

    let row_selector = Selector::parse("tr.yf-wurt5d").unwrap();
    let cell_selector = Selector::parse("td.yf-wurt5d").unwrap();
    let bold_selector = Selector::parse("td.bold.yf-wurt5d").unwrap();

    let mut best_iv: Option<f64> = None;
    let mut min_diff = f64::MAX;

    for row in table.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        let bold_cells: Vec<_> = row.select(&bold_selector).collect();

        if bold_cells.len() >= 2 {
            let strike_text = bold_cells[1].text().collect::<String>().replace(',', "");
            if let Ok(strike) = strike_text.trim().parse::<f64>() {
                if let Some(iv_cell) = cells.last() {
                    let iv_text = iv_cell.text().collect::<String>().replace('%', "").replace(',', "");
                    if let Ok(iv) = iv_text.trim().parse::<f64>() {
                        if iv > 0.0 {
                            let diff = (strike - input_strike).abs();
                            if diff < min_diff {
                                min_diff = diff;
                                best_iv = Some(iv / 100.0);
                            }
                        }
                    }
                }
            }
        }
    }

    best_iv.ok_or_else(|| format!("No in-the-money {}s with nonzero IV found in HTML", option_type).into())
}

pub async fn fetch_stock_price(symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!("https://finance.yahoo.com/quote/{}", symbol);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;
    let body = resp.text().await?;

    let document = scraper::Html::parse_document(&body);
    let price_selector = scraper::Selector::parse(r#"span.base.yf-ipw1h0"#).unwrap();

    if let Some(price_elem) = document.select(&price_selector).next() {
        let price_text = price_elem.text().collect::<String>().replace(",", "");
        if let Ok(price) = price_text.trim().parse::<f64>() {
            return Ok(price);
        }
    }

    Err("Could not find or parse stock price".into())
}