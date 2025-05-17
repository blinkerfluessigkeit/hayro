mod charstring;
mod charstring_parser;
mod decrypt;
mod operator;
mod standard;
pub(crate) mod stream;

use crate::type1::charstring::parse_char_string;
use crate::type1::decrypt::{decrypt, decrypt_byte};
use crate::type1::standard::STANDARD;
use crate::type1::stream::Stream;
use crate::{Matrix, OutlineBuilder};
use log::error;
use std::collections::HashMap;
use std::iter::Copied;
use std::slice::Iter;
use std::str::FromStr;
use std::sync::Arc;
// Many parts of the parser code are adapted from
// https://github.com/janpe2/CFFDump/blob/master/cff/type1/Type1Dump.java

#[derive(Debug)]
pub(crate) struct Parameters {
    font_matrix: Matrix,
    encoding_type: EncodingType,
    subroutines: HashMap<u32, Vec<u8>>,
    charstrings: HashMap<String, Vec<u8>>,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            font_matrix: Matrix::default(),
            encoding_type: EncodingType::Standard,
            subroutines: HashMap::new(),
            charstrings: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table<'a> {
    data: &'a [u8],
    params: Arc<Parameters>,
}

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if !data.starts_with(b"%!") {
            error!("type1 font didn't start with %!");

            return None;
        }

        let mut s = Stream::new(data);
        let mut params = Parameters::default();

        while let Some(token) = s.next_token() {
            match token {
                b"/FontInfo" => s.skip_dict(),
                b"/FontName" => s.skip_token(),
                b"/PaintType" => s.skip_token(),
                b"/FontType" => s.skip_token(),
                b"/FontBBox" => s.skip_token(),
                b"/UniqueID" => s.skip_token(),
                b"/Metrics" => s.skip_dict(),
                b"/StrokeWidth" => s.skip_token(),
                b"/FontMatrix" => {
                    let matrix = s.read_font_matrix();
                    params.font_matrix = Matrix {
                        sx: matrix[0],
                        kx: matrix[1],
                        ky: matrix[2],
                        sy: matrix[3],
                        tx: matrix[4],
                        ty: matrix[5],
                    };
                }
                b"/Encoding" => params.encoding_type = s.read_encoding(),
                b"eexec" => {
                    let decrypted = decrypt(s.tail().unwrap());
                    Self::parse_eexec(&decrypted, &mut params);
                }
                b"/Private" => {
                    println!("reached private dict");
                }
                _ => {}
            }
        }

        Some(Self {
            data,
            params: Arc::new(params),
        })
    }

    fn parse_eexec(data: &[u8], params: &mut Parameters) {
        let mut s = Stream::new(data);

        let mut lenIv = 4;

        while let Some(token) = s.next_token() {
            match token {
                b"/Subrs" => {
                    params.subroutines = s.parse_subroutines(lenIv);
                }
                b"/CharStrings" => {
                    params.charstrings = s.parse_charstrings(lenIv).unwrap();
                }
                b"/lenIV" => {
                    lenIv = s.next_int() as usize;
                }
                _ => {}
            }
        }
    }

    /// Returns a font transformation matrix.
    #[inline]
    pub fn matrix(&self) -> Matrix {
        self.params.font_matrix
    }

    /// Outlines a glyph.
    pub fn outline(&self, string: &str, builder: &mut dyn OutlineBuilder) -> Option<()> {
        let data = self.params.charstrings.get(string)?;

        parse_char_string(data, &self.params, builder).unwrap();

        Some(())
    }

    pub fn code_to_string(&self, code_point: u8) -> Option<&str> {
        self.params.encoding_type.encode(code_point)
    }
}

const ND: &[u8] = b"ND";
const ND_ALT: &[u8] = b"|-";

const RD: &[u8] = b"RD";
const RD_ALT: &[u8] = b"-|";

const NP: &[u8] = b"NP";
const NP_ALT: &[u8] = b"|";

impl<'a> Stream<'a> {
    fn next_int(&mut self) -> i32 {
        i32::from_str(std::str::from_utf8(self.next_token().unwrap()).unwrap()).unwrap()
    }

    fn parse_charstrings(&mut self, len_iv: usize) -> Option<HashMap<String, Vec<u8>>> {
        let mut charstrings = HashMap::new();

        let mut first_glyph_name = None;
        let mut int_token = None;

        while let Some(token) = self.next_token() {
            if token == b"end" {
                return Some(charstrings);
            }

            if token.starts_with(b"/") {
                first_glyph_name = Some(token);
            } else if token
                .iter()
                .all(|b| matches!(*b, b'#') || b.is_ascii_digit())
            {
                int_token = Some(i32::from_str(std::str::from_utf8(token).unwrap()).unwrap());
            } else if token == RD || token == RD_ALT {
                break;
            }
        }

        let (first_glyph_name, int_token) = (first_glyph_name.unwrap(), int_token.unwrap());

        let mut is_first = true;

        loop {
            let bin_len;
            let mut glyph_name;

            if is_first {
                is_first = false;
                bin_len = int_token;
                glyph_name = first_glyph_name;

                if glyph_name.starts_with(b"/") {
                    glyph_name = &glyph_name[1..];
                }

                self.read_byte();
            } else {
                let tok = self.next_token().unwrap();
                if tok == b"end" {
                    break;
                }

                if tok.starts_with(b"/") {
                    glyph_name = &tok[1..];
                } else {
                    glyph_name = tok;
                }

                bin_len = self.next_int();
                let tok = self.next_token().unwrap();

                if tok == RD || tok == RD_ALT {
                    self.read_byte();
                } else {
                    panic!("invalid charstring in start, expected RD");
                }
            }

            let encrypted_bytes = self.read_bytes(bin_len as usize).unwrap();
            let decrypted_bytes = decrypt_charstring(encrypted_bytes, len_iv);
            charstrings.insert(
                std::str::from_utf8(glyph_name).unwrap().to_string(),
                decrypted_bytes,
            );

            let tok = self.next_token().unwrap();
            if tok == ND || tok == ND_ALT {
            } else {
                panic!("invalid charstring in end, expected ND, found {:?}", tok);
            }
        }

        Some(charstrings)
    }

    fn parse_subroutines(&mut self, len_iv: usize) -> HashMap<u32, Vec<u8>> {
        let mut subroutines = HashMap::new();

        let num_subrs =
            u32::from_str(std::str::from_utf8(self.next_token().unwrap()).unwrap()).unwrap();

        if num_subrs < 1 {
            return subroutines;
        }

        if !self.skip_until_before(b"dup", |b| matches!(b, ND | ND_ALT | b"noaccess")) {
            return subroutines;
        }

        while let Some(token) = self.next_token() {
            if matches!(token, ND | ND_ALT) {
                break;
            }

            if token == b"noaccess" {
                if self.next_token() == Some(b"def") {
                    break;
                } else {
                    panic!("invallid sequence noaccess");
                }
            }

            if token != b"dup" {
                panic!("expected dup, got token {:?} instead", &token);
            }

            let subr_idx = self.next_int();
            let bin_len = self.next_int();

            let tok = self.next_token().unwrap();

            if tok != RD && tok != RD_ALT {
                panic!("invalid subroutine start token {:?}", tok);
            } else {
                // WHitespace
                self.read_byte();
            }

            // TODO: Decrypt
            let encrypted_bytes = self.read_bytes(bin_len as usize).unwrap();
            subroutines.insert(subr_idx as u32, decrypt_charstring(encrypted_bytes, len_iv));

            let mut tok = self.next_token().unwrap();
            if tok == NP || tok == NP_ALT {
            } else if tok == b"noaccess" {
                tok = self.next_token().unwrap();
                if tok == b"def" {
                    break;
                }

                if tok == b"put" {
                } else {
                    panic!("invallid subroutine end {:?}", tok);
                }
            } else {
                panic!("invalid subroutine end token {:?}", tok);
            }
        }

        subroutines
    }

    fn peek_token(&mut self) -> Option<&'a [u8]> {
        self.clone().next_token()
    }

    fn next_token(&mut self) -> Option<&'a [u8]> {
        let skip_token = |st: &mut Stream| -> usize {
            let mut count = 1;
            while let Some(ch) = st.read_bytes(1) {
                if is_whitespace(ch[0]) || is_self_delim_after_token(ch[0]) {
                    st.move_back(1);
                    break;
                }

                count += 1;
            }

            count
        };

        self.skip_whitespaces();

        while let Some(ch) = self.clone().read_bytes(1) {
            let tail = self.tail()?;
            self.read_bytes(1);

            match ch[0] {
                b'%' => self.skip_line_comment(),
                b'(' => return Some(b"("),
                b'<' => {
                    if let Some(ch2) = self.read_bytes(1) {
                        if ch2[0] == b'>' {
                            return Some(b"( )");
                        } else if ch2[0] == b'<' {
                            return Some(b"<<");
                        } else {
                            return Some(b"<");
                        }
                    }
                }
                b'>' => {
                    if let Some(ch2) = self.read_bytes(1) {
                        if ch2[0] == b'>' {
                            return Some(b">>");
                        } else {
                            self.move_back(1);
                            return Some(b">");
                        }
                    }
                }
                b'[' => {
                    return Some(b"[");
                }
                b']' => {
                    return Some(b"]");
                }
                b'{' => {
                    return Some(b"{");
                }
                b'}' => {
                    return Some(b"}");
                }
                b'/' => {
                    if let Some(ch2) = self.read_bytes(1) {
                        if is_whitespace(ch2[0]) || is_self_delim_after_token(ch2[0]) {
                            let token = b"/";

                            if is_self_delim_after_token(ch2[0]) {
                                self.move_back(1);
                            }

                            return Some(token);
                        } else {
                            let count = skip_token(self);

                            return Some(&tail[0..(count + 1)]);
                        }
                    }
                }
                _ => {
                    let count = skip_token(self);
                    return Some(&tail[0..count]);
                }
            }

            self.skip_whitespaces();
        }

        None
    }

    fn read_font_matrix(&mut self) -> [f32; 6] {
        let mut entries = [0.0f32; 6];
        let mut idx = 0;

        // Skip '[';
        self.skip_token();

        while let Some(token) = self.next_token() {
            entries[idx] = f32::from_str(std::str::from_utf8(token).unwrap()).unwrap();

            idx += 1;
            if idx == 5 {
                break;
            }
        }

        // Skip `]`.
        self.skip_token();

        entries
    }

    fn read_encoding(&mut self) -> EncodingType {
        let mut map = HashMap::new();

        let t1 = self.next_token().unwrap();
        let t2 = self.next_token().unwrap();

        if t1 == b"StandardEncoding" && t2 == b"def" {
            return EncodingType::Standard;
        }

        if !self.skip_until_before(b"dup", |b| matches!(b, b"def" | b"readonly")) {
            return EncodingType::Custom(Arc::new(map));
        }

        while let Some(token) = self.next_token() {
            if matches!(token, b"def" | b"readonly") {
                break;
            }

            if token != b"dup" {
                panic!("Unexpected token {:?}", token);
            }

            let code =
                u8::from_str(std::str::from_utf8(self.next_token().unwrap()).unwrap()).unwrap();
            let glyph_name = std::str::from_utf8(&self.next_token().unwrap()[1..])
                .unwrap()
                .to_string();

            if self.next_token().unwrap() != b"put" {
                panic!("Unexpected token {:?}", token);
            }

            map.insert(code, glyph_name);
        }

        EncodingType::Custom(Arc::new(map))
    }

    fn skip_dict(&mut self) {
        self.skip_until(b"begin", |b| matches!(b, b"end"));
    }

    fn skip_token(&mut self) {
        self.next_token();
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.read_byte() {
            if matches!(ch, b'\n' | b'\r') {
                break;
            }
        }
    }

    fn skip_until(&mut self, find: &[u8], stop: impl Fn(&[u8]) -> bool) -> bool {
        while let Some(token) = self.next_token() {
            if token == find {
                return true;
            }

            if stop(token) {
                break;
            }
        }

        false
    }

    fn skip_whitespaces(&mut self) {
        while let Some(ch) = self.peek_byte() {
            if is_whitespace(ch) {
                self.read_byte();
            } else {
                break;
            }
        }
    }

    fn skip_until_before(&mut self, find: &[u8], stop: impl Fn(&[u8]) -> bool) -> bool {
        while let Some(token) = self.peek_token() {
            if token == find {
                return true;
            }

            self.next_token().unwrap();

            if stop(token) {
                break;
            }
        }

        false
    }
}

fn decrypt_charstring(data: &[u8], len_iv: usize) -> Vec<u8> {
    let mut r = 4330;
    let mut cb: Copied<Iter<u8>> = data.iter().copied();
    let mut decrypted = vec![];

    for i in 0..len_iv {
        let _ = decrypt_byte(cb.next().unwrap(), &mut r);
    }

    for byte in cb {
        decrypted.push(decrypt_byte(byte, &mut r))
    }

    decrypted
}

fn is_whitespace(c: u8) -> bool {
    if c <= 32 {
        return matches!(c, b' ' | b'\n' | b'\r' | b'\t' | 0x00 | 0x0C);
    }

    false
}

fn is_self_delim_after_token(c: u8) -> bool {
    // The characters ()<>[]{}/% are special. They delimit syntactic entities
    // such as strings, procedure bodies, name literals, and comments. Any of these
    // characters terminates the entity preceding it and is not included in the entity.

    matches!(
        c,
        b'(' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%' | b')'
    )

    // This checks for self delimiters appearing after tokens. Thus there is no
    // need to check for ')'. However, char '>' can appear in keyword >>, like
    // here: /Pages 2 0 R>>. So the char '>' must end the token R.
}

#[derive(Debug, Clone)]
pub(crate) enum EncodingType {
    Standard,
    Custom(Arc<HashMap<u8, String>>),
}

impl EncodingType {
    pub(crate) fn encode(&self, code: u8) -> Option<&str> {
        match self {
            EncodingType::Standard => STANDARD.get(&code).copied(),
            EncodingType::Custom(c) => c.get(&code).map(|s| s.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::type1::stream::Stream;

    macro_rules! assert_token {
        ($content:expr, $token:expr) => {
            assert_eq!($content.next_token(), Some(&$token[..]))
        };
    }

    #[test]
    fn lexing_1() {
        let mut content = Stream::new(b"/FontInfo ");

        assert_token!(content, b"/FontInfo");
    }

    #[test]
    fn lexing_2() {
        let mut content = Stream::new(b"/version (01) readonly def");

        assert_token!(content, b"/version");
        assert_token!(content, b"(");
        assert_token!(content, b"01");
        assert_token!(content, b")");
        assert_token!(content, b"readonly");
        assert_token!(content, b"def");
    }
}
