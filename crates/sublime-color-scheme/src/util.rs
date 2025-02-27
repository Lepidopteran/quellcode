fn parse_percentage_or_float(input: &str) -> Option<f64> {
    input.trim().strip_suffix('%')
        .map_or_else(|| input.parse().ok(), |s| s.parse::<f64>().ok().map(|n| n / 100.0))
}
