mod lib;
use lib::FunPyre;
use lib::tokenize_page;
use std::str;

//fn scan_overview(fp: &FunPyre, first: usize) {
//    println!("--Scanning first {} pages for length--\npage  length", first);
//    for pagenum in 0..first {
//        let iv = fp.fetch_page(pagenum);
//        println!("  {}    {}", pagenum, iv.len());
//    }
//}

fn main() {
    let fp = FunPyre::new("../workfiles/enwik9".to_string(), 400);

    //scan_overview(&fp, 400);
    
    let pagenums = [2,7,16,19,113,177,267,347];

    for pagenum in pagenums {
        let iv = fp.fetch_page(pagenum);
        //print!("--Page {} (original)--\n{}\n", pagenum, str::from_utf8(&iv).unwrap());
        let mut ev = tokenize_page(&iv);
        ev.truncate(2500);
        println!("--Page {} (tokenized)--\n{}\n", pagenum, str::from_utf8(&ev).unwrap());
    }
}
