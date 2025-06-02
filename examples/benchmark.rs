use rust_g2p::RustG2P;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let g2p = RustG2P::new()?;
    
    let test_text = "The quick brown fox jumps over the lazy dog. \
                     Dr. Smith and Mrs. Johnson went to the store to buy 10 apples and 5 oranges. \
                     This is a longer sentence to test the performance of our G2P system.";
    
    println!("Benchmarking G2P performance...");
    println!("Test text: {}", test_text);
    println!();
    
    // 预热
    for _ in 0..10 {
        let _ = g2p.text_to_phonemes(test_text)?;
    }
    
    // 基准测试
    let iterations = 1000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = g2p.text_to_phonemes(test_text)?;
    }
    
    let elapsed = start.elapsed();
    let avg_time = elapsed / iterations;
    let avg_time_us = avg_time.as_micros() as f64;
    
    println!("Performance results:");
    println!("Iterations: {}", iterations);
    println!("Total time: {:?}", elapsed);
    println!("Average time per conversion: {:?}", avg_time);
    println!("Conversions per second: {:.2}", 1_000_000.0 / avg_time_us);
    
    Ok(())
}