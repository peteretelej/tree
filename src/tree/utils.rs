pub fn bytes_to_human_readable(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let base: f64 = 1024.0;
    let unit = UNITS.iter().enumerate().find_map(|(i, unit)| {
        let size = bytes as f64 / base.powi(i as i32);
        if size < base {
            Some(format!("{:.1} {}", size, unit))
        } else {
            None
        }
    }).unwrap_or_else(|| format!("{:} B", bytes));

    unit
}