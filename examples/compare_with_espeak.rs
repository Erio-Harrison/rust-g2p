use rust_g2p::RustG2P;

// 注意：这个示例需要安装espeak-rs依赖
// [dependencies]
// espeak-rs = "0.1"

fn main() -> anyhow::Result<()> {
    let g2p = RustG2P::new()?;
    
    let test_words = vec![
        "hello", "world", "computer", "language", "artificial", "intelligence",
        "pneumonia", "psychology", "rhythm", "colonel", "knight", "yacht",
    ];
    
    println!("=== Comparing with espeak-ng ===");
    println!("{:<15} {:<30} {:<30}", "Word", "Rust G2P", "espeak-ng");
    println!("{}", "-".repeat(75));
    
    for word in test_words {
        // Rust G2P结果
        let rust_result = match g2p.word_to_phonemes(word) {
            Ok(phonemes) => {
                phonemes.iter()
                    .map(|p| p.symbol.clone())
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            Err(_) => "ERROR".to_string(),
        };
        
        // espeak-ng结果
        let espeak_result = match espeak_rs::text_to_phonemes(word, "en", None, true, false) {
            Ok(phonemes) => phonemes.join(" "),
            Err(_) => "ERROR".to_string(),
        };
        
        println!("{:<15} {:<30} {:<30}", word, rust_result, espeak_result);
    }
    
    Ok(())
}