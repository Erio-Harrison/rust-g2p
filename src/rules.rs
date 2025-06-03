use crate::phoneme::Phoneme;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;

/// 规则引擎
pub struct RulesEngine {
    rules: Vec<Rule>,
    rule_groups: HashMap<char, Vec<usize>>, // 按首字母分组的规则索引
    irregular_words: HashMap<String, Vec<String>>, // 不规则词汇
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub pattern: String,                    // 匹配模式
    pub left_context: Option<String>,       // 左上下文
    pub right_context: Option<String>,      // 右上下文
    pub phonemes: Vec<String>,             // 输出音素
    pub priority: usize,                   // 优先级（模式长度）
    pub conditions: Vec<RuleCondition>,    // 额外条件
}

#[derive(Debug, Clone)]
pub enum RuleCondition {
    WordStart,      // 词首
    WordEnd,        // 词尾
    BeforeVowel,    // 元音前
    AfterVowel,     // 元音后
    Stressed,       // 重音音节
    Unstressed,     // 非重音音节
}

impl RulesEngine {
    /// 加载英语规则 - 仅从文件加载
    pub fn load_english_rules(rules_path: &str) -> Result<Self> {
        let mut engine = Self {
            rules: Vec::new(),
            rule_groups: HashMap::new(),
            irregular_words: HashMap::new(),
        };
        
        // 从文件加载规则和不规则词汇
        let content = fs::read_to_string(rules_path)
            .map_err(|e| anyhow::anyhow!("Failed to read rules file '{}': {}", rules_path, e))?;
        engine.parse_rules(&content)?;
        
        engine.build_index();
        
        Ok(engine)
    }
    
    /// 解析规则文件
    fn parse_rules(&mut self, content: &str) -> Result<()> {
        for line in content.lines() {
            let line = line.trim();
            
            // 跳过空行和注释行
            if line.is_empty() || line.starts_with('#') || line.starts_with("=") {
                continue;
            }
            
            // 处理不规则词汇
            if line.starts_with("IRREGULAR|") {
                self.parse_irregular_word(line)?;
                continue;
            }
            
            // 解析常规规则，格式：pattern|left_context|right_context|phonemes|priority|conditions
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 4 {
                continue; // 跳过格式不正确的行
            }
            
            let pattern = parts[0].to_string();
            let left_context = if parts.len() > 1 && !parts[1].is_empty() {
                Some(parts[1].to_string())
            } else {
                None
            };
            let right_context = if parts.len() > 2 && !parts[2].is_empty() {
                Some(parts[2].to_string())
            } else {
                None
            };
            
            // 解析音素列表（用空格分隔）
            let phonemes: Vec<String> = if parts[3] == "SILENT" {
                Vec::new() // 静音规则
            } else {
                parts[3]
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect()
            };
            
            // 解析优先级
            let priority = if parts.len() > 4 && !parts[4].is_empty() {
                parts[4].parse::<usize>().unwrap_or(pattern.len())
            } else {
                pattern.len()
            };
            
            // 解析条件
            let conditions = if parts.len() > 5 && !parts[5].is_empty() {
                self.parse_conditions(parts[5])?
            } else {
                Vec::new()
            };
            
            let rule = Rule {
                pattern,
                left_context,
                right_context,
                phonemes,
                priority,
                conditions,
            };
            
            self.rules.push(rule);
        }
        
        // 按优先级排序，优先级高的在前
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }
    
    /// 解析不规则词汇行
    fn parse_irregular_word(&mut self, line: &str) -> Result<()> {
        // 格式：IRREGULAR|word|phoneme1 phoneme2 phoneme3
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            let word = parts[1].trim().to_lowercase();
            let phonemes: Vec<String> = parts[2]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            
            if !word.is_empty() && !phonemes.is_empty() {
                self.irregular_words.insert(word, phonemes);
            }
        }
        Ok(())
    }
    
    /// 解析条件字符串
    fn parse_conditions(&self, conditions_str: &str) -> Result<Vec<RuleCondition>> {
        let mut conditions = Vec::new();
        
        for condition in conditions_str.split(',') {
            let condition = condition.trim();
            match condition {
                "START" => conditions.push(RuleCondition::WordStart),
                "END" => conditions.push(RuleCondition::WordEnd),
                "VOWEL_BEFORE" => conditions.push(RuleCondition::BeforeVowel),
                "VOWEL_AFTER" => conditions.push(RuleCondition::AfterVowel),
                "word_start" => conditions.push(RuleCondition::WordStart),
                "word_end" => conditions.push(RuleCondition::WordEnd),
                "before_vowel" => conditions.push(RuleCondition::BeforeVowel),
                "after_vowel" => conditions.push(RuleCondition::AfterVowel),
                "stressed" => conditions.push(RuleCondition::Stressed),
                "unstressed" => conditions.push(RuleCondition::Unstressed),
                _ => {} // 忽略未知条件
            }
        }
        
        Ok(conditions)
    }
    
    /// 应用规则到单词
    pub fn apply_rules(&self, word: &str) -> Result<Vec<Phoneme>> {
        // 首先检查不规则词汇
        if let Some(phonemes) = self.irregular_words.get(&word.to_lowercase()) {
            return Ok(phonemes.iter().map(|p| Phoneme::from_arpabet(p)).collect());
        }
        
        let mut phonemes = Vec::new();
        let mut pos = 0;
        let word_chars: Vec<char> = word.chars().collect();
        
        while pos < word_chars.len() {
            match self.find_best_rule(&word_chars, pos) {
                Ok(rule) => {
                    // 添加规则输出的音素
                    for phoneme_str in &rule.phonemes {
                        if !phoneme_str.is_empty() {
                            phonemes.push(Phoneme::from_arpabet(phoneme_str));
                        }
                    }
                    
                    // 前进位置
                    pos += rule.pattern.chars().count();
                }
                Err(_) => {
                    // 如果找不到规则，使用默认处理
                    let current_char = word_chars[pos];
                    
                    if let Some(default_phoneme) = Self::get_default_phoneme(current_char) {
                        phonemes.push(Phoneme::from_arpabet(&default_phoneme));
                    }
                    
                    pos += 1;
                }
            }
        }
        
        Ok(phonemes)
    }
    
    /// 获取字符的默认音素
    fn get_default_phoneme(ch: char) -> Option<String> {
        match ch.to_ascii_lowercase() {
            'a' => Some("AE0".to_string()),
            'b' => Some("B".to_string()),
            'c' => Some("K".to_string()),
            'd' => Some("D".to_string()),
            'e' => Some("EH0".to_string()),
            'f' => Some("F".to_string()),
            'g' => Some("G".to_string()),
            'h' => Some("HH".to_string()),
            'i' => Some("IH0".to_string()),
            'j' => Some("JH".to_string()),
            'k' => Some("K".to_string()),
            'l' => Some("L".to_string()),
            'm' => Some("M".to_string()),
            'n' => Some("N".to_string()),
            'o' => Some("OW0".to_string()),
            'p' => Some("P".to_string()),
            'q' => Some("K".to_string()),
            'r' => Some("R".to_string()),
            's' => Some("S".to_string()),
            't' => Some("T".to_string()),
            'u' => Some("UH0".to_string()),
            'v' => Some("V".to_string()),
            'w' => Some("W".to_string()),
            'x' => Some("K".to_string()),
            'y' => Some("Y".to_string()),
            'z' => Some("Z".to_string()),
            _ => None,
        }
    }
    
    /// 查找最佳匹配规则
    fn find_best_rule(&self, word: &[char], pos: usize) -> Result<&Rule> {
        let current_char = word[pos];
        let mut best_rule: Option<&Rule> = None;
        let mut best_priority = 0;
        
        // 获取当前字符的候选规则
        if let Some(rule_indices) = self.rule_groups.get(&current_char) {
            for &rule_idx in rule_indices {
                let rule = &self.rules[rule_idx];
                
                if let Some(priority) = self.rule_matches(rule, word, pos) {
                    if priority > best_priority {
                        best_priority = priority;
                        best_rule = Some(rule);
                    }
                }
            }
        }
        
        best_rule.ok_or_else(|| {
            anyhow::anyhow!("No rule found for character '{}' at position {}", word[pos], pos)
        })
    }
    
    /// 检查规则是否匹配
    fn rule_matches(&self, rule: &Rule, word: &[char], pos: usize) -> Option<usize> {
        // 1. 检查模式匹配
        let pattern_chars: Vec<char> = rule.pattern.chars().collect();
        if pos + pattern_chars.len() > word.len() {
            return None;
        }
        
        for (i, &pattern_char) in pattern_chars.iter().enumerate() {
            if word[pos + i].to_ascii_lowercase() != pattern_char {
                return None;
            }
        }
        
        // 2. 检查上下文条件
        if let Some(ref right_ctx) = rule.right_context {
            if !self.check_right_context(right_ctx, word, pos + pattern_chars.len()) {
                return None;
            }
        }
        
        if let Some(ref left_ctx) = rule.left_context {
            if !self.check_left_context(left_ctx, word, pos) {
                return None;
            }
        }
        
        // 3. 检查其他条件
        for condition in &rule.conditions {
            if !self.check_condition(condition, word, pos) {
                return None;
            }
        }
        
        Some(rule.priority)
    }
    
    /// 检查右上下文
    fn check_right_context(&self, context: &str, word: &[char], pos: usize) -> bool {
        if context == "END" {
            return pos >= word.len(); // 词尾
        }
        
        let context_chars: Vec<char> = context.chars().collect();
        if pos + context_chars.len() > word.len() {
            return false;
        }
        
        for (i, &ctx_char) in context_chars.iter().enumerate() {
            if word[pos + i].to_ascii_lowercase() != ctx_char {
                return false;
            }
        }
        
        true
    }
    
    /// 检查左上下文
    fn check_left_context(&self, context: &str, word: &[char], pos: usize) -> bool {
        if context == "START" {
            return pos == 0; // 词首
        }
        
        let context_chars: Vec<char> = context.chars().collect();
        if context_chars.len() > pos {
            return false;
        }
        
        let start = pos - context_chars.len();
        for (i, &ctx_char) in context_chars.iter().enumerate() {
            if word[start + i].to_ascii_lowercase() != ctx_char {
                return false;
            }
        }
        
        true
    }
    
    /// 检查条件
    fn check_condition(&self, condition: &RuleCondition, word: &[char], pos: usize) -> bool {
        match condition {
            RuleCondition::WordStart => pos == 0,
            RuleCondition::WordEnd => pos == word.len() - 1,
            RuleCondition::BeforeVowel => {
                if pos + 1 < word.len() {
                    self.is_vowel(word[pos + 1])
                } else {
                    false
                }
            }
            RuleCondition::AfterVowel => {
                if pos > 0 {
                    self.is_vowel(word[pos - 1])
                } else {
                    false
                }
            }
            _ => true,
        }
    }
    
    /// 判断是否为元音
    fn is_vowel(&self, ch: char) -> bool {
        matches!(ch.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
    }
    
    /// 构建规则索引
    fn build_index(&mut self) {
        for (idx, rule) in self.rules.iter().enumerate() {
            if let Some(first_char) = rule.pattern.chars().next() {
                self.rule_groups
                    .entry(first_char)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }
    }
    
    /// 获取规则数量
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}