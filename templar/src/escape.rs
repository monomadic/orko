
use std::string::FromUtf8Error;

pub fn escape_html(raw_str:&str) -> Result<String, FromUtf8Error> {
    let mut allocated = Vec::with_capacity(raw_str.len() * 2); // this is allocation free
    for c in raw_str.as_bytes() {
        match *c {
            b'&' | b'<' | b'>' | b'"' | b'\'' | b'/' | b'`' => {
                match *c {
                    b'&' => allocated.extend_from_slice(b"&amp;"),
                    b'<' => allocated.extend_from_slice(b"&lt;"),
                    b'>' => allocated.extend_from_slice(b"&gt;"),
                    b'"' => allocated.extend_from_slice(b"&quot;"),
                    b'\'' => allocated.extend_from_slice(b"&#x27;"),
                    b'/' => allocated.extend_from_slice(b"&#x2F;"),
                    // Old versions of IE treat a ` as a '.
                    b'`' => allocated.extend_from_slice(b"&#96;"),
                    _ => unreachable!()
                }
            }
            _ => allocated.push(*c),
        }
    }

    String::from_utf8(allocated)
}

pub fn escape_default(raw_str:&str) -> String {
    let mut out : Vec<char> = Vec::new();
    for c in raw_str.chars() {
        for ec in c.escape_default() {
            out.push(ec);
        }
    }

    out.into_iter().collect()

}