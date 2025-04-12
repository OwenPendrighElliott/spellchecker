use spellcheck::SpellCorrector;
use std::fs;
use std::time::Instant;

fn main() {
    let dict_file = "words_100k.txt".to_string();
    let text_file = "benches/test_text.txt".to_string();
    let max_edit_distance = 3;
    let max_suggestions = 5;

    let text_content = fs::read_to_string(text_file).expect("Unable to read text file, please ensure you have a file named test_text.txt in the current directory with any text contents.");

    let spell_corrector = SpellCorrector::from_file(&dict_file, max_edit_distance);

    // split the text content into words using white spa
    let words: Vec<String> = text_content
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();

    let start_time = Instant::now();

    let _ = spell_corrector.suggest_word_corrections(&words, max_suggestions);

    let elapsed_time = start_time.elapsed();
    println!("Elapsed time for spell checking: {:?}", elapsed_time);
    println!(
        "Words corrected per second: {}",
        words.len() as f64 / elapsed_time.as_secs_f64()
    );
}
