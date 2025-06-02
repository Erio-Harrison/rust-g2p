use crate::phoneme::Phoneme;
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::fs;

/// CMU发音词典
pub struct Dictionary {
    entries: HashMap<String, Vec<Phoneme>>,
}

impl Dictionary {
    /// 加载CMU词典 - 正确处理编码问题
    pub fn load_cmu_dict(path: &str) -> Result<Self> {
        println!("Loading CMU dictionary from: {}", path);
        
        // 确保文件存在
        if !std::path::Path::new(path).exists() {
            return Err(anyhow::anyhow!("CMU dictionary file not found: {}", path));
        }
        
        // 读取原始字节并处理编码问题
        let bytes = fs::read(path)
            .with_context(|| format!("Failed to read CMU dictionary file: {}", path))?;
        
        // 将字节转换为字符串，替换无效的UTF-8字符
        let content = String::from_utf8_lossy(&bytes);
        
        let mut entries = HashMap::new();
        let mut line_count = 0;
        let mut valid_entries = 0;
        let mut skipped_lines = 0;
        
        for line in content.lines() {
            line_count += 1;
            
            // 跳过注释行和空行
            if line.starts_with(";;;") || line.trim().is_empty() {
                continue;
            }
            
            // 检查行是否包含有效字符
            if !Self::is_valid_line(line) {
                skipped_lines += 1;
                if skipped_lines <= 10 {  // 只显示前10个跳过的行
                    eprintln!("Warning: Skipping invalid line {}: '{}'", line_count, 
                             Self::truncate_string(line, 50));
                }
                continue;
            }
            
            // 解析词典条目
            if let Some((word, phonemes_str)) = Self::parse_cmu_line(line) {
                match Self::parse_phonemes(&phonemes_str) {
                    Ok(phonemes) => {
                        let clean_word = Self::clean_word(&word);
                        entries.insert(clean_word, phonemes);
                        valid_entries += 1;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse phonemes for '{}' on line {}: {}", 
                                word, line_count, e);
                    }
                }
            } else {
                skipped_lines += 1;
                if skipped_lines <= 10 {
                    eprintln!("Warning: Skipping malformed line {}: '{}'", line_count, 
                             Self::truncate_string(line, 50));
                }
            }
            
            // 每10000行显示一次进度
            if line_count % 10000 == 0 {
                println!("Processed {} lines, {} valid entries, {} skipped", 
                        line_count, valid_entries, skipped_lines);
            }
        }
        
        println!("Successfully loaded CMU dictionary:");
        println!("  Total lines processed: {}", line_count);
        println!("  Valid entries: {}", valid_entries);
        println!("  Skipped lines: {}", skipped_lines);
        
        if valid_entries == 0 {
            return Err(anyhow::anyhow!("No valid entries found in CMU dictionary"));
        }
        
        Ok(Self { entries })
    }
    
    /// 检查行是否包含有效字符
    fn is_valid_line(line: &str) -> bool {
        // 检查行是否太短或太长
        if line.trim().len() < 3 || line.len() > 200 {
            return false;
        }
        
        // 检查是否包含基本的可打印ASCII字符
        for ch in line.chars() {
            if !ch.is_ascii() && !ch.is_whitespace() {
                return false;
            }
        }
        
        // 检查是否包含至少一个字母（单词部分）
        let has_letter = line.chars().any(|c| c.is_ascii_alphabetic());
        if !has_letter {
            return false;
        }
        
        true
    }
    
    /// 截断字符串用于显示
    fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
    
    /// 解析CMU词典行格式 - 更健壮的版本
    fn parse_cmu_line(line: &str) -> Option<(String, String)> {
        // 清理行内容
        let line = line.trim();
        
        // 尝试多种分隔符
        let separators = ["  ", "\t"];  // 双空格或制表符
        
        for separator in &separators {
            if let Some(pos) = line.find(separator) {
                let word_part = line[..pos].trim();
                let phonemes_part = line[pos..].trim();
                
                if !word_part.is_empty() && !phonemes_part.is_empty() {
                    // 验证单词部分只包含字母和括号
                    if Self::is_valid_word_part(word_part) && Self::is_valid_phonemes_part(phonemes_part) {
                        return Some((word_part.to_string(), phonemes_part.to_string()));
                    }
                }
            }
        }
        
        None
    }
    
    /// 验证单词部分是否有效
    fn is_valid_word_part(word: &str) -> bool {
        if word.is_empty() || word.len() > 50 {
            return false;
        }
        
        // 单词应该主要包含字母，可能有括号和数字
        for ch in word.chars() {
            if !ch.is_ascii_alphabetic() && !matches!(ch, '(' | ')' | '\'' | '-' | '0'..='9') {
                return false;
            }
        }
        
        true
    }
    
    /// 验证音素部分是否有效
    fn is_valid_phonemes_part(phonemes: &str) -> bool {
        if phonemes.is_empty() || phonemes.len() > 100 {
            return false;
        }
        
        // 音素部分应该只包含字母、数字和空格
        for ch in phonemes.chars() {
            if !ch.is_ascii_alphabetic() && !ch.is_ascii_digit() && !ch.is_ascii_whitespace() {
                return false;
            }
        }
        
        // 至少包含一个字母
        phonemes.chars().any(|c| c.is_ascii_alphabetic())
    }
    
    /// 清理单词格式
    fn clean_word(word: &str) -> String {
        // 移除变体标记，如 HELLO(1) -> HELLO
        let cleaned = if let Some(pos) = word.find('(') {
            &word[..pos]
        } else {
            word
        };
        
        // 转换为小写并移除特殊字符
        cleaned.to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphabetic() || *c == '\'')
            .collect()
    }
    
    /// 解析音素字符串 - 更健壮的版本
    fn parse_phonemes(phonemes_str: &str) -> Result<Vec<Phoneme>> {
        let phoneme_tokens: Vec<&str> = phonemes_str
            .split_whitespace()
            .filter(|p| !p.is_empty())
            .collect();
        
        if phoneme_tokens.is_empty() {
            return Err(anyhow::anyhow!("No phonemes found"));
        }
        
        let mut phonemes = Vec::new();
        
        for token in phoneme_tokens {
            // 跳过明显错误的token
            if token.len() > 4 || token.is_empty() {
                continue;
            }
            
            // 验证并创建音素
            if Self::is_valid_arpabet(token) {
                phonemes.push(Phoneme::from_arpabet(token));
            } else {
                // 尝试修复常见错误
                if let Some(fixed) = Self::try_fix_phoneme(token) {
                    phonemes.push(Phoneme::from_arpabet(&fixed));
                } else {
                    eprintln!("Warning: Skipping invalid phoneme: '{}'", token);
                }
            }
        }
        
        if phonemes.is_empty() {
            return Err(anyhow::anyhow!("No valid phonemes after parsing"));
        }
        
        Ok(phonemes)
    }
    
    /// 尝试修复常见的音素错误
    fn try_fix_phoneme(token: &str) -> Option<String> {
        // 移除非ASCII字符
        let cleaned: String = token.chars()
            .filter(|c| c.is_ascii_alphabetic() || c.is_ascii_digit())
            .collect();
        
        if Self::is_valid_arpabet(&cleaned) {
            return Some(cleaned);
        }
        
        None
    }
    
    /// 验证是否是有效的ARPAbet音素 - 改进版
    fn is_valid_arpabet(phoneme: &str) -> bool {
        if phoneme.is_empty() || phoneme.len() > 4 {
            return false;
        }
        
        // 检查字符
        for ch in phoneme.chars() {
            if !ch.is_ascii_alphabetic() && !ch.is_ascii_digit() {
                return false;
            }
        }
        
        // 常见的ARPAbet音素（更完整的列表）
        const VALID_PHONEMES: &[&str] = &[
            // 元音
            "AA", "AE", "AH", "AO", "AW", "AY", "EH", "ER", "EY", 
            "IH", "IY", "OW", "OY", "UH", "UW",
            // 辅音
            "B", "CH", "D", "DH", "F", "G", "HH", "JH", "K", "L", 
            "M", "N", "NG", "P", "R", "S", "SH", "T", "TH", "V", 
            "W", "Y", "Z", "ZH",
            // 一些变体
            "Q", "X"
        ];
        
        // 移除重音标记进行检查
        let base_phoneme = if phoneme.chars().last().map_or(false, |c| c.is_ascii_digit()) {
            if phoneme.len() > 1 {
                &phoneme[..phoneme.len()-1]
            } else {
                return false;  // 只有数字的不是有效音素
            }
        } else {
            phoneme
        };
        
        VALID_PHONEMES.contains(&base_phoneme.to_uppercase().as_str())
    }
    
    /// 查找单词的发音
    pub fn lookup(&self, word: &str) -> Option<Vec<Phoneme>> {
        self.entries.get(&word.to_lowercase()).cloned()
    }
    
    /// 获取词典大小
    pub fn size(&self) -> usize {
        self.entries.len()
    }
    
    /// 添加自定义词条
    pub fn add_entry(&mut self, word: String, phonemes: Vec<Phoneme>) {
        self.entries.insert(word.to_lowercase(), phonemes);
    }
    
    /// 检查词典是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// 获取词典中的所有单词（排序后的前N个）
    pub fn get_sample_words(&self, count: usize) -> Vec<String> {
        let mut words: Vec<String> = self.entries.keys().cloned().collect();
        words.sort();
        words.into_iter().take(count).collect()
    }
}