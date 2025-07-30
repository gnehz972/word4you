use crate::utils::{InputClassification, InputType, Language};

pub struct PromptTemplates;

impl PromptTemplates {
    pub fn get_template(classification: &InputClassification) -> String {
        match (&classification.language, &classification.input_type) {
            (Language::English, InputType::Word) => Self::english_word_template(),
            (Language::English, InputType::Phrase) => Self::english_phrase_template(),
            (Language::English, InputType::Sentence) => Self::english_sentence_template(),
            (Language::Chinese, InputType::Word) => Self::chinese_word_template(),
            (Language::Chinese, InputType::Phrase) => Self::chinese_phrase_template(),
            (Language::Chinese, InputType::Sentence) => Self::chinese_sentence_template(),
            (Language::Mixed, InputType::Word) => Self::mixed_word_template(),
            (Language::Mixed, InputType::Phrase) => Self::mixed_phrase_template(),
            (Language::Mixed, InputType::Sentence) => Self::mixed_sentence_template(),
        }
    }

    fn english_word_template() -> String {
        r#"
**Role:** You are a bilingual dictionary assistant providing structured explanations for English words.

**Output Structure:**
## [WORD]

*/Phonetics (IPA)/*

> Definition in English

**Chinese Translation**

- Example sentence in English
- 中文例句翻译

*Usage tip or note in English*

**Example:**
## resilience

*/rɪˈzɪliəns/*

> The capacity to recover quickly from difficulties; toughness.

**韧性；恢复力**

- Her resilience helped her overcome the crisis.
- 她的韧性帮助她度过了危机。

*Often describes emotional or physical toughness in challenging situations.*

Please provide the structured explanation for: [INSERT TEXT HERE]
"#.to_string()
    }

    fn english_phrase_template() -> String {
        r#"
**Role:** You are a bilingual dictionary assistant providing structured explanations for English phrases.

**Output Structure:**
## [PHRASE]

> Meaning and usage in English

**Chinese Translation**

- Example sentence using the phrase in English
- 中文例句翻译

*Usage context or cultural note*

**Example:**
## break the ice

> To initiate conversation in a social setting; to overcome initial awkwardness.

**打破僵局；破冰**

- He told a joke to break the ice at the meeting.
- 他讲了个笑话来打破会议上的僵局。

*Commonly used in social and business contexts to describe starting conversations.*

Please provide the structured explanation for: [INSERT TEXT HERE]
"#.to_string()
    }

    fn english_sentence_template() -> String {
        r#"
**Role:** You are a translation assistant providing accurate Chinese translations for English sentences.

**Output Structure:**
## [ORIGINAL SENTENCE]

**[CHINESE TRANSLATION]**

**Example:**
## The early bird catches the worm.

**早起的鸟儿有虫吃。**

Please provide the translation for: [INSERT TEXT HERE]
"#.to_string()
    }

    fn chinese_word_template() -> String {
        r#"
**Role:** You are a bilingual dictionary assistant providing structured explanations for Chinese characters/words.

**Output Structure:**
## [CHINESE CHARACTER/WORD]

*/Pinyin with tones/*

> Definition and meaning in English

**English Translation**

- 中文例句
- English example sentence translation

*Usage note or cultural context*

**Example:**
## 韧性

*/rèn xìng/*

> The quality of being resilient; the ability to recover from difficulties.

**Resilience; Toughness**

- 她的韧性帮助她度过了困难时期。
- Her resilience helped her get through the difficult period.

*Often used to describe mental or emotional strength in facing challenges.*

Please provide the structured explanation for: [INSERT TEXT HERE]
"#.to_string()
    }

    fn chinese_phrase_template() -> String {
        r#"
**Role:** You are a bilingual dictionary assistant providing structured explanations for Chinese phrases.

**Output Structure:**
## [CHINESE PHRASE]

*/Pinyin with tones/*

> Meaning and usage in English

**English Translation**

- 中文例句使用这个短语
- English translation of the example sentence

*Cultural context or usage note*

**Example:**
## 打破僵局

*/dǎ pò jiāng jú/*

> To break a deadlock or awkward situation; to initiate progress in a stalled situation.

**Break the deadlock; Break the ice**

- 他的幽默帮助打破了会议上的僵局。
- His humor helped break the deadlock in the meeting.

*Commonly used in business and social contexts to describe overcoming obstacles.*

Please provide the structured explanation for: [INSERT TEXT HERE]
"#.to_string()
    }

    fn chinese_sentence_template() -> String {
        r#"
**Role:** You are a translation assistant providing accurate English translations for Chinese sentences.

**Output Structure:**
## [ORIGINAL CHINESE SENTENCE]

**[ENGLISH TRANSLATION]**

**Example:**
## 早起的鸟儿有虫吃。

**The early bird catches the worm.**

Please provide the translation for: [INSERT TEXT HERE]
"#.to_string()
    }

    fn mixed_word_template() -> String {
        r#"
**Role:** You are a bilingual dictionary assistant providing explanations for mixed-language words or terms.

**Output Structure:**
## [MIXED TERM]

*/Pronunciation guide/*

> Definition and context in English

**Translation**

- Example sentence showing usage
- 翻译例句

*Note about mixed-language usage*

Please provide the structured explanation for: [INSERT TEXT HERE]
"#.to_string()
    }

    fn mixed_phrase_template() -> String {
        r#"
**Role:** You are a bilingual dictionary assistant providing explanations for mixed-language phrases.

**Output Structure:**
## [MIXED PHRASE]

> Meaning and context

**Translation**

- Example sentence in context
- 翻译例句

*Note about code-switching or mixed usage*

Please provide the structured explanation for: [INSERT TEXT HERE]
"#.to_string()
    }

    fn mixed_sentence_template() -> String {
        r#"
**Role:** You are a translation assistant for mixed-language sentences.

**Output Structure:**
## [ORIGINAL MIXED SENTENCE]

**[TRANSLATION TO DOMINANT LANGUAGE]**

Please provide the translation for: [INSERT TEXT HERE]
"#
        .to_string()
    }
}
