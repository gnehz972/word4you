use word4you::utils::{classify_input, Language, InputType};

fn main() {
    let test_cases = vec![
        // English examples
        "hello",
        "break the ice", 
        "The early bird catches the worm.",
        "This is a longer sentence with many words to demonstrate sentence classification.",
        
        // Chinese examples
        "好",
        "你好",
        "打破僵局",
        "早起的鸟儿有虫吃。",
        "这是一个很长的中文句子，用来演示句子分类功能的效果。",
        
        // Mixed examples
        "Hello 你好",
        "API接口",
        "I love 中国菜 very much!",
    ];
    
    println!("=== Word4You Input Classification Demo ===\n");
    
    for input in test_cases {
        let classification = classify_input(input);
        
        let lang_str = match classification.language {
            Language::English => "English",
            Language::Chinese => "Chinese",
            Language::Mixed => "Mixed",
        };
        
        let type_str = match classification.input_type {
            InputType::Word => "Word",
            InputType::Phrase => "Phrase", 
            InputType::Sentence => "Sentence",
        };
        
        println!("Input: \"{}\"", input);
        println!("  Language: {}", lang_str);
        println!("  Type: {}", type_str);
        println!();
    }
}