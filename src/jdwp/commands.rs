use binrw::binrw;

use crate::{binrw_enum, jdwp::JdwpString};

binrw_enum! {
    #[repr(u16)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Command {
        VirtualMachineVersion = (1 << 8) | 1,
        VirtualMachineIDSizes = (1 << 8) | 7,
    }
}

#[binrw]
#[brw(big)]
pub struct CommandPacketHeader {
    pub length: u32,
    pub id: u32,
    pub flags: u8,
    pub command: Command,
}
impl CommandPacketHeader {
    pub fn get_length() -> usize {
        return 4 + 4 + 1 + 2;
    }
}

#[binrw]
#[brw(big)]
pub struct ReplyPacketHeader {
    pub length: u32,
    pub id: u32,
    pub flags: u8,
    pub error_code: u16,
}
impl ReplyPacketHeader {
    pub fn default() -> Self {
        ReplyPacketHeader {
            length: 0,
            id: 0xFFFFFFFF,
            flags: 0,
            error_code: 0,
        }
    }
    pub fn get_length() -> usize {
        return 4 + 4 + 1 + 2;
    }
    pub fn is_success(&self) -> bool {
        return self.error_code == 0;
    }
}

#[derive(Debug)]
pub struct VariableLengthId {
    pub value: u64,
}
impl BinRead for VariableLengthId {
    type Args<'a> = JdwpIdSize;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        // TODO: Support non-power-of-2 sizes if needed
        let val: u64 = match args {
            1 => u8::read_options(reader, endian, ())? as u64,
            2 => u16::read_options(reader, endian, ())? as u64,
            4 => u32::read_options(reader, endian, ())? as u64,
            8 => u64::read_options(reader, endian, ())?,
            _ => {
                return binrw::BinResult::Err(binrw::Error::Custom {
                    pos: reader.stream_position().unwrap_or(0),
                    err: Box::new("Unsupported variable size ID"),
                });
            }
        };

        Ok(VariableLengthId { value: val })
    }
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct VersionReply {
    pub description: JdwpString,
    pub jdwp_major: i32,
    pub jdwp_minor: i32,
    pub vm_version: JdwpString,
    pub vm_name: JdwpString,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct IdSizesReply {
    pub field_id_size: i32,
    pub method_id_size: i32,
    pub object_id_size: i32,
    pub reference_type_id_size: i32,
    pub frame_id_size: i32,
}

#[cfg(test)]
mod tests {
    use crate::jdwp::Command;
    use binrw::BinRead;
    use std::io::Cursor;

    #[test]
    fn test_deserialize_vm_version_command() {
        let data = [1u8, 1u8];
        let mut cursor = Cursor::new(&data);
        let value = Command::read_be(&mut cursor).unwrap();
        assert_eq!(value, Command::VirtualMachineVersion);
    }
}
