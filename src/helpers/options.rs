pub fn black_scholes_delta(spot: f64, strike: f64, r: f64, sigma: f64, t_years: f64, is_call: bool) -> Option<f64> {
    if spot <= 0.0 || strike <= 0.0 || sigma <= 0.0 || t_years <= 0.0 {
        return None;
    }
    let d1 = ((spot / strike).ln() + (r + 0.5 * sigma * sigma) * t_years) / (sigma * t_years.sqrt());
    let norm = statrs::distribution::Normal::new(0.0, 1.0).ok()?;
    let nd1 = statrs::distribution::ContinuousCDF::cdf(&norm, d1);
    if is_call { Some(nd1) } else { Some(nd1 - 1.0) }
}


