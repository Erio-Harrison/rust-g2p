use rust_g2p::RustG2P;

#[test]
fn test_basic_word_conversion() {
    let g2p = RustG2P::new().expect("Failed to create G2P");
    
    // 测试基础单词
    let phonemes = g2p.word_to_phonemes("hello").unwrap();
    assert!(!phonemes.is_empty());
    
    let phonemes = g2p.word_to_phonemes("world").unwrap();
    assert!(!phonemes.is_empty());
}

#[test]
fn test_text_processing() {
    let g2p = RustG2P::new().expect("Failed to create G2P");
    
    // 测试句子处理
    let phonemes = g2p.text_to_phonemes("Hello, world!").unwrap();
    assert!(!phonemes.is_empty());
    
    // 测试数字展开
    let phonemes = g2p.text_to_phonemes("I have 5 cats").unwrap();
    assert!(!phonemes.is_empty());
}

#[test]
fn test_abbreviations() {
    let g2p = RustG2P::new().expect("Failed to create G2P");
    
    let phonemes = g2p.text_to_phonemes("Dr. Smith").unwrap();
    assert!(!phonemes.is_empty());
}

#[test]
fn test_rules_engine() {
    let g2p = RustG2P::new().expect("Failed to create G2P");
    
    // 测试不在词典中的词
    let phonemes = g2p.word_to_phonemes("pseudoword").unwrap();
    assert!(!phonemes.is_empty());
}
