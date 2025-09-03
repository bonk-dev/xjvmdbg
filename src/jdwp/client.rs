use binrw::{BinRead, BinWrite, meta::ReadEndian};
use std::io::{self, Cursor};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::jdwp::{
    Command, CommandPacketHeader, IdSizesReply, ReplyPacketHeader, VersionReply, result,
};

pub struct JdwpClient<T: AsyncRead + AsyncWrite> {
    packet_id: u32,
    stream: T,
}

impl<T: AsyncRead + AsyncWrite + Unpin> JdwpClient<T> {
    pub fn from(stream: T) -> Self {
        JdwpClient {
            packet_id: 0,
            stream: stream,
        }
    }

    pub async fn do_handshake(&mut self) -> result::Result<()> {
        const HANDSHAKE_STR: &str = "JDWP-Handshake";

        let handshake_bytes = HANDSHAKE_STR.as_bytes();
        self.stream.write_all(handshake_bytes).await?;
        self.stream.flush().await?;

        let mut buffer = [0u8; 14];
        self.stream.read_exact(&mut buffer).await?;

        let received = std::str::from_utf8(&buffer)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        if received != HANDSHAKE_STR {
            return Err(result::Error::ParsingError {
                message: format!(
                    "Invalid handshake: expected '{}', got '{}'",
                    HANDSHAKE_STR, received
                ),
            });
        }

        Ok(())
    }

    async fn send_command_header(
        &mut self,
        command: Command,
        data_length: u32,
    ) -> result::Result<()> {
        let header = CommandPacketHeader {
            length: CommandPacketHeader::get_length() as u32 + data_length,
            id: self.packet_id,
            flags: 0,
            command: command,
        };
        let mut buffer = Vec::with_capacity(CommandPacketHeader::get_length());
        let mut cursor = Cursor::new(&mut buffer);
        header.write_be(&mut cursor).unwrap();
        self.stream.write_all(&buffer).await?;
        Ok(())
    }

    async fn read_reply_header(&mut self) -> result::Result<ReplyPacketHeader> {
        let mut recv_buffer = vec![0u8; ReplyPacketHeader::get_length()];
        self.stream.read_exact(&mut recv_buffer).await?;
        let mut recv_cursor = Cursor::new(&mut recv_buffer);
        let reply_header = ReplyPacketHeader::read_be(&mut recv_cursor)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(reply_header)
    }

    async fn send_bodyless<TReply: BinRead + ReadEndian>(
        &mut self,
        cmd: Command,
    ) -> result::Result<TReply>
    where
        for<'a> <TReply as BinRead>::Args<'a>: Default,
    {
        self.send_command_header(cmd, 0).await?;
        let reply_header = self.read_reply_header().await?;

        let mut buffer = vec![0u8; reply_header.length as usize - ReplyPacketHeader::get_length()];
        self.stream.read_exact(&mut buffer).await?;
        let mut cursor = Cursor::new(&mut buffer);
        let reply =
            TReply::read(&mut cursor).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(reply)
    }

    pub async fn vm_get_version(&mut self) -> result::Result<VersionReply> {
        self.send_bodyless(Command::VirtualMachineVersion).await
    }

    pub async fn vm_get_id_sizes(&mut self) -> result::Result<IdSizesReply> {
        self.send_bodyless(Command::VirtualMachineIDSizes).await
    }
}
