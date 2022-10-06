mod lib;
use lib::FunPyre;
use std::str;
use std::collections::HashMap;

//fn scan_overview(fp: &FunPyre, first: usize) {
//    println!("--Scanning first {} pages for length--\npage  length", first);
//    for pagenum in 0..first {
//        let iv = fp.fetch_page(pagenum);
//        println!("  {}    {}", pagenum, iv.len());
//    }
//}

struct WordCounter {
    hm: HashMap<Vec<u8>, usize>
}

impl WordCounter {
    fn new() -> Self {
        WordCounter { hm: HashMap::new() }
    }

    fn count_text(&mut self, iv: &Vec<u8>) {
        // superbly stupid approach:
        let mut a = 0;
        for (b, ch) in iv.into_iter().enumerate() {
            if ch==&b' ' {
               let word = iv[a..b].to_vec();
               a = b;
               match self.hm.get_mut(&word) {
                   Some(v) => {let n = *v + 1; *v = n;},
                   None => { self.hm.insert(word.to_vec(), 1); }
               }
            }
        }
    }

    fn display(&self) {
        for (key, val) in &self.hm {
            println!("{}: {}", String::from_utf8_lossy(&key).to_owned(), val);
        }
    }
}


fn main() {
    let fp = FunPyre::new("../workfiles/enwik9".to_string(), 350);

    //scan_overview(&fp, 350);
    
    let pagenums = [2,7,16,19,113,124,177,267,347];

    let mut wc = WordCounter::new();

    for pagenum in pagenums {
        let iv = fp.fetch_page(pagenum);
        //print!("--Page {} (original)--\n{}\n", pagenum, str::from_utf8(&iv).unwrap());
        let mut ev = fp.pagere.rex(&iv);
        ev.truncate(2500);
        println!("--Page {} (rexed)--\n{}\n", pagenum, str::from_utf8(&ev).unwrap());

        wc.count_text(&ev);
    }
    wc.display();
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
