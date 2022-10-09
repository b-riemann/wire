use std::collections::HashMap;

#[path="constants.rs"]
mod constants;
use constants::MACRO_ESC;

enum WordClass{
    English,
    Unicode,
    Macro
}

fn classify_word(word: &[u8]) -> WordClass {
    for ch in word {
        match ch {
            0x41..=0x5a => continue, //uppercase
            0x61..=0x7a => continue, //lowercase
            &MACRO_ESC => return WordClass::Macro,
            _ => return WordClass::Unicode
        }         
    }
    return WordClass::English
}


struct WordDict {
    hm: HashMap<Vec<u8>, usize>,
    nonsingular_entries: usize,
    counted_words: usize
}

impl WordDict {
    fn new() -> Self {
        WordDict {
            hm: HashMap::new(),
            nonsingular_entries: 0,
            counted_words: 0
        }
    }

    fn add(&mut self, word: &[u8]) {
        let wv = word.to_vec();
        match self.hm.get_mut(&wv) {
            Some(v) => {let n = *v + 1; *v = n; if n==2 { self.nonsingular_entries += 1; }},
            None => { self.hm.insert(wv, 1); }
        }
        self.counted_words += 1;
    }

    fn display(&self) {
        let n_total = self.hm.len();
        println!("nonsingular {} vs total {} ({:.3}), counted words: {}",
            self.nonsingular_entries, n_total, self.nonsingular_entries as f32 / n_total as f32, self.counted_words);
        for (key, val) in self.hm.iter().take(150) {
            print!(":{}: {} ", String::from_utf8_lossy(&key).to_owned(), val);
        }
    }

}




enum AggrState {
    Normal,
    //Uppercase,
    Macro
}

pub struct WordAggregator {
    english: WordDict,
    unicode: WordDict,
    state: AggrState
}

impl WordAggregator {
    pub fn new() -> Self {
        WordAggregator {
            english: WordDict::new(),
            unicode: WordDict::new(),
            state: AggrState::Normal
        }
    }
 
    pub fn count_text(&mut self, iv: &Vec<u8>) {
        let mut a = 0;
        for (b, ch) in iv.into_iter().enumerate() {
            if ch!=&b' ' {
                continue; //splitting words at spaces b' '
            }
            let word = &iv[a..b]; //.to_vec();
            a = b+1;

            match self.state {
                AggrState::Normal => {
                    match classify_word(word) {
                        WordClass::English => self.english.add(word),
                        WordClass::Unicode => self.unicode.add(word),
                        WordClass::Macro => self.state = AggrState::Macro,
                    }
                },
                AggrState::Macro => {
                    match classify_word(word) {
                        WordClass::Macro => self.state = AggrState::Normal,
                        _ => (), // macro parsing here
                    }
                }
            }
        }
    }

    pub fn display(&self) {
        print!("english>> "); self.english.display();
        print!("\n\nunicode>> "); self.unicode.display();
    }
}


