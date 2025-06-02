use std::fmt;

/// 表示一个音素
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Phoneme {
    pub symbol: String,
    pub stress: StressLevel,
    pub features: PhonemeFeatures,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StressLevel {
    Primary,      // 1
    Secondary,    // 2  
    Unstressed,   // 0
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhonemeFeatures {
    pub phoneme_type: PhonemeType,
    pub manner: Option<Manner>,
    pub place: Option<Place>,
    pub voicing: Option<Voicing>,
    pub height: Option<Height>,     // 元音高度
    pub backness: Option<Backness>, // 元音前后位置
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhonemeType {
    Vowel,
    Consonant,
    Special,  // 停顿、边界等
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Manner {
    Stop,
    Fricative,
    Affricate,
    Nasal,
    Liquid,
    Glide,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Place {
    Bilabial,
    Labiodental,
    Dental,
    Alveolar,
    Postalveolar,
    Palatal,
    Velar,
    Glottal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Voicing {
    Voiced,
    Voiceless,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Height {
    High,
    Mid,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Backness {
    Front,
    Central,
    Back,
}

impl Phoneme {
    /// 从ARPAbet符号创建音素
    pub fn from_arpabet(symbol: &str) -> Self {
        let (base_symbol, stress) = Self::parse_stress(symbol);
        let features = Self::get_arpabet_features(&base_symbol);
        
        Self {
            symbol: base_symbol,
            stress,
            features,
        }
    }
    
    /// 创建词边界标记
    pub fn word_boundary() -> Self {
        Self {
            symbol: " ".to_string(),
            stress: StressLevel::Unstressed,
            features: PhonemeFeatures {
                phoneme_type: PhonemeType::Special,
                manner: None,
                place: None,
                voicing: None,
                height: None,
                backness: None,
            },
        }
    }
    
    /// 解析重音标记
    fn parse_stress(symbol: &str) -> (String, StressLevel) {
        if symbol.ends_with('0') {
            (symbol[..symbol.len()-1].to_string(), StressLevel::Unstressed)
        } else if symbol.ends_with('1') {
            (symbol[..symbol.len()-1].to_string(), StressLevel::Primary)
        } else if symbol.ends_with('2') {
            (symbol[..symbol.len()-1].to_string(), StressLevel::Secondary)
        } else {
            (symbol.to_string(), StressLevel::Unstressed)
        }
    }
    
    /// 获取ARPAbet音素的特征
    fn get_arpabet_features(symbol: &str) -> PhonemeFeatures {
        match symbol {
            // 元音
            "AA" => PhonemeFeatures::vowel(Height::Low, Backness::Back),
            "AE" => PhonemeFeatures::vowel(Height::Low, Backness::Front),
            "AH" => PhonemeFeatures::vowel(Height::Mid, Backness::Central),
            "AO" => PhonemeFeatures::vowel(Height::Mid, Backness::Back),
            "AW" => PhonemeFeatures::vowel(Height::Low, Backness::Central), // 双元音
            "AY" => PhonemeFeatures::vowel(Height::Low, Backness::Central), // 双元音
            "EH" => PhonemeFeatures::vowel(Height::Mid, Backness::Front),
            "ER" => PhonemeFeatures::vowel(Height::Mid, Backness::Central),
            "EY" => PhonemeFeatures::vowel(Height::Mid, Backness::Front), // 双元音
            "IH" => PhonemeFeatures::vowel(Height::High, Backness::Front),
            "IY" => PhonemeFeatures::vowel(Height::High, Backness::Front),
            "OW" => PhonemeFeatures::vowel(Height::Mid, Backness::Back), // 双元音
            "OY" => PhonemeFeatures::vowel(Height::Mid, Backness::Back), // 双元音
            "UH" => PhonemeFeatures::vowel(Height::High, Backness::Back),
            "UW" => PhonemeFeatures::vowel(Height::High, Backness::Back),
            
            // 辅音
            "B" => PhonemeFeatures::consonant(Manner::Stop, Place::Bilabial, Voicing::Voiced),
            "CH" => PhonemeFeatures::consonant(Manner::Affricate, Place::Postalveolar, Voicing::Voiceless),
            "D" => PhonemeFeatures::consonant(Manner::Stop, Place::Alveolar, Voicing::Voiced),
            "DH" => PhonemeFeatures::consonant(Manner::Fricative, Place::Dental, Voicing::Voiced),
            "F" => PhonemeFeatures::consonant(Manner::Fricative, Place::Labiodental, Voicing::Voiceless),
            "G" => PhonemeFeatures::consonant(Manner::Stop, Place::Velar, Voicing::Voiced),
            "HH" => PhonemeFeatures::consonant(Manner::Fricative, Place::Glottal, Voicing::Voiceless),
            "JH" => PhonemeFeatures::consonant(Manner::Affricate, Place::Postalveolar, Voicing::Voiced),
            "K" => PhonemeFeatures::consonant(Manner::Stop, Place::Velar, Voicing::Voiceless),
            "L" => PhonemeFeatures::consonant(Manner::Liquid, Place::Alveolar, Voicing::Voiced),
            "M" => PhonemeFeatures::consonant(Manner::Nasal, Place::Bilabial, Voicing::Voiced),
            "N" => PhonemeFeatures::consonant(Manner::Nasal, Place::Alveolar, Voicing::Voiced),
            "NG" => PhonemeFeatures::consonant(Manner::Nasal, Place::Velar, Voicing::Voiced),
            "P" => PhonemeFeatures::consonant(Manner::Stop, Place::Bilabial, Voicing::Voiceless),
            "R" => PhonemeFeatures::consonant(Manner::Liquid, Place::Alveolar, Voicing::Voiced),
            "S" => PhonemeFeatures::consonant(Manner::Fricative, Place::Alveolar, Voicing::Voiceless),
            "SH" => PhonemeFeatures::consonant(Manner::Fricative, Place::Postalveolar, Voicing::Voiceless),
            "T" => PhonemeFeatures::consonant(Manner::Stop, Place::Alveolar, Voicing::Voiceless),
            "TH" => PhonemeFeatures::consonant(Manner::Fricative, Place::Dental, Voicing::Voiceless),
            "V" => PhonemeFeatures::consonant(Manner::Fricative, Place::Labiodental, Voicing::Voiced),
            "W" => PhonemeFeatures::consonant(Manner::Glide, Place::Bilabial, Voicing::Voiced),
            "Y" => PhonemeFeatures::consonant(Manner::Glide, Place::Palatal, Voicing::Voiced),
            "Z" => PhonemeFeatures::consonant(Manner::Fricative, Place::Alveolar, Voicing::Voiced),
            "ZH" => PhonemeFeatures::consonant(Manner::Fricative, Place::Postalveolar, Voicing::Voiced),
            
            _ => PhonemeFeatures::default(),
        }
    }
    
    pub fn is_vowel(&self) -> bool {
        matches!(self.features.phoneme_type, PhonemeType::Vowel)
    }
    
    pub fn is_consonant(&self) -> bool {
        matches!(self.features.phoneme_type, PhonemeType::Consonant)
    }
}

impl PhonemeFeatures {
    fn vowel(height: Height, backness: Backness) -> Self {
        Self {
            phoneme_type: PhonemeType::Vowel,
            manner: None,
            place: None,
            voicing: None,
            height: Some(height),
            backness: Some(backness),
        }
    }
    
    fn consonant(manner: Manner, place: Place, voicing: Voicing) -> Self {
        Self {
            phoneme_type: PhonemeType::Consonant,
            manner: Some(manner),
            place: Some(place),
            voicing: Some(voicing),
            height: None,
            backness: None,
        }
    }
    
    fn default() -> Self {
        Self {
            phoneme_type: PhonemeType::Special,
            manner: None,
            place: None,
            voicing: None,
            height: None,
            backness: None,
        }
    }
}

impl fmt::Display for Phoneme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stress_mark = match self.stress {
            StressLevel::Primary => "1",
            StressLevel::Secondary => "2", 
            StressLevel::Unstressed => "0",
        };
        
        if self.symbol == " " {
            write!(f, " ")
        } else {
            write!(f, "{}{}", self.symbol, stress_mark)
        }
    }
}