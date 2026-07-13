use serde_json::Value;
use std::fmt;
use std::io::{self, BufRead, Write};

#[derive(Debug)]
pub enum ProtocolError {
    Io(io::Error),
    Json(serde_json::Error),
    MissingContentLength,
    InvalidContentLength,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Json(error) => write!(formatter, "invalid JSON-RPC payload: {error}"),
            Self::MissingContentLength => formatter.write_str("missing Content-Length header"),
            Self::InvalidContentLength => formatter.write_str("invalid Content-Length header"),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<io::Error> for ProtocolError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ProtocolError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, ProtocolError> {
    let mut content_length = None;
    let mut header = String::new();

    loop {
        header.clear();
        let read = reader.read_line(&mut header)?;
        if read == 0 {
            return if content_length.is_none() {
                Ok(None)
            } else {
                Err(ProtocolError::MissingContentLength)
            };
        }
        if header == "\r\n" || header == "\n" {
            break;
        }
        if let Some(value) = header
            .strip_prefix("Content-Length:")
            .or_else(|| header.strip_prefix("content-length:"))
        {
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| ProtocolError::InvalidContentLength)?,
            );
        }
    }

    let content_length = content_length.ok_or(ProtocolError::MissingContentLength)?;
    let mut payload = vec![0; content_length];
    reader.read_exact(&mut payload)?;
    Ok(Some(serde_json::from_slice(&payload)?))
}

pub fn write_message(writer: &mut impl Write, message: &Value) -> Result<(), ProtocolError> {
    let payload = serde_json::to_vec(message)?;
    write!(writer, "Content-Length: {}\r\n\r\n", payload.len())?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn round_trips_a_framed_message() {
        let message = serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": null});
        let mut bytes = Vec::new();
        write_message(&mut bytes, &message).unwrap();

        assert_eq!(
            read_message(&mut Cursor::new(bytes)).unwrap(),
            Some(message)
        );
    }
}
