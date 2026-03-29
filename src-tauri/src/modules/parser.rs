/// Bencode Parser Module
///
/// Implements parsing of Bencode format used in torrent files.
/// Bencode is a simple encoding format consisting of:
/// - Integers: i<number>e (e.g., i42e)
/// - Strings: <length>:<string> (e.g., 4:spam)
/// - Lists: l<items>e (e.g., l4:spami42ee)
/// - Dictionaries: d<key><value>...e (e.g., d3:key5:valuee)

use bytes::Bytes;
use std::collections::BTreeMap;
use std::str;

/// Represents a Bencoded value
#[derive(Debug, Clone, PartialEq)]
pub enum BencodeValue {
    /// String value (raw bytes for non-UTF8 safety)
    String(Bytes),
    /// Integer value
    Integer(i64),
    /// List of values
    List(Vec<BencodeValue>),
    /// Dictionary of string keys to values (ordered map)
    Dict(BTreeMap<String, BencodeValue>),
}

impl BencodeValue {
    /// Convert to string (UTF-8 safe)
    pub fn as_str(&self) -> Option<&str> {
        match self {
            BencodeValue::String(b) => str::from_utf8(b).ok(),
            _ => None,
        }
    }

    /// Convert to bytes
    pub fn as_bytes(&self) -> Option<&Bytes> {
        match self {
            BencodeValue::String(b) => Some(b),
            _ => None,
        }
    }

    /// Convert to integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            BencodeValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert to list
    pub fn as_list(&self) -> Option<&Vec<BencodeValue>> {
        match self {
            BencodeValue::List(l) => Some(l),
            _ => None,
        }
    }

    /// Convert to dictionary
    pub fn as_dict(&self) -> Option<&BTreeMap<String, BencodeValue>> {
        match self {
            BencodeValue::Dict(d) => Some(d),
            _ => None,
        }
    }

    /// Get value from dictionary by key
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&BencodeValue> {
        self.as_dict().and_then(|d| d.get(key))
    }

    /// Get string value from dictionary by key
    #[allow(dead_code)]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    /// Get integer value from dictionary by key
    #[allow(dead_code)]
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_int())
    }

    /// Get list value from dictionary by key
    #[allow(dead_code)]
    pub fn get_list(&self, key: &str) -> Option<&Vec<BencodeValue>> {
        self.get(key).and_then(|v| v.as_list())
    }
}

/// Bencode parser
pub struct BencodeParser;

impl BencodeParser {
    /// Parse bencoded data
    pub fn parse(data: &[u8]) -> Result<BencodeValue, ParseError> {
        let mut parser = Parser::new(data);
        parser.parse_value()
    }

    /// Encode a value to bencode
    pub fn encode(value: &BencodeValue) -> Vec<u8> {
        match value {
            BencodeValue::String(s) => {
                let mut result = format!("{}:", s.len()).into_bytes();
                result.extend_from_slice(s);
                result
            }
            BencodeValue::Integer(i) => format!("i{}e", i).into_bytes(),
            BencodeValue::List(l) => {
                let mut result = b"l".to_vec();
                for item in l {
                    result.extend(Self::encode(item));
                }
                result.push(b'e');
                result
            }
            BencodeValue::Dict(d) => {
                let mut result = b"d".to_vec();
                for (k, v) in d.iter() {
                    // Keys must be strings
                    result.extend(format!("{}:", k.len()).as_bytes());
                    result.extend(k.as_bytes());
                    result.extend(Self::encode(v));
                }
                result.push(b'e');
                result
            }
        }
    }
}

/// Parsing errors
#[derive(Debug)]
pub enum ParseError {
    /// Unexpected end of data
    UnexpectedEof,
    /// Invalid format
    InvalidFormat(String),
    /// Invalid character
    InvalidCharacter(char),
    /// Non-UTF8 dictionary key
    InvalidDictKey,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "Unexpected end of data"),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::InvalidCharacter(c) => write!(f, "Invalid character: {}", c),
            ParseError::InvalidDictKey => write!(f, "Dictionary keys must be UTF-8 strings"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Internal parser state
struct Parser {
    data: Vec<u8>,
    pos: usize,
}

impl Parser {
    fn new(data: &[u8]) -> Self {
        Parser {
            data: data.to_vec(),
            pos: 0,
        }
    }

    fn parse_value(&mut self) -> Result<BencodeValue, ParseError> {
        match self.peek()? {
            b'i' => self.parse_int(),
            b'd' => self.parse_dict(),
            b'l' => self.parse_list(),
            b'0'..=b'9' => self.parse_string(),
            c => Err(ParseError::InvalidCharacter(c as char)),
        }
    }

    fn parse_int(&mut self) -> Result<BencodeValue, ParseError> {
        self.expect(b'i')?;
        let start = self.pos;

        // Handle negative sign
        if self.peek_byte() == Some(b'-') {
            self.pos += 1;
        }

        // Read digits
        while let Some(b'0'..=b'9') = self.peek_byte() {
            self.pos += 1;
        }

        let end_pos = self.pos;
        let num_str = str::from_utf8(&self.data[start..end_pos]).map_err(|_| {
            ParseError::InvalidFormat("Invalid UTF-8 in integer".to_string())
        })?;

        let value: i64 = num_str.parse().map_err(|_| {
            ParseError::InvalidFormat("Invalid integer value".to_string())
        })?;

        self.expect(b'e')?;

        Ok(BencodeValue::Integer(value))
    }

    fn parse_string(&mut self) -> Result<BencodeValue, ParseError> {
        let start = self.pos;

        // Read length
        while let Some(b'0'..=b'9') = self.peek_byte() {
            self.pos += 1;
        }

        let len_str = str::from_utf8(&self.data[start..self.pos]).map_err(|_| {
            ParseError::InvalidFormat("Invalid UTF-8 in string length".to_string())
        })?;

        let len: usize = len_str.parse().map_err(|_| {
            ParseError::InvalidFormat("Invalid string length".to_string())
        })?;

        self.expect(b':')?;

        let string_start = self.pos;
        self.pos += len;

        if self.pos > self.data.len() {
            return Err(ParseError::UnexpectedEof);
        }

        Ok(BencodeValue::String(Bytes::copy_from_slice(
            &self.data[string_start..self.pos],
        )))
    }

    fn parse_list(&mut self) -> Result<BencodeValue, ParseError> {
        self.expect(b'l')?;
        let mut list = Vec::new();

        while self.peek()? != b'e' {
            list.push(self.parse_value()?);
        }

        self.expect(b'e')?;
        Ok(BencodeValue::List(list))
    }

    fn parse_dict(&mut self) -> Result<BencodeValue, ParseError> {
        self.expect(b'd')?;
        let mut dict = BTreeMap::new();

        while self.peek()? != b'e' {
            // Keys must be strings
            let key_value = self.parse_string()?;
            let key = key_value
                .as_str()
                .ok_or(ParseError::InvalidDictKey)?
                .to_string();

            // Parse value
            let value = self.parse_value()?;

            dict.insert(key, value);
        }

        self.expect(b'e')?;
        Ok(BencodeValue::Dict(dict))
    }

    fn peek(&self) -> Result<u8, ParseError> {
        self.peek_byte()
            .ok_or(ParseError::UnexpectedEof)
    }

    fn peek_byte(&self) -> Option<u8> {
        if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), ParseError> {
        let actual = self.peek()?;
        if actual != expected {
            return Err(ParseError::InvalidFormat(format!(
                "Expected '{}', found '{}'",
                expected as char, actual as char
            )));
        }
        self.pos += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let data = b"i42e";
        let result = BencodeParser::parse(data).unwrap();
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_parse_negative_integer() {
        let data = b"i-42e";
        let result = BencodeParser::parse(data).unwrap();
        assert_eq!(result.as_int(), Some(-42));
    }

    #[test]
    fn test_parse_string() {
        let data = b"4:spam";
        let result = BencodeParser::parse(data).unwrap();
        assert_eq!(result.as_str(), Some("spam"));
    }

    #[test]
    fn test_parse_list() {
        let data = b"l4:spami42ee";
        let result = BencodeParser::parse(data).unwrap();
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].as_str(), Some("spam"));
        assert_eq!(list[1].as_int(), Some(42));
    }

    #[test]
    fn test_parse_dict() {
        let data = b"d3:key5:valuee";
        let result = BencodeParser::parse(data).unwrap();
        assert_eq!(result.get_str("key"), Some("value"));
    }

    #[test]
    fn test_encode_integer() {
        let value = BencodeValue::Integer(42);
        let encoded = BencodeParser::encode(&value);
        assert_eq!(encoded, b"i42e");
    }

    #[test]
    fn test_encode_string() {
        let value = BencodeValue::String(Bytes::from_static(b"hello"));
        let encoded = BencodeParser::encode(&value);
        assert_eq!(encoded, b"5:hello");
    }

    #[test]
    fn test_roundtrip() {
        let original = b"d4:name5:hello5:valuei42ee";
        let parsed = BencodeParser::parse(original).unwrap();
        let encoded = BencodeParser::encode(&parsed);
        assert_eq!(encoded, original);
    }
}
