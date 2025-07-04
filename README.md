# VegaFlow

**VegaFlow** is a Rust-based toolkit for pricing European options using both analytical and simulation-based models. It implements the Black-Scholes formula and a Monte Carlo simulation engine based on Geometric Brownian Motion (GBM).

It also integrates real-time financial data from Yahoo Finance and the FRED API, allowing for dynamic pricing based on current market conditions.

## Features

- Black-Scholes model for calculating theoretical option prices.
- Monte Carlo simulation using GBM for estimating prices based on asset path behavior.
- Real-time stock data retrieval via the Yahoo Finance API.
- Live risk-free rate data from the FRED API.
- Regression to predict IV using historical data

## Visualisations

VegaFlow provides six powerful visualisations to help analyse option characteristics:

1. **Volatility Smile**: Plots implied volatility against strike price, showing market pricing across different strike levels and revealing supply/demand dynamics.

2. **Option Greeks**: Visualises Delta, Gamma, Theta, and Vega as functions of the underlying price, helping traders understand option sensitivity to various market factors.

3. **Time Decay Curve**: Shows how option prices decay as expiration approaches, with separate lines for intrinsic and time value components.

4. **Stock Price Paths (Monte Carlo)**: Displays multiple simulated future price paths based on GBM, illustrating the range of possible outcomes.

5. **PnL Distribution Histogram**: Shows the probability distribution of profit and loss outcomes at expiration, with statistics on probability of profit.

6. **IV Reciprocal Curve**: Plots implied volatility as a function of expiry using a reciprocal model fit, useful for volatility forecasting.

## Example Usage (coming soon)

## Goals
- Provide multiple methods for calculating and comparing option prices.
- Explore how market volatility and randomness affect pricing accuracy.
- Offer a flexible base for extending to other types of options or pricing models.

## In Progress
- Web application with visualisations of regression for IV