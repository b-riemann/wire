pub const MACRO_ESC : u8 = b'\x05'; // x05 is nice, as its the enquiry symbol in ascii
pub const ANTISPACE : u8 = b'\x15';

mod lib;
mod aggregator;

use progress_bar::*;
use std::env;

fn main() {
    let mut args = env::args();
    args.next();
    if args.len() != 1 {
        println!("specify argument (readword, pageview, pagelen). Exiting.");
        return;
    }

    let first_n_pages = 1024;
    let fp = lib::FunPyre::new("../workfiles/enwik9".to_string(), first_n_pages+1);

    let mut wc = aggregator::WordAggregator::new();

    match args.next().unwrap().as_str() {
        "readword" => {
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
        },
        "pageview" => {
            for pagenum in [2,7,16,19,113,124,177,267,347] {
                let iv = fp.fetch_page(pagenum);
                let ev = fp.pagere.rex(&iv);
                println!("--Page {} (rexed)--\n{}\n", pagenum, String::from_utf8_lossy(&ev));
            }
        },
        "pagelen" => {
            let first = 150;
            println!("--Scanning first {} pages for length--\npage  length", first);
            for pagenum in 0..first {
                let iv = fp.fetch_page(pagenum);
                println!("  {}    {}", pagenum, iv.len());
            }
        }
        a => println!("unknown argument: {}", a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canbe_pagerexed() {
        let n_pages = 5000;
        let fp = lib::FunPyre::new("../workfiles/enwik9".to_string(), n_pages);
        let iv = fp.fetch_page(0);
        assert_eq!(fp.pagere.matches(&iv), false);
        for pagenum in 1..n_pages {
            let iv = fp.fetch_page(pagenum);
            assert!(fp.pagere.matches(&iv));
        }
    }


}
