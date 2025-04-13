use criterion::{Criterion, criterion_group, criterion_main};
use spellcheck::SpellCorrector;

fn bench_spell_check_string(c: &mut Criterion) {
    let dict_file = "words_100k.txt".to_string();
    let max_edit_distance = 2;
    let max_suggestions = 5;

    let text_content = "This is a short sntence with some misspelled wrds. It is used for testing the spell checker functionality.".to_string();

    let spell_corrector = SpellCorrector::from_word_list_file(&dict_file, max_edit_distance);

    // split the text content into words using white spa
    let words: Vec<String> = text_content
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();

    c.bench_function("spell_check_words", |b| {
        b.iter(|| {
            let _ = spell_corrector.suggest_word_corrections(&words, max_suggestions);
        })
    });
}

criterion_group!(benches, bench_spell_check_string);
criterion_main!(benches);
