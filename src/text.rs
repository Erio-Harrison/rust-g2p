use anyhow::Result;
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;

/// 文本预处理器
pub struct TextProcessor {
    number_words: HashMap<&'static str, &'static str>,
    abbreviations: HashMap<&'static str, &'static str>,
}

impl TextProcessor {
    pub fn new() -> Self {
        let mut number_words = HashMap::new();
        number_words.insert("0", "zero");
        number_words.insert("1", "one");
        number_words.insert("2", "two");
        number_words.insert("3", "three");
        number_words.insert("4", "four");
        number_words.insert("5", "five");
        number_words.insert("6", "six");
        number_words.insert("7", "seven");
        number_words.insert("8", "eight");
        number_words.insert("9", "nine");
        number_words.insert("10", "ten");
        number_words.insert("11", "eleven");
        number_words.insert("12", "twelve");
        number_words.insert("13", "thirteen");
        number_words.insert("14", "fourteen");
        number_words.insert("15", "fifteen");
        number_words.insert("16", "sixteen");
        number_words.insert("17", "seventeen");
        number_words.insert("18", "eighteen");
        number_words.insert("19", "nineteen");
        number_words.insert("20", "twenty");
        
        let mut abbreviations = HashMap::new();
        abbreviations.insert("dr.", "doctor");
        abbreviations.insert("mr.", "mister");
        abbreviations.insert("mrs.", "misses");
        abbreviations.insert("ms.", "miss");
        abbreviations.insert("prof.", "professor");
        abbreviations.insert("st.", "street");
        abbreviations.insert("ave.", "avenue");
        abbreviations.insert("blvd.", "boulevard");
        abbreviations.insert("etc.", "etcetera");
        abbreviations.insert("vs.", "versus");
        
        Self {
            number_words,
            abbreviations,
        }
    }
    
    /// 文本标准化
    pub fn normalize(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();
        
        // 1. 转小写
        result = result.to_lowercase();
        
        // 2. 处理缩写
        result = self.expand_abbreviations(&result);
        
        // 3. 处理数字
        result = self.expand_numbers(&result);
        
        // 4. 清理标点符号
        result = self.clean_punctuation(&result);
        
        // 5. 标准化空格
        result = self.normalize_whitespace(&result);
        
        Ok(result)
    }
    
    /// 分词
    pub fn tokenize(&self, text: &str) -> Result<Vec<String>> {
        let words: Vec<String> = text
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .map(|word| word.trim_matches(|c: char| !c.is_alphabetic()))
            .filter(|word| !word.is_empty())
            .map(|word| word.to_string())
            .collect();
        
        Ok(words)
    }
    
    /// 展开缩写
    fn expand_abbreviations(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        for (abbrev, expansion) in &self.abbreviations {
            result = result.replace(abbrev, expansion);
        }
        
        result
    }
    
    /// 展开数字
    fn expand_numbers(&self, text: &str) -> String {
        lazy_static! {
            static ref NUMBER_RE: Regex = Regex::new(r"\b\d+\b").unwrap();
        }
        
        NUMBER_RE.replace_all(text, |caps: &regex::Captures| {
            let number = &caps[0];
            self.number_words.get(number).unwrap_or(&number).to_string()
        }).to_string()
    }
    
    /// 清理标点符号
    fn clean_punctuation(&self, text: &str) -> String {
        lazy_static! {
            static ref PUNCT_RE: Regex = Regex::new(r"[^\w\s]").unwrap();
        }
        
        PUNCT_RE.replace_all(text, " ").to_string()
    }
    
    /// 标准化空格
    fn normalize_whitespace(&self, text: &str) -> String {
        lazy_static! {
            static ref WHITESPACE_RE: Regex = Regex::new(r"\s+").unwrap();
        }
        
        WHITESPACE_RE.replace_all(text.trim(), " ").to_string()
    }
}
