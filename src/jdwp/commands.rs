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

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct VersionReply {
    description: JdwpString,
    jdwp_major: i32,
    jdwp_minor: i32,
    vm_version: JdwpString,
    vm_name: JdwpString,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct IdSizesReply {
    field_id_size: i32,
    method_id_size: i32,
    object_id_size: i32,
    reference_type_id_size: i32,
    frame_id_size: i32,
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
