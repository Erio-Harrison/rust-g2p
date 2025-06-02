pub mod phoneme;
pub mod rules;
pub mod dict;
pub mod text;
pub mod lang;

pub use phoneme::Phoneme;
pub use rules::RulesEngine;
pub use dict::Dictionary;

use anyhow::Result;

/// 主要的G2P转换器
pub struct RustG2P {
    dictionary: Dictionary,
    rules_engine: RulesEngine,
    text_processor: text::TextProcessor,
}

impl RustG2P {
    /// 创建新的G2P转换器
    pub fn new() -> Result<Self> {
        let dictionary = Dictionary::load_cmu_dict("data/cmudict.txt")?;
        let rules_engine = RulesEngine::load_english_rules("data/en_rules.txt")?;
        let text_processor = text::TextProcessor::new();
        
        Ok(Self {
            dictionary,
            rules_engine,
            text_processor,
        })
    }
    
    /// 将文本转换为音素
    pub fn text_to_phonemes(&self, text: &str) -> Result<Vec<Phoneme>> {
        // 1. 文本预处理
        let normalized = self.text_processor.normalize(text)?;
        
        // 2. 分词
        let words = self.text_processor.tokenize(&normalized)?;
        
        // 3. 逐词转换
        let mut phonemes = Vec::new();
        for word in words {
            let word_phonemes = self.word_to_phonemes(&word)?;
            phonemes.extend(word_phonemes);
            
            // 添加词间停顿（可选）
            phonemes.push(Phoneme::word_boundary());
        }
        
        Ok(phonemes)
    }
    
    /// 单词转音素（核心功能）
    pub fn word_to_phonemes(&self, word: &str) -> Result<Vec<Phoneme>> {
        let word = word.to_lowercase();
        
        // 1. 先查词典
        if let Some(phonemes) = self.dictionary.lookup(&word) {
            return Ok(phonemes);
        }
        
        // 2. 使用规则引擎
        self.rules_engine.apply_rules(&word)
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> G2PStats {
        G2PStats {
            dict_entries: self.dictionary.size(),
            rule_count: self.rules_engine.rule_count(),
        }
    }
}

#[derive(Debug)]
pub struct G2PStats {
    pub dict_entries: usize,
    pub rule_count: usize,
}