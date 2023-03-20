use core::fmt;
use std::{array::TryFromSliceError, error::Error, str};

const MAX_ARGS: usize = 1024;

pub enum ResponseCode {
    Success,
    Error,
    Nonexistent,
}

// TODO: Do you want to thiserror macro lmao?
#[derive(Debug)]
pub struct TooManyArgsError;

impl Error for TooManyArgsError {}

impl fmt::Display for TooManyArgsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Too many args! Max args is {MAX_ARGS}")
    }
}

// TODO: Give extra data
#[derive(Debug)]
pub struct TrailingGarbageError;

impl Error for TrailingGarbageError {}

impl fmt::Display for TrailingGarbageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "There's trailing garbage! There's more bytes after parsing"
        )
    }
}

#[derive(Debug)]
pub struct ExcessiveDataError {
    last_position: usize,
    given_len: usize,
}

impl Error for ExcessiveDataError {}

impl fmt::Display for ExcessiveDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "There's too much data. The last position {} would overflow given len: {}",
            self.last_position, self.given_len
        )
    }
}

#[derive(Debug)]
pub struct InvalidCmd;

impl Error for InvalidCmd {}

impl fmt::Display for InvalidCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "You provided an invalid command")
    }
}

#[derive(Debug)]
pub enum ParseError {
    TryFromSliceError(TryFromSliceError),
    TooManyArgsError(TooManyArgsError),
    TrailingGarbageError(TrailingGarbageError),
    ExcessiveDataError(ExcessiveDataError),
    InvalidCmd(InvalidCmd),
}
impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::TryFromSliceError(e) => write!(f, "{e}"),
            ParseError::TooManyArgsError(e) => write!(f, "{e}"),
            ParseError::TrailingGarbageError(e) => write!(f, "{e}"),
            ParseError::ExcessiveDataError(e) => write!(f, "{e}"),
            ParseError::InvalidCmd(e) => write!(f, "{e}"),
        }
    }
}

impl From<TryFromSliceError> for ParseError {
    fn from(value: TryFromSliceError) -> Self {
        Self::TryFromSliceError(value)
    }
}

impl From<TooManyArgsError> for ParseError {
    fn from(value: TooManyArgsError) -> Self {
        Self::TooManyArgsError(value)
    }
}

impl From<TrailingGarbageError> for ParseError {
    fn from(value: TrailingGarbageError) -> Self {
        Self::TrailingGarbageError(value)
    }
}

impl From<ExcessiveDataError> for ParseError {
    fn from(value: ExcessiveDataError) -> Self {
        Self::ExcessiveDataError(value)
    }
}

impl From<InvalidCmd> for ParseError {
    fn from(value: InvalidCmd) -> Self {
        Self::InvalidCmd(value)
    }
}

pub struct GetCmd<'a> {
    pub key: &'a str,
}

pub struct SetCmd<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

pub struct DelCmd<'a> {
    pub key: &'a str,
}

pub enum Command<'a> {
    Get(GetCmd<'a>),
    Set(SetCmd<'a>),
    Del(DelCmd<'a>),
}

pub fn to_cmd(data: &[u8], total_len: usize) -> Result<Command, ParseError> {
    let mut parsed = vec![];
    parse_request(data, total_len, &mut parsed)?;
    
    let cmd = match (parsed.len(), parsed.get(0)) {
        (2, Some(&"get")) => Command::Get(GetCmd { key: parsed[1] }),
        (3, Some(&"set")) => Command::Set(SetCmd { key: parsed[1], value: parsed[2] }),
        (2, Some(&"del")) => Command::Del(DelCmd { key: parsed[1] }),
        _ => return Err(ParseError::InvalidCmd(InvalidCmd{})),
    };

    Ok(cmd)
}

/// Parse this:
/// +------+-----+------+-----+------+-----+-----+------+
/// | nstr | len | str1 | len | str2 | ... | len | strn |
/// +------+-----+------+-----+------+-----+-----+------+
///
/// nstr is u32 <-> 4 bytes
/// len is also u32 <-> 4 bytes
fn parse_request<'a, 'b>(
    data: &'a [u8],
    total_len: usize,
    out: &'b mut Vec<&'a str>,
) -> Result<(), ParseError> {
    const LEN_SIZE: usize = 4;

    let arr = data[..LEN_SIZE].try_into()?;
    let nstr = u32::from_le_bytes(arr);
    if nstr as usize > MAX_ARGS {
        return Err((TooManyArgsError {}).into());
    }

    let mut pos = LEN_SIZE;
    for _ in 0..nstr {
        let len = u32::from_le_bytes(data[pos..pos + LEN_SIZE].try_into()?) as usize;
        let end = pos + LEN_SIZE + len;
        if end > total_len {
            return Err((ExcessiveDataError {
                last_position: end,
                given_len: total_len,
            })
            .into());
        }
        pos += LEN_SIZE;
        let string = str::from_utf8(&data[pos..pos + len]).unwrap();
        out.push(string);
        pos += len;
    }

    if pos != total_len {
        return Err((TrailingGarbageError {}).into());
    }

    Ok(())
}
