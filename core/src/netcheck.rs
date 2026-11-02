use serde::Serialize;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize)]
pub struct LagResult {
    pub target: String,
    pub ping_ms: f64,
    pub jitter_ms: f64,
    pub loss_pct: f64,
    pub rating: String,
    pub message: String,
    pub series: Vec<Option<f64>>,
}

/// (average ping, jitter = average change between neighbouring samples, loss percent)
pub fn summarize(samples: &[Option<f64>]) -> (f64, f64, f64) {
    let ok: Vec<f64> = samples.iter().flatten().cloned().collect();
    if samples.is_empty() || ok.is_empty() {
        return (0.0, 0.0, 100.0);
    }
    let avg = ok.iter().sum::<f64>() / ok.len() as f64;
    let jit = if ok.len() < 2 { 0.0 } else { ok.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (ok.len() - 1) as f64 };
    let loss = 100.0 * (samples.len() - ok.len()) as f64 / samples.len() as f64;
    (avg, jit, loss)
}

pub fn rate(ping: f64, jitter: f64, loss: f64) -> &'static str {
    if ping < 60.0 && jitter < 15.0 && loss < 1.0 {
        "Good"
    } else if ping < 120.0 && jitter < 30.0 && loss < 3.0 {
        "Okay"
    } else {
        "Poor"
    }
}

fn build(target: &str, series: Vec<Option<f64>>) -> LagResult {
    let (ping, jitter, loss) = summarize(&series);
    let rating = rate(ping, jitter, loss);
    let message = match rating {
        "Good" => "Your connection looks stable for gaming.",
        "Okay" => "Playable, but you may notice some lag in fast games.",
        _ => "Your connection is unstable. Try a wired connection and close apps that use the internet.",
    };
    LagResult {
        target: target.to_string(),
        ping_ms: (ping * 10.0).round() / 10.0,
        jitter_ms: (jitter * 10.0).round() / 10.0,
        loss_pct: (loss * 10.0).round() / 10.0,
        rating: rating.to_string(),
        message: format!("{} Measured to a public server, so your game server may differ.", message),
        series,
    }
}

/// 20 quick connection tests to a public server (about 2 to 4 seconds).
pub fn lag_check() -> Result<LagResult, String> {
    for target in ["1.1.1.1:443", "8.8.8.8:443"] {
        let addr: SocketAddr = target.parse().map_err(|_| "bad address".to_string())?;
        let series: Vec<Option<f64>> = (0..20)
            .map(|_| {
                let t = Instant::now();
                let r = TcpStream::connect_timeout(&addr, Duration::from_millis(1000)).ok().map(|_| t.elapsed().as_secs_f64() * 1000.0);
                std::thread::sleep(Duration::from_millis(80));
                r
            })
            .collect();
        if series.iter().any(|x| x.is_some()) {
            return Ok(build(target, series));
        }
    }
    Err("Could not reach the internet. Check your connection.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stats() {
        let (p, j, l) = summarize(&[Some(10.0), Some(20.0), None, Some(10.0)]);
        assert!((p - 13.333).abs() < 0.01);
        assert!((j - 10.0).abs() < 0.01);
        assert!((l - 25.0).abs() < 0.01);
        assert_eq!(summarize(&[None, None]).2, 100.0);
    }
    #[test]
    fn ratings() {
        assert_eq!(rate(30.0, 5.0, 0.0), "Good");
        assert_eq!(rate(90.0, 20.0, 2.0), "Okay");
        assert_eq!(rate(200.0, 5.0, 0.0), "Poor");
    }
}
