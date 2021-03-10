use crate::error::ParseError;
use std::convert::TryInto;

pub struct Reader {
    data: Vec<u8>,
    curr: usize,
}
impl Reader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, curr: 0 }
    }

    pub fn read(&mut self, len: usize) -> &[u8] {
        let start = self.curr;
        let end = self.curr + len;
        self.curr = end;

        let out = &self.data[start..end];
        out
    }

    pub fn read_string(&mut self, len: usize) -> Result<String, ParseError> {
        let start = self.curr;
        let end = self.curr + len;
        self.curr = end;

        let mut data = self.data[start..end].to_vec();
        // Null terminate it, just in case...
        data.push(0x0);

        let name = unsafe { std::ffi::CStr::from_ptr(data.as_ptr() as _) };

        let name = name.to_str().map_err(ParseError::from)?;
        let name = name.to_owned();

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
