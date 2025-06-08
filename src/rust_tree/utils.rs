pub fn bytes_to_human_readable(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string(); // Handle zero case explicitly
    }
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let base: f64 = 1024.0;
    let i = (bytes as f64).log(base).floor() as usize;

    // Ensure index is within bounds, default to B if calculation yields strange result
    let i = if i < UNITS.len() { i } else { 0 };

    let size = bytes as f64 / base.powi(i as i32);

    // Adjust format based on whether it's bytes or a larger unit
    if i == 0 {
        // Bytes
        format!("{} {}", size as u64, UNITS[i])
    } else {
        // KB, MB, etc.
        format!("{:.1} {}", size, UNITS[i])
    }
}
