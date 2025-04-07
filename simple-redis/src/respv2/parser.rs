use std::{collections::BTreeMap, num::NonZeroUsize};

use crate::{
    BulkString, NullBulkString, RespArray, RespError, RespFrame, RespMap, RespNull, RespNullArray,
    SimpleError, SimpleString,
};

use winnow::{
    Parser,
    ascii::{digit1, float},
    combinator::{alt, dispatch, fail, opt, preceded, terminated},
    error::ContextError,
    token::{any, take, take_until},
};

const CRLF: &[u8] = b"\r\n";

pub fn parse_frame_length(input: &[u8]) -> Result<usize, RespError> {
    let target = &mut (&*input);
    let ret = parse_frame_len(target);
    match ret {
        Ok(_) => {
            let start = input.as_ptr() as usize;
            let end = (*target).as_ptr() as usize;
            let len = end - start;
            Ok(len)
        }
        Err(_) => Err(RespError::NotComplete),
    }
}

pub fn parse_frame_len(input: &mut &[u8]) -> winnow::Result<()> {
    let mut simple_parser = terminated(take_until(0.., CRLF), CRLF).value(());

    dispatch! {any;
       b'+' => simple_parser,
       b'-' => simple_parser,
       b':' => simple_parser,
       b'$' => bulk_string_len,
       b'*' => array_len,
       b'_' => simple_parser,
       b',' => simple_parser,
       b'%' => map_len,
       b'#' => simple_parser,
       // b'~' => simple_strut),
       _v => fail::<_, _, _>
    }
    .parse_next(input)
}

pub fn parse_frame(input: &mut &[u8]) -> winnow::Result<RespFrame> {
    dispatch! {any;
        b'+' => simple_string.map(RespFrame::SimpleString),
        b'-' => error.map(RespFrame::Error),
        b':' => integer.map(RespFrame::Integer),
        b'$' => alt((null_bulk_string.map(RespFrame::NullBulkString), bulk_string.map(RespFrame::BulkString))),
        b'*' => alt((null_array.map(RespFrame::NullArray), array.map(RespFrame::Array))),
        b'_' => null.map(RespFrame::Null),
        b',' => double.map(RespFrame::Double),
        b'%' => map.map(RespFrame::Map),
        b'#' => boolean.map(RespFrame::Boolean),
        // b'~' => simple_strut),
        _v => fail::<_, _, _>
    }.parse_next(input)
}

fn parse_string(input: &mut &[u8]) -> winnow::Result<String> {
    terminated(take_until(0.., CRLF), CRLF)
        .map(|s: &[u8]| String::from_utf8_lossy(s).into_owned())
        .parse_next(input)
}

fn simple_string(input: &mut &[u8]) -> winnow::Result<SimpleString> {
    parse_string.map(SimpleString).parse_next(input)
}

fn error(input: &mut &[u8]) -> winnow::Result<SimpleError> {
    parse_string.map(SimpleError).parse_next(input)
}

fn integer(input: &mut &[u8]) -> winnow::Result<i64> {
    let sign = opt(alt(('+', '-'))).parse_next(input)?.unwrap_or('+');
    let sign = if sign == '+' { 1 } else { -1 };
    let v: i64 = terminated(digit1.parse_to(), CRLF).parse_next(input)?;

    Ok(sign * v)
}

fn null_bulk_string(input: &mut &[u8]) -> winnow::Result<NullBulkString> {
    "-1\r\n".value(NullBulkString).parse_next(input)
}

#[allow(clippy::comparison_chain)]
fn bulk_string(input: &mut &[u8]) -> winnow::Result<BulkString> {
    let len = integer.parse_next(input)?;
    if len == 0 {
        return Ok(BulkString::new(vec![]));
    } else if len < 0 {
        return Err(err_cut("bulk string length must be non-negative"));
    }

    let data = terminated(take(len as usize), CRLF)
        .map(|s: &[u8]| s.to_vec())
        .parse_next(input)?;
    Ok(BulkString(data))
}

fn bulk_string_len(input: &mut &[u8]) -> winnow::Result<()> {
    let len = integer.parse_next(input)?;
    if len == 0 || len == -1 {
        return Ok(());
    } else if len < -1 {
        return Err(err_cut("bulk String length must be non-megatice"));
    }

    let len_with_crlf = len as usize + 2;
    if input.len() < len_with_crlf {
        let _size = NonZeroUsize::new((len_with_crlf - input.len()) as usize).unwrap();
        return Err(err_cut("needed NonZeroSize"));
    }
    *input = &input[(len + 2) as usize..];
    Ok(())
}

fn null_array(input: &mut &[u8]) -> winnow::Result<RespNullArray> {
    "-1\r\n".value(RespNullArray).parse_next(input)
}

#[allow(clippy::comparison_chain)]
fn array(input: &mut &[u8]) -> winnow::Result<RespArray> {
    let len = integer.parse_next(input)?;
    if len == 0 {
        return Ok(RespArray(vec![]));
    } else if len < 0 {
        return Err(err_cut("array length must be non-negative"));
    }
    let mut arr = Vec::with_capacity(len as usize);
    for _ in 0..len {
        arr.push(parse_frame(input)?);
    }
    Ok(RespArray(arr))
}

fn array_len(input: &mut &[u8]) -> winnow::Result<()> {
    let len = integer.parse_next(input)?;
    if len == 0 || len == -1 {
        return Ok(());
    } else if len < -1 {
        return Err(err_cut("array length must be non-negative"));
    }
    for _ in 0..len {
        parse_frame(input)?;
    }
    Ok(())
}

fn boolean(input: &mut &[u8]) -> winnow::Result<bool> {
    let b = alt(('t', 'f')).parse_next(input)?;
    Ok(b == 't')
}

fn double(input: &mut &[u8]) -> winnow::Result<f64> {
    terminated(float, CRLF).parse_next(input)
}

fn map(input: &mut &[u8]) -> winnow::Result<RespMap> {
    let len = integer.parse_next(input)?;
    if len <= 0 {
        return Err(err_cut("map length must be non-negative"));
    }
    let mut map = BTreeMap::new();
    for _ in 0..len {
        let key = preceded('+', parse_string).parse_next(input)?;
        let value = parse_frame(input)?;
        map.insert(key, value);
    }
    Ok(RespMap(map))
}

fn map_len(input: &mut &[u8]) -> winnow::Result<()> {
    let len = integer.parse_next(input)?;
    if len <= 0 {
        return Err(err_cut("map length must be non-negative"));
    }
    for _ in 0..len {
        terminated(take_until(0.., CRLF), CRLF)
            .value(())
            .parse_next(input)?;
        parse_frame_len(input)?;
    }
    Ok(())
}

fn null(input: &mut &[u8]) -> winnow::Result<RespNull> {
    CRLF.value(RespNull).parse_next(input)
}

fn err_cut(_s: impl Into<String>) -> ContextError {
    ContextError::default()
}
