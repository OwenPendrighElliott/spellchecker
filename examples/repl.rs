use spellcheck::{SpellCorrector, SuggestedCorrection};
use std::{
    env,
    io::{self, Write},
    path::Path,
};

const MAX_EDIT_DISTANCE: usize = 2;
const MAX_SUGGESTIONS: usize = 5;

fn main() -> io::Result<()> {
    let dict_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "words_100k.txt".into());

    if !Path::new(&dict_path).exists() {
        eprintln!("Dictionary file not found: {}", dict_path);
        std::process::exit(1);
    }

    let corrector = SpellCorrector::from_file(&dict_path, MAX_EDIT_DISTANCE);

    println!(
        "SymSpell REPL - dictionary: {}\n:type text, :q to quit",
        dict_path
    );
    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;
        input.clear();
        if io::stdin().read_line(&mut input)? == 0 {
            break; // EOF
        }
        if input.trim() == ":q" {
            break;
        }

        for token in input.split_whitespace() {
            let word = token
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if word.is_empty() {
                continue;
            }

            match corrector.suggest_single_word_corrections(&word, MAX_SUGGESTIONS) {
                SuggestedCorrection::NoSuggestions => {}
                SuggestedCorrection::Suggestions(list) => {
                    let suggestions: Vec<_> = list.into_iter().map(|s| s.word).collect();
                    println!("  {}  ->  {}", word, suggestions.join(", "));
                }
            }
        }
    }
    Ok(())
}
