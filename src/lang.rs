/// 语言特定处理的trait
pub trait Language {
    fn normalize_text(&self, text: &str) -> anyhow::Result<String>;
    fn tokenize(&self, text: &str) -> anyhow::Result<Vec<String>>;
    fn get_stress_pattern(&self, word: &str) -> anyhow::Result<Vec<usize>>;
}

/// 英语语言处理
pub struct English;

impl Language for English {
    fn normalize_text(&self, text: &str) -> anyhow::Result<String> {
        // 使用TextProcessor进行标准化
        let processor = crate::text::TextProcessor::new();
        processor.normalize(text)
    }
    
    fn tokenize(&self, text: &str) -> anyhow::Result<Vec<String>> {
        let processor = crate::text::TextProcessor::new();
        processor.tokenize(text)
    }
    
    fn get_stress_pattern(&self, _word: &str) -> anyhow::Result<Vec<usize>> {
        // 简化的重音规则：单音节词重音在第一个音节
        Ok(vec![0])
    }
}