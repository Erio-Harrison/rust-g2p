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
    /// 加载英语规则
    pub fn load_english_rules(path: &str) -> Result<Self> {
        let mut engine = Self {
            rules: Vec::new(),
            rule_groups: HashMap::new(),
            irregular_words: HashMap::new(),
        };
        
        // 加载内置规则
        engine.load_builtin_rules();
        
        // 加载不规则词汇
        engine.load_irregular_words();
        
        // 如果文件存在，也加载文件规则
        if std::path::Path::new(path).exists() {
            let content = fs::read_to_string(path)?;
            engine.parse_rules(&content)?;
        }
        
        engine.build_index();
        
        Ok(engine)
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
        if context == "_" {
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
        if context == "_" {
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
    
    /// 加载不规则词汇
    fn load_irregular_words(&mut self) {
        let irregular = vec![
            // 完全不规则的发音
            ("colonel", vec!["K", "ER1", "N", "AH0", "L"]),
            ("yacht", vec!["Y", "AA1", "T"]),
            ("island", vec!["AY1", "L", "AH0", "N", "D"]),
            ("aisle", vec!["AY1", "L"]),
            ("isle", vec!["AY1", "L"]),
            
            // 无声字母
            ("knight", vec!["N", "AY1", "T"]),
            ("knee", vec!["N", "IY1"]),
            ("knife", vec!["N", "AY1", "F"]),
            ("know", vec!["N", "OW1"]),
            ("gnome", vec!["N", "OW1", "M"]),
            ("gnat", vec!["N", "AE1", "T"]),
            ("write", vec!["R", "AY1", "T"]),
            ("wrong", vec!["R", "AO1", "NG"]),
            ("wrist", vec!["R", "IH1", "S", "T"]),
            ("lamb", vec!["L", "AE1", "M"]),
            ("comb", vec!["K", "OW1", "M"]),
            ("tomb", vec!["T", "UW1", "M"]),
            ("thumb", vec!["TH", "AH1", "M"]),
            ("debt", vec!["D", "EH1", "T"]),
            ("doubt", vec!["D", "AW1", "T"]),
            
            // 复杂的单词
            ("psychology", vec!["S", "AY0", "K", "AA1", "L", "AH0", "JH", "IY0"]),
            ("pneumonia", vec!["N", "UW0", "M", "OW1", "N", "Y", "AH0"]),
            ("rhythm", vec!["R", "IH1", "DH", "AH0", "M"]),
            ("phone", vec!["F", "OW1", "N"]),
            ("graph", vec!["G", "R", "AE1", "F"]),
            ("laugh", vec!["L", "AE1", "F"]),
            ("cough", vec!["K", "AO1", "F"]),
            ("rough", vec!["R", "AH1", "F"]),
            ("tough", vec!["T", "AH1", "F"]),
            ("enough", vec!["IH0", "N", "AH1", "F"]),
            
            // 常见的不规则词
            ("one", vec!["W", "AH1", "N"]),
            ("once", vec!["W", "AH1", "N", "S"]),
            ("two", vec!["T", "UW1"]),
            ("eight", vec!["EY1", "T"]),
            ("women", vec!["W", "IH1", "M", "AH0", "N"]),
            ("woman", vec!["W", "UH1", "M", "AH0", "N"]),
            ("busy", vec!["B", "IH1", "Z", "IY0"]),
            ("business", vec!["B", "IH1", "Z", "N", "AH0", "S"]),
            ("minute", vec!["M", "AY0", "N", "UW1", "T"]), // 作为形容词"微小的"
        ];
        
        for (word, phonemes) in irregular {
            self.irregular_words.insert(
                word.to_string(), 
                phonemes.into_iter().map(|s| s.to_string()).collect()
            );
        }
    }
    
    /// 加载内置规则 - 大幅扩展版本
    fn load_builtin_rules(&mut self) {
        let builtin_rules = vec![
            // 无声字母组合 (优先级最高)
            ("kn", "", "", vec!["N"], vec![RuleCondition::WordStart], 10),
            ("gn", "", "", vec!["N"], vec![RuleCondition::WordStart], 10),
            ("wr", "", "", vec!["R"], vec![RuleCondition::WordStart], 10),
            ("ps", "", "", vec!["S"], vec![RuleCondition::WordStart], 10),
            ("pn", "", "", vec!["N"], vec![RuleCondition::WordStart], 10),
            ("pt", "", "", vec!["T"], vec![RuleCondition::WordStart], 10),
            
            // 词尾无声字母
            ("mb", "", "", vec!["M"], vec![RuleCondition::WordEnd], 8),
            ("bt", "", "", vec!["T"], vec![RuleCondition::WordEnd], 8),
            ("mn", "", "", vec!["M"], vec![RuleCondition::WordEnd], 8),
            
            // TH音 (需要区分清浊音，这里简化)
            ("th", "", "", vec!["TH"], vec![], 5),
            
            // CH音和相关
            ("ch", "", "", vec!["CH"], vec![], 5),
            ("tch", "", "", vec!["CH"], vec![], 6),
            ("sch", "", "", vec!["S", "K"], vec![], 6),
            
            // SH音
            ("sh", "", "", vec!["SH"], vec![], 5),
            ("ti", "", "on", vec!["SH"], vec![], 6), // nation, station
            ("ci", "", "an", vec!["SH"], vec![], 6), // musician
            ("si", "", "on", vec!["ZH"], vec![], 6), // vision
            
            // PH音
            ("ph", "", "", vec!["F"], vec![], 5),
            
            // GH音的各种情况
            ("gh", "", "", vec!["F"], vec![RuleCondition::WordEnd], 6), // laugh, rough
            ("gh", "", "", vec![], vec![], 5), // night, right (通常不发音)
            ("ght", "", "", vec!["T"], vec![], 6), // night, fight
            
            // NG音
            ("ng", "", "", vec!["NG"], vec![], 5),
            ("nk", "", "", vec!["NG", "K"], vec![], 5),
            
            // QU组合
            ("qu", "", "", vec!["K", "W"], vec![], 5),
            
            // X的处理
            ("x", "", "", vec!["K", "S"], vec![], 3),
            ("x", "_", "", vec!["Z"], vec![RuleCondition::WordStart], 4), // xylophone
            
            // CK组合
            ("ck", "", "", vec!["K"], vec![], 5),
            
            // DG组合
            ("dge", "", "", vec!["JH"], vec![RuleCondition::WordEnd], 6), // bridge, edge
            
            // 复杂元音组合 (按长度排序，长的优先)
            ("ough", "", "", vec!["AH1", "F"], vec![], 8), // rough, tough
            ("augh", "", "", vec!["AO1", "F"], vec![], 8), // laugh
            ("eigh", "", "", vec!["EY1"], vec![], 8), // eight, weigh
            
            // 四字母组合
            ("tion", "", "", vec!["SH", "AH0", "N"], vec![], 7),
            ("sion", "", "", vec!["ZH", "AH0", "N"], vec![], 7),
            ("ture", "", "", vec!["CH", "ER0"], vec![], 7), // nature, picture
            
            // 三字母元音组合
            ("eau", "", "", vec!["OW1"], vec![], 6), // beauty
            ("ieu", "", "", vec!["UW1"], vec![], 6), // lieu
            ("oor", "", "", vec!["UH1", "R"], vec![], 6), // poor, door
            ("ear", "", "", vec!["IH1", "R"], vec![], 6), // hear, fear
            ("eer", "", "", vec!["IH1", "R"], vec![], 6), // beer, deer
            ("air", "", "", vec!["EH1", "R"], vec![], 6), // fair, hair
            ("are", "", "", vec!["EH1", "R"], vec![], 6), // care, share
            ("ore", "", "", vec!["AO1", "R"], vec![], 6), // more, store
            ("our", "", "", vec!["AW1", "R"], vec![], 6), // hour, sour
            
            // 双字母元音组合
            ("ai", "", "", vec!["EY1"], vec![], 5),
            ("ay", "", "", vec!["EY1"], vec![], 5),
            ("au", "", "", vec!["AO1"], vec![], 5),
            ("aw", "", "", vec!["AO1"], vec![], 5),
            ("ea", "", "", vec!["IY1"], vec![], 5), // 默认情况，如 eat
            ("ee", "", "", vec!["IY1"], vec![], 5),
            ("ei", "", "", vec!["EY1"], vec![], 5), // 默认情况
            ("eu", "", "", vec!["Y", "UW1"], vec![], 5),
            ("ey", "", "", vec!["EY1"], vec![], 5),
            ("ie", "", "", vec!["IY1"], vec![], 5), // 默认情况，如 piece
            ("oa", "", "", vec!["OW1"], vec![], 5),
            ("oe", "", "", vec!["OW1"], vec![], 5),
            ("oi", "", "", vec!["OY1"], vec![], 5),
            ("oo", "", "", vec!["UW1"], vec![], 5), // 默认情况，如 food
            ("ou", "", "", vec!["AW1"], vec![], 5), // 默认情况，如 house
            ("ow", "", "", vec!["AW1"], vec![], 5), // 默认情况，如 cow
            ("oy", "", "", vec!["OY1"], vec![], 5),
            ("ue", "", "", vec!["UW1"], vec![], 5),
            ("ui", "", "", vec!["UW1"], vec![], 5),
            
            // 一些特殊的双字母组合
            ("al", "", "", vec!["AO1", "L"], vec![], 4), // all, call
            ("ar", "", "", vec!["AA1", "R"], vec![], 4),
            ("er", "", "", vec!["ER1"], vec![], 4),
            ("ir", "", "", vec!["ER1"], vec![], 4),
            ("or", "", "", vec!["AO1", "R"], vec![], 4),
            ("ur", "", "", vec!["ER1"], vec![], 4),
            
            // C的上下文规则
            ("c", "", "e", vec!["S"], vec![], 4),
            ("c", "", "i", vec!["S"], vec![], 4),
            ("c", "", "y", vec!["S"], vec![], 4),
            
            // G的上下文规则
            ("g", "", "e", vec!["JH"], vec![], 4),
            ("g", "", "i", vec!["JH"], vec![], 4),
            ("g", "", "y", vec!["JH"], vec![], 4),
            
            // Y的特殊处理
            ("y", "", "", vec!["AY1"], vec![RuleCondition::WordEnd], 4), // 词尾y读/aɪ/
            ("y", "", "", vec!["IH0"], vec![], 3), // 其他情况读/ɪ/
            
            // E的特殊处理
            ("e", "", "_", vec![], vec![RuleCondition::WordEnd], 4), // 词尾e通常不发音
            
            // 基本元音 (优先级较低)
            ("a", "", "", vec!["AE0"], vec![], 2),
            ("e", "", "", vec!["EH0"], vec![], 2),
            ("i", "", "", vec!["IH0"], vec![], 2),
            ("o", "", "", vec!["AA0"], vec![], 2),
            ("u", "", "", vec!["AH0"], vec![], 2),
            
            // 基本辅音
            ("b", "", "", vec!["B"], vec![], 2),
            ("c", "", "", vec!["K"], vec![], 2),
            ("d", "", "", vec!["D"], vec![], 2),
            ("f", "", "", vec!["F"], vec![], 2),
            ("g", "", "", vec!["G"], vec![], 2),
            ("h", "", "", vec!["HH"], vec![], 2),
            ("j", "", "", vec!["JH"], vec![], 2),
            ("k", "", "", vec!["K"], vec![], 2),
            ("l", "", "", vec!["L"], vec![], 2),
            ("m", "", "", vec!["M"], vec![], 2),
            ("n", "", "", vec!["N"], vec![], 2),
            ("p", "", "", vec!["P"], vec![], 2),
            ("r", "", "", vec!["R"], vec![], 2),
            ("s", "", "", vec!["S"], vec![], 2),
            ("t", "", "", vec!["T"], vec![], 2),
            ("v", "", "", vec!["V"], vec![], 2),
            ("w", "", "", vec!["W"], vec![], 2),
            ("z", "", "", vec!["Z"], vec![], 2),
        ];
        
        for (pattern, left, right, phonemes, conditions, priority) in builtin_rules {
            let rule = Rule {
                pattern: pattern.to_string(),
                left_context: if left.is_empty() { None } else { Some(left.to_string()) },
                right_context: if right.is_empty() { None } else { Some(right.to_string()) },
                phonemes: phonemes.into_iter().map(|s| s.to_string()).collect(),
                priority,
                conditions,
            };
            self.rules.push(rule);
        }
        
        // 按优先级排序，优先级高的在前
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    fn parse_rules(&mut self, _content: &str) -> Result<()> {
        // TODO: 实现规则文件解析
        Ok(())
    }
    
    /// 获取规则数量
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}