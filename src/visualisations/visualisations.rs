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