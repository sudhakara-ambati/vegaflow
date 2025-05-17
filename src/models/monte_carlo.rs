use statrs::distribution::{Normal, ContinuousCDF};
use rand::prelude::*;
use plotters::prelude::*;

pub fn monte_carlo_option_price(s0: f64, k: f64, t: f64, r: f64, sigma: f64, option_type: &str, n: usize,) -> f64 {
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

fn plot_stock_paths(s0: f64, k: f64, t: f64, paths: Vec<Vec<f64>>) -> Result<(), Box<dyn std::error::Error>> {
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