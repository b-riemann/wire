mod lib;
use lib::WordAggregator;
use lib::FunPyre;
use std::str;

use progress_bar::*;

//fn scan_overview(fp: &FunPyre, first: usize) {
//    println!("--Scanning first {} pages for length--\npage  length", first);
//    for pagenum in 0..first {
//        let iv = fp.fetch_page(pagenum);
//        println!("  {}    {}", pagenum, iv.len());
//    }
//}


fn main() {
    let mode = "run";

    let first_n_pages = 1024;
    let fp = FunPyre::new("../workfiles/enwik9".to_string(), first_n_pages+1);

    let mut wc = WordAggregator::new();

    if mode == "run" {
        init_progress_bar(first_n_pages);
        set_progress_bar_action("Reading", Color::Green, Style::Bold);
        // idea for later: "Reading" "..skipped as file xy exists"

        for pagenum in 1..first_n_pages+1 {
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
