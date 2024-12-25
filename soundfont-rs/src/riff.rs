//! Utilities for reading RIFF-formatted files

// (Based on `riff` MIT crate: Copyright 2018 Francesco Bertolaccini)

use std::{
    convert::TryInto,
    fmt,
    io::{Read, Seek, SeekFrom},
};

/// A chunk id, also known as FourCC
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct ChunkId([u8; 4]);

macro_rules! def_ids {
    (
        $(
            $(#[doc = $doc:expr])?
            $ident: ident
        ),*
        // trailing comma
        $(,)?
    ) => {
        $(
            $(#[doc = $doc])?
            #[allow(non_upper_case_globals)]
            pub const $ident: Self = Self({
                let v = stringify!($ident).as_bytes();
                [v[0], v[1], v[2], v[3]]
            });
        )*
    };
}

impl ChunkId {
    // 3.1 General RIFF File Structure
    def_ids![RIFF, LIST];

    // 4.1 SoundFont 2 RIFF File Format Level 0

    def_ids![
        /// RIFF form header
        sfbk,
    ];

    // RIFF(sfbk)
    def_ids![
        /// Supplemental Information
        INFO,
        /// The Sample Binary Data
        sdta,
        /// The Preset, Instrument, and Sample Header data
        pdta,
    ];

    // 4.2 SoundFont 2 RIFF File Format Level 1

    // List(INFO)
    def_ids![
        /// Refers to the version of the Sound Font RIFF file
        ifil,
        /// Refers to the target Sound Engine
        isng,
        /// Refers to the Sound Font Bank Name
        INAM,
        /// Refers to the Sound ROM Name
        irom,
        /// Refers to the Sound ROM Version
        iver,
        /// Refers to the Date of Creation of the Bank
        ICRD,
        /// Sound Designers and Engineers for the Bank
        IENG,
        /// Product for which the Bank was intended
        IPRD,
        /// Contains any Copyright message
        ICOP,
        /// Contains any Comments on the Bank
        ICMT,
        /// The SoundFont tools used to create and alter the bank
        ISFT,
    ];

    // List(sdta)
    def_ids![
        /// The Digital Audio Samples for the upper 16 bits
        smpl,
        /// The Digital Audio Samples for the lower 8 bits
        sm24,
    ];

    // List(pdta)
    def_ids![
        /// The Preset Headers
        phdr,
        /// The Preset Index list
        pbag,
        /// The Preset Modulator list
        pmod,
        /// The Preset Generator list
        pgen,
        /// The Instrument Names and Indices
        inst,
        /// The Instrument Index list
        ibag,
        /// The Instrument Modulator list
        imod,
        /// The Instrument Generator list
        igen,
        /// The Sample Headers
        shdr,
    ];
}

impl fmt::Debug for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        if let Ok(v) = std::str::from_utf8(&self.0) {
            write!(f, "{v:?}")
        } else {
            write!(f, "{:?}", self.0)
        }
    }
}

/// A chunk, also known as a form
#[derive(PartialEq, Eq, Debug)]
pub struct Chunk {
    pos: u64,
    id: ChunkId,
    len: u32,
}

/// An iterator over the children of a `Chunk`
pub struct Iter<'a, T>
where
    T: Seek + Read,
{
    end: u64,
    cur: u64,
    // TODO: Make stream a `next` function argument
    stream: &'a mut T,
}

impl<T> Iterator for Iter<'_, T>
where
    T: Seek + Read,
{
    type Item = std::io::Result<Chunk>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.end {
            return None;
        }

        let chunk = match Chunk::read(&mut self.stream, self.cur) {
            Ok(chunk) => chunk,
            Err(err) => return Some(Err(err)),
        };
        let len = chunk.len() as u64;
        self.cur = self.cur + len + 8 + (len % 2);
        Some(Ok(chunk))
    }
}

impl Chunk {
    /// Returns the `ChunkId` of this chunk.
    pub fn id(&self) -> ChunkId {
        self.id
    }

    /// Returns the number of bytes in this chunk.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns the offset of this chunk from the start of the stream.
    pub fn offset(&self) -> u64 {
        self.pos
    }

    /// Reads the chunk type of this chunk.
    ///
    /// Generally only valid for `RIFF` and `LIST` chunks.
    pub fn read_type<T>(&self, stream: &mut T) -> std::io::Result<ChunkId>
    where
        T: Read + Seek,
    {
        stream.seek(SeekFrom::Start(self.pos + 8))?;

        let mut fourcc: [u8; 4] = [0; 4];
        stream.read_exact(&mut fourcc)?;

        Ok(ChunkId(fourcc))
    }

    /// Reads a chunk from the specified position in the stream.
    pub fn read<T>(stream: &mut T, pos: u64) -> std::io::Result<Chunk>
    where
        T: Read + Seek,
    {
        stream.seek(SeekFrom::Start(pos))?;

        let mut fourcc: [u8; 4] = [0; 4];
        stream.read_exact(&mut fourcc)?;

        let mut len: [u8; 4] = [0; 4];
        stream.read_exact(&mut len)?;

        Ok(Chunk {
            pos,
            id: ChunkId(fourcc),
            len: u32::from_le_bytes(len),
        })
    }

    /// Reads the entirety of the contents of a chunk.
    pub fn read_contents<T>(&self, stream: &mut T) -> std::io::Result<Vec<u8>>
    where
        T: Read + Seek,
    {
        stream.seek(SeekFrom::Start(self.pos + 8))?;

        let mut data: Vec<u8> = vec![0; self.len.try_into().unwrap()];
        stream.read_exact(&mut data)?;

        Ok(data)
    }

    /// Returns an iterator over the children of the chunk.
    pub fn iter<'a, T>(&self, stream: &'a mut T) -> Iter<'a, T>
    where
        T: Seek + Read,
    {
        Iter {
            cur: self.pos + 12,
            end: self.pos + 4 + (self.len as u64),
            stream,
        }
    }
}
