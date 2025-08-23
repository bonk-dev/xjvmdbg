use binrw::BinRead;
use std::io::{Read, Seek};

use crate::{
    java_class::JavaClassContainerBuilder,
    java_class::errors::AttributeReadError,
    java_class_file::{
        CodeAttributeRaw, CodeExceptionRaw, ConstantValueAttributeRaw, JavaClassFile,
        SourceFileAttributeRaw,
    },
};

#[derive(Debug)]
pub enum ConstantAttribute {
    Int(i32),
    Short(i16),
    Char(char),
    Byte(u8),
    Boolean(bool),
    Float(f32),
    Long(i64),
    Double(f64),
    String(String),
}
impl ToString for ConstantAttribute {
    fn to_string(&self) -> String {
        match self {
            ConstantAttribute::Int(int) => int.to_string(),
            ConstantAttribute::Short(short) => short.to_string(),
            ConstantAttribute::Char(char) => char.to_string(),
            ConstantAttribute::Byte(byte) => byte.to_string(),
            ConstantAttribute::Boolean(bool) => bool.to_string(),
            ConstantAttribute::Float(float) => float.to_string(),
            ConstantAttribute::Long(long) => long.to_string(),
            ConstantAttribute::Double(double) => double.to_string(),
            ConstantAttribute::String(string) => format!("\"{}\"", string),
        }
    }
}

#[derive(Debug)]
pub struct SourceFileAttribute {
    pub file_name: String,
}
impl SourceFileAttribute {
    pub fn read<T: Read + Seek>(
        reader: &mut T,
        raw_class: &JavaClassFile,
    ) -> Result<Self, binrw::Error> {
        let raw_r = SourceFileAttributeRaw::read(reader);
        match raw_r {
            Ok(raw) => {
                let file_name = raw_class
                    .constant_pool
                    .find_utf8(raw.file_name_cp_index)
                    .unwrap();
                Result::Ok(SourceFileAttribute {
                    file_name: String::from(file_name),
                })
            }
            Err(err) => Result::Err(err),
        }
    }
}

#[derive(Debug)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<CodeExceptionRaw>,
    pub attributes: Vec<AttributeType>,
}
impl CodeAttribute {
    pub(crate) fn read<R>(
        reader: &mut R,
        raw_class: &JavaClassFile,
    ) -> Result<Self, AttributeReadError>
    where
        R: Read + Seek,
    {
        let raw_r = CodeAttributeRaw::read(reader);
        if raw_r.is_err() {
            return Err(AttributeReadError::Deserialization(raw_r.unwrap_err()));
        }

        let raw = raw_r.unwrap();
        let mut attributes = Vec::with_capacity(raw.attributes.len());
        for attr_info in raw.attributes {
            let attr = JavaClassContainerBuilder::parse_attribute(&attr_info, &raw_class);
            attributes.push(attr);
        }

        // TODO: Parse code
        Ok(CodeAttribute {
            max_stack: raw.max_stack,
            max_locals: raw.max_locals,
            code: raw.code,
            exception_table: raw.exception_table,
            attributes: attributes,
        })
    }
}

#[derive(Debug)]
pub struct ErrorAttribute {
    pub message: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum AttributeType {
    Code(CodeAttribute),
    ConstantValue(ConstantAttribute),
    ConstantValueIndex(ConstantValueAttributeRaw),
    Deprecated,
    SourceFile(SourceFileAttribute),
    Error(ErrorAttribute),
}
