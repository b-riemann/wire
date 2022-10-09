// for enwik9, available escape bytes are:
//fb fd c0 0f f2 1d 0e 1c 7f 1b 08 1f fc 04 1e df 00 05 0c 19 f9 f8 18 10 0b 16 1a fa f3 \r f6 f7 12 f4 17 14 01 15 f1 f5 11 dd 07 13 c1 06 02 03
// of which the ascii control characters (in range 00-1f) are 
// 00 01 02 03 04 05 06 07 08 \r(0a) 0b 0c 0e 0f 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f

// these escape bytes should not be part of the input file
// and preferably still be part of the ascii table (for easier utf8 output for conversion if required
pub const MACRO_ESC : u8 = b'\x05'; // x05 is nice, as its the enquiry symbol in ascii
pub const ANTISPACE : u8 = b'\x15';

