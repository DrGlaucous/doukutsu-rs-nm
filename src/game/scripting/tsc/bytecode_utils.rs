use std::io::{Cursor, Read};
use std::iter::Peekable;

use crate::framework::error::GameError::ParseError;
use crate::framework::error::GameResult;
use crate::game::scripting::tsc::text_script::TextScriptEncoding;
use crate::game::scripting::tsc::parse_utils::expect_char;

pub fn put_varint(val: i32, out: &mut Vec<u8>) {
    let mut x = ((val as u32) >> 31) ^ ((val as u32) << 1);

    loop {
        let mut n = (x & 0x7f) as u8;
        x >>= 7;

        if x != 0 {
            n |= 0x80;
        }

        out.push(n);

        if x == 0 {
            break;
        }
    }
}

pub fn read_cur_varint(cursor: &mut Cursor<&[u8]>) -> GameResult<i32> {
    let mut result = 0u32;

    for o in 0..5 {
        let mut n = [0u8];
        cursor.read_exact(&mut n)?;
        let [n] = n;

        result |= (n as u32 & 0x7f) << (o * 7);

        if n & 0x80 == 0 {
            break;
        }
    }

    Ok(((result << 31) ^ (result >> 1)) as i32)
}

#[allow(unused)]
pub fn read_varint<I: Iterator<Item = u8>>(iter: &mut I) -> GameResult<i32> {
    let mut result = 0u32;

    for o in 0..5 {
        let n = iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
        result |= (n as u32 & 0x7f) << (o * 7);

        if n & 0x80 == 0 {
            break;
        }
    }

    Ok(((result << 31) ^ (result >> 1)) as i32)
}


/// Used to grab event numbers and opcodes only! (NOT for actual TSC args!)
pub fn put_string(buffer: &mut Vec<u8>, out: &mut Vec<u8>, encoding: TextScriptEncoding) {
    if buffer.is_empty() {
        return;
    }
    let mut chars_count = 0;

    let mut tmp_buf = Vec::new();

    let encoding: &encoding_rs::Encoding = encoding.into();

    let decoded_text = encoding.decode_without_bom_handling(&buffer).0;
    for chr in decoded_text.chars() {
        chars_count += 1;
        put_varint(chr as _, &mut tmp_buf);
    }

    buffer.clear();

    put_varint(chars_count, out);
    out.append(&mut tmp_buf);
}

/// parses a string argument for TSC delimited by $:
/// 
/// `<EXPstring1$`
pub fn put_string_tsc<I: Iterator<Item=u8>>(
    iter: &mut Peekable<I>,
    out: &mut Vec<u8>,
) {

    //terminates with < or end of file

    let mut char_buf = Vec::with_capacity(64);
    while let Some(&chr) = iter.peek() {
        match chr
        {
            //End at '$' or '<' command (technically $ or < will work as a delimiter, but if the command ends early, the next one will be eatten by the parser)
            b'<' | b'$' => {
                iter.next();
                break;
            }
            //move reader forward
            b'\r' => {
                iter.next();
            }
            //add char to holding tank
            _ => {
                char_buf.push(chr);
                iter.next();
            }
        }
    }

    //don't use fancy encoding for now: the string goes directly into the compiled code
    put_varint(char_buf.len() as  i32, out); //put length of string arg
    out.append(&mut char_buf); //put string itself

}

/// parses n count of back-to-back string arguments for TSC, delimited by $:
/// 
/// `<EXPstring1$:string2$:string3$<END`
#[allow(unused)]
pub fn put_string_multi_tsc<I: Iterator<Item=u8>>(
    iter: &mut Peekable<I>,
    out: &mut Vec<u8>,
    count: usize,
    strict: bool
) -> GameResult {

    for i in 0..count {

        put_string_tsc(iter, out);

        //for all entries except the last one, check for the delimiter colon
        if i < count - 1 {
            if strict {
                expect_char(b':', iter)?;
            } else {
                iter.next().ok_or_else(|| ParseError("Script unexpectedly ended.".to_owned()))?;
            }
        }

    }

    Ok(())


}



/// Gets the next string argument for the TSC command (for use in conjunction with `parse_string_args`, NOT compatible with `put_string`)
pub fn read_string_tsc(cursor: &mut Cursor<&[u8]>, size: usize) -> GameResult<String> {

    //holds the string we get from the cursor
    let mut strvec: Vec<u8> = Vec::with_capacity(size);

    //shove all the bytes we get into the vector
    for p in cursor.bytes().take(size)
    {
        match p
        {
            Ok(a) =>
            {
                strvec.push(a)
            }
            Err(_) =>{
                return Err( ParseError(String::from("Problem reading string from TSC parser")) );
            }
        }
    };

    //turn that into a string
    let str_string = String::from_utf8(strvec).unwrap();

    Ok(str_string)
}

#[test]
fn test_varint() {
    for n in -4000..=4000 {
        let mut out = Vec::new();
        put_varint(n, &mut out);

        let result = read_varint(&mut out.iter().copied()).unwrap();
        assert_eq!(result, n);
        let mut cur: Cursor<&[u8]> = Cursor::new(&out);
        let result = read_cur_varint(&mut cur).unwrap();
        assert_eq!(result, n);
    }
}
