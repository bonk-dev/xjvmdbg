use binrw::{BinRead, BinWrite, meta::ReadEndian};
use std::io::{self, Cursor, Read, Write};

use crate::jdwp::{Command, CommandPacketHeader, IdSizesReply, ReplyPacketHeader, VersionReply};

pub struct JdwpClient<T: Read + Write> {
    packet_id: u32,
    stream: T,
}

impl<T: Read + Write> JdwpClient<T> {
    pub fn from(stream: T) -> Self {
        JdwpClient {
            packet_id: 0,
            stream: stream,
        }
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

    fn send_command_header(&mut self, command: Command, data_length: u32) -> io::Result<()> {
        let header = CommandPacketHeader {
            length: CommandPacketHeader::get_length() as u32 + data_length,
            id: self.packet_id,
            flags: 0,
            command: command,
        };
        let mut buffer = Vec::with_capacity(CommandPacketHeader::get_length());
        let mut cursor = Cursor::new(&mut buffer);
        header.write_be(&mut cursor).unwrap();
        self.stream.write_all(&buffer)
    }

    fn read_reply_header(&mut self) -> io::Result<ReplyPacketHeader> {
        let mut recv_buffer = vec![0u8; ReplyPacketHeader::get_length()];
        self.stream.read_exact(&mut recv_buffer).unwrap();
        let mut recv_cursor = Cursor::new(&mut recv_buffer);
        let reply_header = ReplyPacketHeader::read_be(&mut recv_cursor)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(reply_header)
    }

    fn send_bodyless<TReply: BinRead + ReadEndian>(&mut self, cmd: Command) -> io::Result<TReply>
    where
        for<'a> <TReply as BinRead>::Args<'a>: Default,
    {
        self.send_command_header(cmd, 0)?;
        let reply_header = self.read_reply_header()?;

        let mut buffer = vec![0u8; reply_header.length as usize - ReplyPacketHeader::get_length()];
        self.stream.read_exact(&mut buffer)?;
        let mut cursor = Cursor::new(&mut buffer);
        let reply =
            TReply::read(&mut cursor).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(reply)
    }

    pub fn vm_get_version(&mut self) -> io::Result<VersionReply> {
        self.send_bodyless(Command::VirtualMachineVersion)
    }

    pub fn vm_get_id_sizes(&mut self) -> io::Result<IdSizesReply> {
        self.send_bodyless(Command::VirtualMachineIDSizes)
    }
}
