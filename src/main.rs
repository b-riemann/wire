mod lib;
use lib::FunPyre;
use lib::tokenize_page;
use std::str;

fn main() {
    println!("Hello, world!");
    let fp = FunPyre::new("../workfiles/enwik9".to_string());
    let segments = fp.find_pages(300);

    let iv = fp.rfrom_segment( &segments.unwrap().getitem(7) );
    //print!("--ORIGINAL--\n{}\n", str::from_utf8(&iv).unwrap());

    let ev = tokenize_page(&iv);
    print!("--TOKENIZED--\n{}\n", str::from_utf8(&ev).unwrap());
}
