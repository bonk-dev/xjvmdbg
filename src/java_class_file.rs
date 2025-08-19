use binrw::{BinRead, BinResult, BinWrite, Endian, binrw};
use bitflags::bitflags;
use byteorder::ReadBytesExt;
use std::io::{Read, Seek, Write};
use std::u16;

const JAVA_CLASS_FILE_MAGIC: u32 = 0xCAFEBABE;

#[binrw]
#[derive(Debug, Clone, Copy)]
pub struct Version {
    magic: u32,
    minor: u16,
    major: u16,
}
impl Version {
    pub fn default() -> Self {
        Self {
            magic: JAVA_CLASS_FILE_MAGIC,
            major: 52,
            minor: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[binrw]
pub struct ClassAccessFlags(u16);

#[derive(Debug, Clone, Copy)]
#[binrw]
pub struct FieldAccessFlags(u16);

#[derive(Debug, Clone, Copy)]
#[binrw]
pub struct MethodAccessFlags(u16);

bitflags! {
    impl ClassAccessFlags : u16 {
        const PUBLIC = 1;
        const FINAL = 1 << 4;
        const SUPER = 1 << 5;
        const INTERFACE = 1 << 9;
        const ABSTRACT = 1 << 10;
        const SYNTHETIC = 1 << 12;
        const ANNOTATION = 1 << 13;
        const ENUM = 1 << 14;
        const MODULE = 1 << 15;
    }
}
bitflags! {
    impl FieldAccessFlags : u16 {
        const PUBLIC = 1;
        const PRIVATE = 1 << 1;
        const PROTECTED = 1 << 2;
        const STATIC = 1 << 3;
        const FINAL = 1 << 4;
        const VOLATILE = 1 << 6;
        const TRANSIENT = 1 << 7;
        const SYNTHETIC = 1 << 12;
        const ENUM = 1 << 14;
    }
}
bitflags! {
    impl MethodAccessFlags : u16 {
        const PUBLIC = 1;
        const PRIVATE = 1 << 1;
        const PROTECTED = 1 << 2;
        const STATIC = 1 << 3;
        const FINAL = 1 << 4;
        const SYNCHRONIZED = 1 << 5;
        const BRIDGE = 1 << 6;
        const VARARGS = 1 << 7;
        const NATIVE = 1 << 8;
        const ABSTRACT = 1 << 10;
        const STRICT = 1 << 11;
        const SYNTHETIC = 1 << 12;
    }
}

#[binrw]
pub struct AttributeInfo {
    pub name_index: u16,
    data_length: u32,
    #[br(count = data_length)]
    pub data: Vec<u8>,
}
#[binrw]
pub struct FieldInfo {
    pub access_flags: FieldAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    attributes_length: u16,
    #[br(count = attributes_length)]
    pub attributes: Vec<AttributeInfo>,
}
#[binrw]
pub struct MethodInfo {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    attributes_length: u16,
    #[br(count = attributes_length)]
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Clone)]
pub struct ModifiedUtf8String(pub String);
impl BinRead for ModifiedUtf8String {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let length = u16::read_options(reader, endian, args)?;
        let mut buffer = vec![0u8; length as usize];
        reader
            .read_exact(&mut buffer)
            .expect("Could not read from buffer");

        let string = String::from_utf8(buffer).map_err(|e| binrw::Error::AssertFail {
            pos: 0,
            message: format!("Invalid modified UTF-8: {}", e),
        })?;

        Ok(ModifiedUtf8String(string))
    }
}
impl BinWrite for ModifiedUtf8String {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let bytes = self.0.as_bytes();
        if bytes.len() > u16::MAX as usize {
            return Err(binrw::Error::AssertFail {
                pos: 0,
                message: "String too long to write as u16 length prefix".to_string(),
            });
        }
        (bytes.len() as u16).write_options(writer, endian, args)?;
        writer.write_all(bytes)?;
        Ok(())
    }
}

#[binrw]
pub struct CpClass {
    pub name_index: u16,
}
#[binrw]
pub struct CpString {
    string_index: u16,
}
#[binrw]
pub struct CpRef {
    class_index: u16,
    name_and_type_index: u16,
}
#[binrw]
pub struct CpNameAndType {
    name_index: u16,
    descriptor_index: u16,
}

pub enum ConstantPoolEntry {
    Utf8(ModifiedUtf8String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(CpClass),
    String(CpString),
    FieldRef(CpRef),
    MethodRef(CpRef),
    InterfaceMethodRef(CpRef),
    NameAndType(CpNameAndType),

    Invalid,
}

#[repr(u8)]
enum ConstantPoolTag {
    Utf8 = 1,
    Integer = 3,
    Float = 4,
    Long = 5,
    Double = 6,
    Class = 7,
    String = 8,
    FieldRef = 9,
    MethodRef = 10,
    InterfaceMethodRef = 11,
    NameAndType = 12,
    MethodHandle = 15,
    MethodType = 16,
    Dynamic = 17,
    InvokeDynamic = 18,
    Module = 19,
    Package = 20,
}
#[derive(Debug)]
struct InvalidCpTag {
    pub tag: u8,
}
impl TryFrom<u8> for ConstantPoolTag {
    type Error = InvalidCpTag;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ConstantPoolTag::Utf8),
            3 => Ok(ConstantPoolTag::Integer),
            4 => Ok(ConstantPoolTag::Float),
            5 => Ok(ConstantPoolTag::Long),
            6 => Ok(ConstantPoolTag::Double),
            7 => Ok(ConstantPoolTag::Class),
            8 => Ok(ConstantPoolTag::String),
            9 => Ok(ConstantPoolTag::FieldRef),
            10 => Ok(ConstantPoolTag::MethodRef),
            11 => Ok(ConstantPoolTag::InterfaceMethodRef),
            12 => Ok(ConstantPoolTag::NameAndType),
            15 => Ok(ConstantPoolTag::MethodHandle),
            16 => Ok(ConstantPoolTag::MethodType),
            17 => Ok(ConstantPoolTag::Dynamic),
            18 => Ok(ConstantPoolTag::InvokeDynamic),
            19 => Ok(ConstantPoolTag::Module),
            20 => Ok(ConstantPoolTag::Package),
            other => Err(InvalidCpTag { tag: other }),
        }
    }
}

impl BinRead for ConstantPoolEntry {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let tag_raw: u8 = reader.read_u8()?;
        ConstantPoolTag::try_from(tag_raw)
            .map_err(|err| binrw::Error::Custom {
                pos: 0,
                err: Box::new(format!("Invalid CP tag: {}", err.tag)),
            })
            .and_then(|tag| match tag {
                ConstantPoolTag::Utf8 => {
                    let str = ModifiedUtf8String::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::Utf8(str))
                }
                ConstantPoolTag::Integer => {
                    let integer = i32::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::Integer(integer))
                }
                ConstantPoolTag::Float => {
                    let float = f32::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::Float(float))
                }
                ConstantPoolTag::Long => {
                    let v = i64::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::Long(v))
                }
                ConstantPoolTag::Double => {
                    let double = f64::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::Double(double))
                }
                ConstantPoolTag::Class => {
                    let class = CpClass::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::Class(class))
                }
                ConstantPoolTag::String => {
                    let string = CpString::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::String(string))
                }
                ConstantPoolTag::FieldRef => {
                    let f_ref = CpRef::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::FieldRef(f_ref))
                }
                ConstantPoolTag::MethodRef => {
                    let m_ref = CpRef::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::MethodRef(m_ref))
                }
                ConstantPoolTag::InterfaceMethodRef => {
                    let im_ref = CpRef::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::InterfaceMethodRef(im_ref))
                }
                ConstantPoolTag::NameAndType => {
                    let nt = CpNameAndType::read_options(reader, endian, args)?;
                    Ok(ConstantPoolEntry::NameAndType(nt))
                }
                ConstantPoolTag::MethodHandle => todo!("CP MethodHandle not implemented"),
                ConstantPoolTag::MethodType => todo!("CP MethodType not implemented"),
                ConstantPoolTag::Dynamic => todo!("CP Dynamic not implemented"),
                ConstantPoolTag::InvokeDynamic => todo!("CP InvokeDynamic not implemented"),
                ConstantPoolTag::Module => todo!("CP Module not implemented"),
                ConstantPoolTag::Package => todo!("CP Package not implemented"),
            })
    }
}

impl BinWrite for ConstantPoolEntry {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        todo!("Writing is not supported for ConstantPoolEntry")
    }
}

pub struct ConstantPool {
    entries: Vec<ConstantPoolEntry>,
}
impl ConstantPool {
    pub fn find_utf8(&self, cp_index: u16) -> Option<&str> {
        let cp_index_s = cp_index as usize;
        if self.entries.len() <= cp_index_s {
            return None;
        }

        let cp_entry = &self.entries[cp_index_s];
        match cp_entry {
            ConstantPoolEntry::Utf8(cp_entry_utf8) => Some(cp_entry_utf8.0.as_str()),
            _ => None,
        }
    }
    pub fn find_class(&self, cp_index: u16) -> Option<&CpClass> {
        let cp_index_s = cp_index as usize;
        if self.entries.len() <= cp_index_s {
            return None;
        }

        let cp_entry = &self.entries[cp_index_s];
        match cp_entry {
            ConstantPoolEntry::Class(cp_entry_class) => Some(cp_entry_class),
            _ => None,
        }
    }
    pub fn find_int(&self, cp_index: u16) -> Option<i32> {
        let cp_index_s = cp_index as usize;
        if self.entries.len() <= cp_index_s {
            return None;
        }

        let cp_entry = &self.entries[cp_index_s];
        match cp_entry {
            ConstantPoolEntry::Integer(cp_entry_integer) => Some(*cp_entry_integer),
            _ => None,
        }
    }
    pub fn find_short(&self, cp_index: u16) -> Option<i16> {
        self.find_int(cp_index).map(|int| int as i16)
    }
    pub fn find_byte(&self, cp_index: u16) -> Option<u8> {
        self.find_int(cp_index).map(|int| int as u8)
    }
    pub fn find_char(&self, cp_index: u16) -> Option<char> {
        self.find_byte(cp_index).map(|byte| byte as char)
    }
    pub fn find_bool(&self, cp_index: u16) -> Option<bool> {
        self.find_int(cp_index).map(|int| int != 0)
    }

    pub fn find_float(&self, cp_index: u16) -> Option<f32> {
        let cp_index_s = cp_index as usize;
        if self.entries.len() <= cp_index_s {
            return None;
        }

        let cp_entry = &self.entries[cp_index_s];
        match cp_entry {
            ConstantPoolEntry::Float(cp_entry) => Some(*cp_entry),
            _ => None,
        }
    }

    pub fn find_long(&self, cp_index: u16) -> Option<i64> {
        let cp_index_s = cp_index as usize;
        if self.entries.len() <= cp_index_s {
            return None;
        }

        let cp_entry = &self.entries[cp_index_s];
        match cp_entry {
            ConstantPoolEntry::Long(cp_entry) => Some(*cp_entry),
            _ => None,
        }
    }

    pub fn find_double(&self, cp_index: u16) -> Option<f64> {
        let cp_index_s = cp_index as usize;
        if self.entries.len() <= cp_index_s {
            return None;
        }

        let cp_entry = &self.entries[cp_index_s];
        match cp_entry {
            ConstantPoolEntry::Double(cp_entry) => Some(*cp_entry),
            _ => None,
        }
    }

    pub fn find_string_ref(&self, cp_index: u16) -> Option<&str> {
        let cp_index_s = cp_index as usize;
        if self.entries.len() <= cp_index_s {
            return None;
        }

        let cp_entry = &self.entries[cp_index_s];
        match cp_entry {
            ConstantPoolEntry::String(cp_entry) => Some(self.find_utf8(cp_entry.string_index)?),
            _ => None,
        }
    }
}

impl BinRead for ConstantPool {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let length = u16::read_options(reader, endian, args)? as usize;
        let mut entries: Vec<ConstantPoolEntry> =
            (0..length).map(|_| ConstantPoolEntry::Invalid).collect();
        let mut parsed_count = 1;

        while parsed_count < length {
            let entry = ConstantPoolEntry::read_options(reader, endian, args)?;
            let insert_index = parsed_count;

            // 8-byte entries note: All 8-byte constants take up two entries in the constant_pool table of the class file.
            // If a CONSTANT_Long_info or CONSTANT_Double_info structure is the entry at index n
            // in the constant_pool table, then the next usable entry in the table is located at index n+2.
            // The constant_pool index n+1 must be valid but is considered unusable.
            match entry {
                ConstantPoolEntry::Long(_) | ConstantPoolEntry::Double(_) => parsed_count += 2,
                _ => parsed_count += 1,
            }

            entries[insert_index] = entry;
        }

        Ok(ConstantPool { entries })
    }
}
impl BinWrite for ConstantPool {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        todo!()
    }
}

#[binrw]
#[brw(big)]
pub struct JavaClassFile {
    pub version: Version,

    pub constant_pool: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,

    interfaces_length: u16,
    #[br(count = interfaces_length)]
    pub interfaces: Vec<u16>,

    fields_length: u16,
    #[br(count = fields_length)]
    pub fields: Vec<FieldInfo>,

    methods_length: u16,
    #[br(count = methods_length)]
    pub methods: Vec<MethodInfo>,

    attributes_length: u16,
    #[br(count = attributes_length)]
    pub attributes: Vec<AttributeInfo>,
}

impl JavaClassFile {
    pub fn get_name(&self) -> &str {
        let class_info = self.constant_pool.find_class(self.this_class).unwrap();
        self.constant_pool.find_utf8(class_info.name_index).unwrap()
    }
}

// Attributes

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub struct ConstantValueAttributeRaw {
    pub value_cp_index: u16,
}

#[binrw]
#[brw(big)]
pub struct SourceFileAttributeRaw {
    pub file_name_cp_index: u16,
}
