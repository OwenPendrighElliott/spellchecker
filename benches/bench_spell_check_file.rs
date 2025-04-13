use spellcheck::SpellCorrector;
use std::fs;
use std::time::Instant;

fn main() {
    let word_list_file = "words_100k.txt".to_string();
    let dict_spell_corrector_data_file = "benches/dict_spell_corrector_data.json".to_string();
    let text_file = "benches/test_text.txt".to_string();
    let max_edit_distance = 2;
    let max_suggestions = 5;

    let text_content = fs::read_to_string(text_file).expect("Unable to read text file, please ensure you have a file named test_text.txt in the current directory with any text contents.");

    let corrector_load_start_time = Instant::now();

    let spell_corrector;
    // check if the dict_spell_corrector_data_file exists
    if fs::metadata(&dict_spell_corrector_data_file).is_ok() {
        spell_corrector = SpellCorrector::load_spell_corrector(&dict_spell_corrector_data_file);
    } else {
        spell_corrector = SpellCorrector::from_word_list_file(&word_list_file, max_edit_distance);
        spell_corrector.save_spell_corrector(&dict_spell_corrector_data_file).expect("Unable to save the spell corrector data file, please check the file path and permissions.");
    }

    let corrector_load_elapsed_time = corrector_load_start_time.elapsed();
    println!(
        "Elapsed time for loading the spell corrector: {:?}",
        corrector_load_elapsed_time
    );

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
