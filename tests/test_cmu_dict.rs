#[cfg(test)]
mod tests {
    use rust_g2p::dict::Dictionary;

    #[test]
    fn test_cmu_dict_loading() {
        let dict = Dictionary::load_cmu_dict("data/cmudict.txt")
            .expect("Failed to load CMU dictionary");
        
        assert!(!dict.is_empty(), "Dictionary should not be empty");
        assert!(dict.size() > 10000, "Dictionary should have more than 10k entries");
        
        println!("Dictionary size: {}", dict.size());
    }

    #[test]
    fn test_common_words() {
        let dict = Dictionary::load_cmu_dict("data/cmudict.txt")
            .expect("Failed to load CMU dictionary");
        
        // 测试一些常见单词
        let test_words = vec!["hello", "world", "the", "and", "computer"];
        
        for word in test_words {
            let phonemes = dict.lookup(word);
            assert!(phonemes.is_some(), "Word '{}' should be in dictionary", word);
            
            if let Some(phonemes) = phonemes {
                assert!(!phonemes.is_empty(), "Phonemes for '{}' should not be empty", word);
                println!("{}: {:?}", word, phonemes);
            }
        }
    }

    #[test]
    fn test_phoneme_parsing() {
        let dict = Dictionary::load_cmu_dict("data/cmudict.txt")
            .expect("Failed to load CMU dictionary");
        
        if let Some(phonemes) = dict.lookup("hello") {
            // HELLO应该是 HH AH0 L OW1 或类似
            assert!(phonemes.len() >= 3, "Hello should have at least 3 phonemes");
            
            // 检查第一个音素是否是HH
            assert_eq!(phonemes[0].symbol, "HH", "First phoneme should be HH");
        }
    }
}