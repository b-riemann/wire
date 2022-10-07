mod lib;
use lib::FunPyre;
use std::collections::HashMap;
use std::str;

use progress_bar::*;

//fn scan_overview(fp: &FunPyre, first: usize) {
//    println!("--Scanning first {} pages for length--\npage  length", first);
//    for pagenum in 0..first {
//        let iv = fp.fetch_page(pagenum);
//        println!("  {}    {}", pagenum, iv.len());
//    }
//}

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

struct WordAggregator {
    english: WordDict, //used only for lowercase english (conversion)
    unicode: WordDict
}

fn is_english(word: &[u8]) -> bool {
    for ch in word {
        match ch {
            0x41..=0x5a => continue, //uppercase
            0x61..=0x7a => continue, //lowercase
            _ => return false  
        }         
    }
    true
}

impl WordAggregator {
    fn new() -> Self {
        WordAggregator { english: WordDict::new(), unicode: WordDict::new() }
    }
 
    fn count_text(&mut self, iv: &Vec<u8>) {
        // superbly stupid approach for splitting:
        let mut a = 0;
        for (b, ch) in iv.into_iter().enumerate() {
            if ch==&b' ' {
               let word = &iv[a..b]; //.to_vec();
               a = b+1;

               if is_english(word) {
                   self.english.add(word); continue;
               }
               self.unicode.add(word);
            }
        }
    }

    fn display(&self) {
        print!("english>> "); self.english.display();
        print!("\n\nunicode>> "); self.unicode.display();
    }
}


fn main() {
    let mode = "run";

    let first_n_pages = 1024;
    let fp = FunPyre::new("../workfiles/enwik9".to_string(), first_n_pages);

    let mut wc = WordAggregator::new();

    if mode == "run" {
        init_progress_bar(first_n_pages-1);
        set_progress_bar_action("Reading", Color::Green, Style::Bold);

        for pagenum in 1..first_n_pages {
            let iv = fp.fetch_page(pagenum);
            //print!("--Page {} (original)--\n{}\n", pagenum, str::from_utf8(&iv).unwrap());
            let ev = fp.pagere.rex(&iv);
            wc.count_text(&ev);
            inc_progress_bar();
        }
        wc.display();
    } else {
        //scan_overview(&fp, 350);
        let pagenums = [2,7,16,19,113,124,177,267,347];
        for pagenum in pagenums {
            let iv = fp.fetch_page(pagenum);
            let ev = fp.pagere.rex(&iv);
            println!("--Page {} (rexed)--\n{}\n", pagenum, str::from_utf8(&ev).unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canbe_pagerexed() {
        let n_pages = 5000;
        let fp = FunPyre::new("../workfiles/enwik9".to_string(), n_pages);
        let iv = fp.fetch_page(0);
        assert_eq!(fp.pagere.matches(&iv), false);
        for pagenum in 1..n_pages {
            let iv = fp.fetch_page(pagenum);
            assert!(fp.pagere.matches(&iv));
        }
    }


}
