
/// 等待s秒
pub fn sleep(s: &str) -> Result<(), String> {
	let ms: u64 = (s.parse::<f64>().unwrap() * 1000.0) as u64;
    std::thread::sleep(std::time::Duration::from_millis(ms));
    Ok(())
}