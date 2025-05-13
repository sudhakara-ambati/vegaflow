use serde_json;
use scraper::{Html, Selector};


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

pub async fn print_all_expiry_ivs(symbol: &str, input_strike: f64) -> Result<(), Box<dyn std::error::Error>> {
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

    for expiry in expiries {
        match fetch_closest_iv_for_expiry(symbol, expiry, input_strike).await {
            Ok(iv) => println!("Expiry {}: Closest IV = {:.2}%", expiry, iv * 100.0),
            Err(e) => println!("Expiry {}: Error fetching IV: {}", expiry, e),
        }
    }

    Ok(())
}


pub async fn fetch_closest_iv_for_expiry(symbol: &str, expiry: u64, input_strike: f64) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!("https://finance.yahoo.com/quote/{}/options?date={}", symbol, expiry);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;
    let body = resp.text().await?;

    let document = Html::parse_document(&body);
    let row_selector = Selector::parse("tr.yf-wurt5d.inTheMoney").unwrap();
    let cell_selector = Selector::parse("td.yf-wurt5d").unwrap();
    let bold_selector = Selector::parse("td.bold.yf-wurt5d").unwrap();

    let mut closest_iv: Option<f64> = None;
    let mut min_diff = f64::MAX;

    for row in document.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();
        let bold_cells: Vec<_> = row.select(&bold_selector).collect();

        if bold_cells.len() >= 2 {
            let strike_text = bold_cells[1].text().collect::<String>().replace(',', "");
            if let Ok(strike) = strike_text.trim().parse::<f64>() {
                if let Some(iv_cell) = cells.last() {
                    let iv_text = iv_cell.text().collect::<String>().replace('%', "").replace(',', "");
                    if let Ok(iv) = iv_text.trim().parse::<f64>() {
                        let diff = (strike - input_strike).abs();
                        if diff < min_diff {
                            min_diff = diff;
                            closest_iv = Some(iv / 100.0);
                        }
                    }
                }
            }
        }
    }

    closest_iv.ok_or_else(|| "No in-the-money calls with IV found in HTML".into())
}
