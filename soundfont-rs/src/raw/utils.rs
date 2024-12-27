use crate::error::ParseError;
use std::convert::TryInto;

pub(crate) struct Reader<'a> {
    data: &'a [u8],
    curr: usize,
}
impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, curr: 0 }
    }

    #[allow(dead_code)]
    pub fn read(&mut self, len: usize) -> &[u8] {
        let start = self.curr;
        let end = self.curr + len;
        self.curr = end;

        &self.data[start..end]
    }

    pub fn read_string(&mut self, len: usize) -> Result<String, ParseError> {
        let start = self.curr;
        let end = self.curr + len;
        self.curr = end;

        let data = &self.data[start..end];

        let data = if let Some(end) = data.iter().position(|v| *v == 0) {
            &data[..end]
        } else {
            data
        };

        // According to the spec, strings have to be in ASCII.
        // But obviously this is SF2 world, spec is just a suggestion, people use non ASCII characters.
        // So let's just use � for those characters.
        let name = String::from_utf8_lossy(data).to_string();

        Ok(name)
    }

    pub fn read_u8(&mut self) -> Result<u8, ParseError> {
        let start = self.curr;
        let end = self.curr + 1;
        self.curr = end;

        let out: [u8; 1] = self.data[start..end].try_into().map_err(ParseError::from)?;
        Ok(u8::from_le_bytes(out))
    }

    pub fn read_u16(&mut self) -> Result<u16, ParseError> {
        let start = self.curr;
        let end = self.curr + 2;
        self.curr = end;

        let out: [u8; 2] = self.data[start..end].try_into().map_err(ParseError::from)?;
        Ok(u16::from_le_bytes(out))
    }

    pub fn read_u32(&mut self) -> Result<u32, ParseError> {
        let start = self.curr;
        let end = self.curr + 4;
        self.curr = end;

        let out: [u8; 4] = self.data[start..end].try_into().map_err(ParseError::from)?;
        Ok(u32::from_le_bytes(out))
    }

    pub fn read_i8(&mut self) -> Result<i8, ParseError> {
        let start = self.curr;
        let end = self.curr + 1;
        self.curr = end;

        let out: [u8; 1] = self.data[start..end].try_into().map_err(ParseError::from)?;
        Ok(i8::from_le_bytes(out))
    }

    pub fn read_i16(&mut self) -> Result<i16, ParseError> {
        let start = self.curr;
        let end = self.curr + 2;
        self.curr = end;

        let out: [u8; 2] = self.data[start..end].try_into().map_err(ParseError::from)?;
        Ok(i16::from_le_bytes(out))
    }
}
