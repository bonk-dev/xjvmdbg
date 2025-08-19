use binrw::BinRead;

use crate::descriptors::{
    self, FieldDescriptor, MethodDescriptor, parse_field_descriptor, parse_method_descriptor,
};
use crate::java_class_file::{
    AttributeInfo, ConstantValueAttributeRaw, FieldAccessFlags, JavaClassFile, MethodAccessFlags,
    SourceFileAttributeRaw, Version,
};
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};
use std::rc::Rc;

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
    fn read<T: Read + Seek>(
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
pub struct ErrorAttribute {
    pub message: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum AttributeType {
    ConstantValue(ConstantAttribute),
    ConstantValueIndex(ConstantValueAttributeRaw),
    Deprecated,
    SourceFile(SourceFileAttribute),
    Error(ErrorAttribute),
}

#[derive(Debug)]
pub struct Field {
    pub access_flags: FieldAccessFlags,
    pub name: String,
    pub descriptor: FieldDescriptor,
    pub attributes: Vec<AttributeType>,
}
impl Field {
    pub fn new(access_flags: FieldAccessFlags, name: &str, descriptor: FieldDescriptor) -> Self {
        Field {
            access_flags,
            name: String::from(name),
            descriptor: descriptor,
            attributes: vec![],
        }
    }
}

pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name: String,
    pub descriptor: MethodDescriptor,
    pub attributes: Vec<AttributeType>,
}
impl Method {
    pub fn new(access_flags: MethodAccessFlags, name: &str, descriptor: MethodDescriptor) -> Self {
        Method {
            access_flags,
            name: String::from(name),
            descriptor: descriptor,
            attributes: vec![],
        }
    }
}

pub struct JavaClass {
    pub version: Version,
    pub name: String,
    pub super_class: Option<Rc<JavaClass>>,
    pub interfaces: Vec<Rc<JavaClass>>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<AttributeType>,
}
impl JavaClass {
    pub fn new(version: Version, name: String) -> Self {
        JavaClass {
            version,
            name,
            super_class: None,
            interfaces: vec![],
            fields: vec![],
            methods: vec![],
            attributes: vec![],
        }
    }

    /// Creates a JavaClass from only the 'name' (used for unresolved classes).
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the class
    pub fn from_name(name: &str) -> JavaClass {
        JavaClass {
            version: Version::default(),
            name: String::from(name),
            super_class: None,
            interfaces: vec![],
            fields: vec![],
            methods: vec![],
            attributes: vec![],
        }
    }
}

enum AttributeReadError {
    Deserialization(binrw::Error),
    NotSuported,
}
impl ToString for AttributeReadError {
    fn to_string(&self) -> String {
        match self {
            AttributeReadError::Deserialization(error) => {
                format!("Deserialization error: {}", error.to_string())
            }
            AttributeReadError::NotSuported => String::from("Not supported"),
        }
    }
}

#[derive(Debug)]
enum ConstantValueReadError {
    ReferenceTypeNotString,
    NotFoundInPool,
    VoidField,
}

pub struct JavaClassContainerBuilder<'a> {
    raw_classes: &'a Vec<JavaClassFile>,
    classes: HashMap<String, Rc<JavaClass>>,
}
impl<'a> JavaClassContainerBuilder<'a> {
    pub fn new(raw_classes: &'a Vec<JavaClassFile>) -> Self {
        JavaClassContainerBuilder {
            raw_classes,
            classes: HashMap::new(),
        }
    }

    fn find_class(&self, name: &str) -> Option<&Rc<JavaClass>> {
        let result = self.classes.iter().find(|(class_name, _class)| {
            return *class_name == name;
        });
        if result.is_some() {
            return Some(result.unwrap().1);
        }

        None
    }

    fn parse_super_class(&mut self, class: &mut JavaClass, raw_class: &JavaClassFile) {
        if raw_class.super_class != 0 {
            let super_class_info = raw_class
                .constant_pool
                .find_class(raw_class.super_class)
                .unwrap();
            let super_name = raw_class
                .constant_pool
                .find_utf8(super_class_info.name_index)
                .unwrap();

            let parsed_class = self.find_class(super_name);
            class.super_class = match parsed_class {
                None => {
                    let raw_super = self
                        .raw_classes
                        .iter()
                        .find(|raw_class| raw_class.get_name() == super_name);
                    if let Some(found_super) = raw_super {
                        let parsed_super = self.parse_class(found_super);
                        Some(parsed_super)
                    } else {
                        let dummy_rc = Rc::new(JavaClass::from_name(super_name));
                        self.classes
                            .insert(super_name.to_string(), Rc::clone(&dummy_rc));
                        Some(dummy_rc)
                    }
                }
                Some(parsed_super) => Some(Rc::clone(&parsed_super)),
            };
        }
    }

    fn read_attribute(
        data: &Vec<u8>,
        attr_name: &str,
        raw_class: &JavaClassFile,
    ) -> Result<AttributeType, AttributeReadError> {
        let mut cursor = Cursor::new(data);
        match attr_name {
            "ConstantValue" => ConstantValueAttributeRaw::read(&mut cursor)
                .map(AttributeType::ConstantValueIndex)
                .map_err(AttributeReadError::Deserialization),
            "Deprecated" => Ok(AttributeType::Deprecated),
            "SourceFile" => SourceFileAttribute::read(&mut cursor, &raw_class)
                .map(AttributeType::SourceFile)
                .map_err(AttributeReadError::Deserialization),
            _ => Result::Err(AttributeReadError::NotSuported),
        }
    }

    fn parse_attribute(attribute_info: &AttributeInfo, raw_class: &JavaClassFile) -> AttributeType {
        let name = raw_class
            .constant_pool
            .find_utf8(attribute_info.name_index)
            .unwrap();
        println!("Attribute: {}", name);

        let read_result =
            JavaClassContainerBuilder::read_attribute(&attribute_info.data, name, raw_class);
        match read_result {
            Ok(attribute) => attribute,
            Err(e) => match e {
                AttributeReadError::Deserialization(error) => {
                    AttributeType::Error(ErrorAttribute {
                        message: format!("Deserialization error: {}", error.to_string()),
                        data: attribute_info.data.clone(),
                    })
                }
                AttributeReadError::NotSuported => AttributeType::Error(ErrorAttribute {
                    message: format!("Not supported: {}", name),
                    data: attribute_info.data.clone(),
                }),
            },
        }
    }

    fn parse_field_constant_value(
        field: &mut Field,
        idx: u16,
        raw_class: &JavaClassFile,
    ) -> Result<ConstantAttribute, ConstantValueReadError> {
        match &field.descriptor.element_type {
            descriptors::ComponentType::Base(field_type) => match field_type {
                descriptors::Type::SignedByte => {
                    let byte = raw_class
                        .constant_pool
                        .find_byte(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Byte(byte))
                }
                descriptors::Type::Char => {
                    let char = raw_class
                        .constant_pool
                        .find_char(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Char(char))
                }
                descriptors::Type::Double => {
                    let d = raw_class
                        .constant_pool
                        .find_double(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Double(d))
                }
                descriptors::Type::Float => {
                    let f = raw_class
                        .constant_pool
                        .find_float(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Float(f))
                }
                descriptors::Type::Integer => {
                    let i = raw_class
                        .constant_pool
                        .find_int(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Int(i))
                }
                descriptors::Type::Long => {
                    let l = raw_class
                        .constant_pool
                        .find_long(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Long(l))
                }
                descriptors::Type::Short => {
                    let s = raw_class
                        .constant_pool
                        .find_short(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Short(s))
                }
                descriptors::Type::Boolean => {
                    let b = raw_class
                        .constant_pool
                        .find_bool(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Boolean(b))
                }
                descriptors::Type::Void => Err(ConstantValueReadError::VoidField),
            },
            descriptors::ComponentType::Object { class_name } => {
                if class_name == "java/lang/String" {
                    let s = raw_class
                        .constant_pool
                        .find_string_ref(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    return Ok(ConstantAttribute::String(s.to_string()));
                }

                Err(ConstantValueReadError::ReferenceTypeNotString)
            }
        }
    }

    fn parse_fields(class: &mut JavaClass, raw_class: &JavaClassFile) {
        for field_info in raw_class.fields.iter() {
            let name = raw_class
                .constant_pool
                .find_utf8(field_info.name_index)
                .unwrap();
            let descriptor_raw_string = raw_class
                .constant_pool
                .find_utf8(field_info.descriptor_index)
                .unwrap();
            println!("Field: {} (d: {})", name, descriptor_raw_string);

            let descriptor = parse_field_descriptor(descriptor_raw_string);
            if descriptor.is_err() {
                println!(
                    "Could not parse field descriptor: {:?}",
                    descriptor.unwrap_err()
                );
                continue;
            }

            let mut field = Field::new(field_info.access_flags, name, descriptor.unwrap());

            // parse attributes
            for a_info in field_info.attributes.iter() {
                let parsed_attribute =
                    JavaClassContainerBuilder::parse_attribute(a_info, raw_class);

                match parsed_attribute {
                    AttributeType::ConstantValueIndex(idx) => {
                        let cvalue_result = JavaClassContainerBuilder::parse_field_constant_value(
                            &mut field,
                            idx.value_cp_index,
                            raw_class,
                        );
                        field.attributes.push(match cvalue_result {
                            Ok(cvalue) => AttributeType::ConstantValue(cvalue),
                            Err(e) => AttributeType::Error(ErrorAttribute {
                                message: format!("Could not parse constant value, error: {:?}", e),
                                data: a_info.data.clone(),
                            }),
                        });
                    }
                    _ => {
                        field.attributes.push(parsed_attribute);
                    }
                }
            }

            class.fields.push(field);
        }
    }

    fn parse_methods(class: &mut JavaClass, raw_class: &JavaClassFile) {
        for method_info in raw_class.methods.iter() {
            let name = raw_class
                .constant_pool
                .find_utf8(method_info.name_index)
                .unwrap();
            let descriptor_raw = raw_class
                .constant_pool
                .find_utf8(method_info.descriptor_index)
                .unwrap();
            println!("Method: {} (d: {})", name, descriptor_raw);

            let descriptor = parse_method_descriptor(descriptor_raw);
            if descriptor.is_err() {
                println!(
                    "Could not parse method descriptor: {:?}",
                    descriptor.unwrap_err()
                );
                continue;
            }

            let mut method = Method::new(method_info.access_flags, name, descriptor.unwrap());

            // parse attributes
            for a_info in method_info.attributes.iter() {
                let parsed_attribute =
                    JavaClassContainerBuilder::parse_attribute(a_info, raw_class);
                method.attributes.push(parsed_attribute);
            }

            class.methods.push(method);
        }
    }

    fn parse_class_attributes(
        &self,
        class: &mut JavaClass,
        raw_class: &JavaClassFile,
    ) -> Result<(), binrw::Error> {
        for a_info in raw_class.attributes.iter() {
            let parsed_attribute = JavaClassContainerBuilder::parse_attribute(a_info, raw_class);
            class.attributes.push(parsed_attribute);
        }

        Ok(())
    }

    fn parse_class(&mut self, raw_class: &JavaClassFile) -> Rc<JavaClass> {
        let name = raw_class.get_name();
        if let Some(c) = self.find_class(name) {
            println!("Already parsed {}", name);
            return Rc::clone(c);
        }

        let mut class = JavaClass::new(raw_class.version.clone(), name.to_string());
        self.parse_super_class(&mut class, raw_class);

        if let Err(attr_error) = self.parse_class_attributes(&mut class, raw_class) {
            class.attributes.clear();
            class.attributes.push(AttributeType::Error(ErrorAttribute {
                message: format!("Could not parse attributes: {}", attr_error.to_string()),
                data: vec![],
            }));
        }

        JavaClassContainerBuilder::parse_fields(&mut class, raw_class);
        JavaClassContainerBuilder::parse_methods(&mut class, raw_class);

        let rc = Rc::new(class);
        self.classes.insert(name.to_string(), Rc::clone(&rc));

        rc
    }

    pub fn parse_classes(mut self) -> HashMap<String, Rc<JavaClass>> {
        for c in self.raw_classes.iter() {
            self.parse_class(c);
        }

        self.classes
    }
}
