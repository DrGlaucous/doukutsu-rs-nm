use std::io::{Cursor, Read};

use crate::framework::error::GameError::ParseError;
use crate::framework::error::GameResult;
use crate::game::scripting::tsc::text_script::TextScriptEncoding;
use crate::util::encoding::{read_cur_shift_jis, read_cur_wtf8};

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
pub fn read_varint<I: Iterator<Item=u8>>(iter: &mut I) -> GameResult<i32> {
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

pub fn put_string(buffer: &mut Vec<u8>, out: &mut Vec<u8>, encoding: TextScriptEncoding) {
    if buffer.is_empty() {
        return;
    }

    let mut cursor: Cursor<&Vec<u8>> = Cursor::new(buffer);
    let mut tmp_buf = Vec::new();
    let mut remaining = buffer.len() as u32;
    let mut chars = 0;

    while remaining > 0 {
        let (consumed, chr) = match encoding {
            TextScriptEncoding::UTF8 => read_cur_wtf8(&mut cursor, remaining),
            TextScriptEncoding::ShiftJIS => read_cur_shift_jis(&mut cursor, remaining),
        };

        remaining -= consumed;
        chars += 1;

        put_varint(chr as i32, &mut tmp_buf);
    }

    buffer.clear();

    put_varint(chars, out);
    out.append(&mut tmp_buf);
}

pub fn read_string(cursor: &mut Cursor<&[u8]>, size: usize) -> GameResult<String> {

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
