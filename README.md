# Rust G2P - 纯Rust文本到音素转换器

一个从零开始实现的文本到音素(Grapheme-to-Phoneme, G2P)转换器，用于学习和理解TTS系统的核心原理。

## 🎯 项目目标

这是一个教育性质的项目，旨在：
- 深入理解Text-to-Speech系统的G2P组件
- 学习自然语言处理中的音韵学规则
- 实践Rust在机器学习和NLP领域的应用
- 探索从规则驱动到数据驱动的G2P方法

## 🏗️ 项目架构

```
rust-g2p/
├── src/
│   ├── lib.rs              # 主要API接口
│   ├── phoneme.rs          # 音素系统和ARPAbet支持
│   ├── dict.rs             # CMU词典加载和查找
│   ├── rules.rs            # 规则引擎和模式匹配
│   ├── text.rs             # 文本预处理和标准化
│   └── lang.rs             # 语言特定处理(扩展用)
├── data/
│   ├── cmudict.txt         # CMU发音词典
│   └── en_rules.txt        # 英语发音规则(可选)
├── examples/
│   ├── basic_usage.rs      # 基础使用示例
│   ├── benchmark.rs        # 性能基准测试
│   ├── compare_with_espeak.rs # 与espeak-ng对比
│   └── verify_cmu_dict.rs  # 词典验证工具
└── tests/
    └── integration_tests.rs # 集成测试
```

## ⚙️核心组件

### 1. 音素系统 (`phoneme.rs`)
```rust
pub struct Phoneme {
    pub symbol: String,           // ARPAbet符号
    pub stress: StressLevel,      // 重音级别 (0/1/2)
    pub features: PhonemeFeatures, // 音韵特征
}
```

### 2. 词典系统 (`dict.rs`)
- 支持完整的CMU发音词典(133k+词条)
- O(1)哈希查找性能
- 自动处理编码和格式问题
- 支持词汇变体(如WORD(1), WORD(2))

### 3. 规则引擎 (`rules.rs`)
```rust
pub struct RulesEngine {
    rules: Vec<Rule>,                           // 发音规则
    rule_groups: HashMap<char, Vec<usize>>,     // 索引优化
    irregular_words: HashMap<String, Vec<String>>, // 不规则词汇
}
```

**规则类型:**
- 基础字母映射: `a → AE0`, `b → B`
- 字母组合: `ch → CH`, `th → TH`, `ph → F`
- 上下文规则: `c+e → S`, `g+i → JH`
- 无声字母: `kn- → N`, `wr- → R`, `-mb → M`
- 不规则词汇: `colonel → K ER1 N AH0 L`

### 4. 文本处理 (`text.rs`)
- 数字展开: `5 → five`, `25 → twenty five`
- 缩写处理: `Dr. → doctor`, `Mrs. → misses`
- 标点清理和空格标准化
- 智能分词算法

## 🚀 快速开始

### 安装

```bash
git clone https://github.com/Erio-Harrison/rust-g2p.git
cd rust-g2p

# 下载CMU词典
mkdir -p data
curl -o data/cmudict.txt "https://raw.githubusercontent.com/cmusphinx/cmudict/master/cmudict.dict"
```

### 基础使用

```rust
use rust_g2p::RustG2P;

fn main() -> anyhow::Result<()> {
    // 初始化G2P转换器
    let g2p = RustG2P::new()?;
    
    // 单词转换
    let phonemes = g2p.word_to_phonemes("hello")?;
    println!("hello -> {:?}", phonemes); // [HH, EH0, L, OW1]
    
    // 句子转换
    let phonemes = g2p.text_to_phonemes("Hello, Dr. Smith! I have 5 cats.")?;
    for phoneme in phonemes {
        if phoneme.symbol == " " {
            print!("| ");
        } else {
            print!("{} ", phoneme);
        }
    }
    
    Ok(())
}
```

### 运行示例

```bash
# 基础功能演示
cargo run --example basic_usage

# 性能基准测试
cargo run --example benchmark

# 与espeak-ng对比(需要安装espeak-rs)
cargo run --example compare_with_espeak

# 验证CMU词典加载
cargo run --example verify_cmu_dict
```

## 📊 性能表现

在现代硬件上的基准测试结果：

```
=== Performance Results ===
Total time: 122.8ms
Average time per conversion: 122.8μs
Conversions per second: 8,140
Words processed per second: 219,780
Characters processed per second: 1,905,160
```

### 性能特点
- ⚡ **极快转换**: 平均122μs每次转换
- 💾 **低内存**: 词典加载后约150MB内存占用
- 🔄 **高吞吐**: 每秒处理22万单词
- 📦 **零依赖**: 核心功能无外部运行时依赖

## 🎯 准确性评估

与espeak-ng的对比测试：

| 单词 | Rust G2P | espeak-ng | 状态 |
|------|----------|-----------|------|
| hello | HH EH L OW | həlˈəʊ | ✅ 基本正确 |
| knight | N AY T | nˈaɪt | ✅ 无声字母处理 |
| yacht | Y AA T | jˈɒt | ✅ 不规则发音 |
| colonel | K ER N AH L | kˈɜːnəl | ⚠️ 需改进 |
| psychology | S AY K AA L AH JH IY | saɪkˈɒlədʒi | ⚠️ 缺少P音 |

### 当前限制
- 重音预测算法较简单
- 复杂上下文规则不完善
- 缺少音节划分功能
- 形态学分析有限

## 🛠️ 开发指南

### 添加新规则

在`src/rules.rs`的`load_builtin_rules()`中添加：

```rust
("newpattern", "left", "right", vec!["PH1", "ON2"], vec![RuleCondition::WordStart], 5),
```

### 添加不规则词汇

在`load_irregular_words()`中添加：

```rust
("newword", vec!["N", "UW1", "W", "ER0", "D"]),
```

### 扩展语言支持

1. 创建新的音素集合
2. 实现语言特定的规则
3. 添加对应的词典文件
4. 在`lang.rs`中实现`Language` trait

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行基准测试
cargo bench

# 测试特定功能
cargo test test_cmu_dict
```

## 🔮 未来改进

### 短期目标 (1-2周)
- [ ] 完善不规则词汇处理
- [ ] 改进重音预测算法
- [ ] 添加更多上下文敏感规则
- [ ] 优化音节划分

### 中期目标 (1-2月)
- [ ] 实现统计G2P模型
- [ ] 添加多语言支持
- [ ] 集成神经网络G2P
- [ ] 支持自定义规则文件

### 长期目标 (3-6月)
- [ ] 端到端TTS集成
- [ ] Web Assembly支持
- [ ] 实时语音合成
- [ ] 声学模型集成

## 🤝 学习价值

这个项目特别适合学习：

### 自然语言处理
- 音韵学和语音学基础
- 规则驱动vs数据驱动方法
- 文本预处理和标准化
- 语言学特征建模

### Rust编程
- 高性能字符串处理
- 错误处理最佳实践
- 内存安全的算法实现
- 模块化架构设计

### 语音技术
- G2P在TTS中的作用
- 音素表示和编码
- 发音规则的复杂性
- 跨语言语音处理

## 📚 相关资源

### 论文和书籍
- [The CMU Pronouncing Dictionary](http://www.speech.cs.cmu.edu/cgi-bin/cmudict)
- [Speech and Language Processing](https://web.stanford.edu/~jurafsky/slp3/) - Jurafsky & Martin
- [Handbook of Speech Processing](https://www.springer.com/gp/book/9783540491255)

### 相关项目
- [espeak-ng](https://github.com/espeak-ng/espeak-ng) - 参考实现
- [Style TTS](https://github.com/yl4579/StyleTTS2) - 现代TTS框架

### 数据集
- [CMU Pronouncing Dictionary](http://www.speech.cs.cmu.edu/cgi-bin/cmudict)
- [LibriSpeech](https://www.openslr.org/12/) - 语音识别数据集
- [LJSpeech](https://keithito.com/LJ-Speech-Dataset/) - TTS数据集

## 📄 许可证

本项目采用MIT许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- CMU语音组提供的发音词典
- espeak-ng项目的设计启发
- Rust社区的优秀工具链
- 所有为语音技术做出贡献的研究者

## 📞 联系方式

如果你对这个项目有任何问题或建议，欢迎：
- 开Issue讨论
- 提交Pull Request
- 发邮件交流

---

*这是一个学习项目，用于理解TTS系统的核心原理。欢迎fork、学习和改进！* 🎓