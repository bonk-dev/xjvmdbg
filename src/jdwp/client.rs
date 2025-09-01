use binrw::{BinRead, BinResult, BinWrite, Endian, binrw};
use std::io::{self, Read, Write};

pub struct JdwpClient<T: Read + Write> {
    stream: T,
}

impl<T: Read + Write> JdwpClient<T> {
    pub fn from(stream: T) -> Self {
        JdwpClient { stream: stream }
    }

    pub fn do_handshake(&mut self) -> io::Result<()> {
        const HANDSHAKE_STR: &str = "JDWP-Handshake";

        let handshake_bytes = HANDSHAKE_STR.as_bytes();
        self.stream.write_all(handshake_bytes)?;
        self.stream.flush()?;

        let mut buffer = [0u8; 14];
        self.stream.read_exact(&mut buffer)?;

        let received = std::str::from_utf8(&buffer)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        if received != HANDSHAKE_STR {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid handshake: expected '{}', got '{}'",
                    HANDSHAKE_STR, received
                ),
            ));
        }

        Ok(())
    }
}
