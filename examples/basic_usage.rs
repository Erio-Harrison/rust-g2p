use rust_g2p::RustG2P;

fn main() -> anyhow::Result<()> {
    println!("=== Rust G2P Demo ===");
    
    // 初始化G2P转换器
    let g2p = RustG2P::new()?;
    
    // 显示统计信息
    let stats = g2p.get_stats();
    println!("Dictionary entries: {}", stats.dict_entries);
    println!("Rules: {}", stats.rule_count);
    println!();
    
    // 测试单词转换
    let test_words = vec![
        "hello",
        "world", 
        "computer",
        "language",
        "artificial",
        "intelligence",
    ];
    
    println!("=== Word-level conversion ===");
    for word in test_words {
        match g2p.word_to_phonemes(word) {
            Ok(phonemes) => {
                print!("{}: ", word);
                for phoneme in phonemes {
                    print!("{} ", phoneme);
                }
                println!();
            }
            Err(e) => println!("{}: Error - {}", word, e),
        }
    }
    
    println!();
    
    // 测试句子转换
    let test_sentences = vec![
        "Hello, world!",
        "Dr. Smith has 5 cats.",
        "This is a test sentence.",
        "How are you today?",
    ];
    
    println!("=== Sentence-level conversion ===");
    for sentence in test_sentences {
        println!("Input: {}", sentence);
        match g2p.text_to_phonemes(sentence) {
            Ok(phonemes) => {
                print!("Output: ");
                for phoneme in phonemes {
                    if phoneme.symbol == " " {
                        print!("| ");
                    } else {
                        print!("{} ", phoneme);
                    }
                }
                println!();
            }
            Err(e) => println!("Error: {}", e),
        }
        println!();
    }
    
    Ok(())
}
