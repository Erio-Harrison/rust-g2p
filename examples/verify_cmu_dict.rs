use rust_g2p::dict::Dictionary;

fn main() -> anyhow::Result<()> {
    println!("=== CMU Dictionary Verification ===");
    
    // 检查文件是否存在
    let dict_path = "data/cmudict.txt";
    if !std::path::Path::new(dict_path).exists() {
        eprintln!("Error: CMU dictionary file not found at {}", dict_path);
        eprintln!("Please download it using:");
        eprintln!("  mkdir -p data");
        eprintln!("  curl -o data/cmudict.txt \"https://raw.githubusercontent.com/cmusphinx/cmudict/master/cmudict.dict\"");
        return Ok(());
    }
    
    // 显示文件信息
    let metadata = std::fs::metadata(dict_path)?;
    println!("File size: {:.2} MB", metadata.len() as f64 / 1024.0 / 1024.0);
    
    // 加载词典
    let dict = Dictionary::load_cmu_dict(dict_path)?;
    
    println!("\nDictionary loaded successfully!");
    println!("Total entries: {}", dict.size());
    
    // 测试常见单词
    let test_words = vec![
        "hello", "world", "computer", "language", "artificial", 
        "intelligence", "the", "and", "or", "to", "from", "cat", "dog"
    ];
    
    println!("\n=== Testing common words ===");
    let mut found_count = 0;
    for word in &test_words {
        match dict.lookup(word) {
            Some(phonemes) => {
                found_count += 1;
                print!("{}: ", word);
                for phoneme in phonemes {
                    print!("{} ", phoneme);
                }
                println!();
            }
            None => {
                println!("{}: NOT FOUND", word);
            }
        }
    }
    
    println!("\nFound {}/{} test words", found_count, test_words.len());
    
    // 显示词典样本
    println!("\n=== Dictionary sample (first 10 words) ===");
    let sample_words = dict.get_sample_words(10);
    for (i, word) in sample_words.iter().enumerate() {
        if let Some(phonemes) = dict.lookup(word) {
            print!("{}. {}: ", i+1, word);
            for phoneme in phonemes {
                print!("{} ", phoneme);
            }
            println!();
        }
    }
    
    // 计算一些统计信息
    println!("\n=== Statistics ===");
    println!("Average word length: {:.1} characters", 
             sample_words.iter().map(|w| w.len()).sum::<usize>() as f64 / sample_words.len() as f64);
    
    Ok(())
}