use binrw::BinRead;
use std::{collections::HashMap, io::Cursor, rc::Rc};

use crate::{
    descriptors::{ComponentType, Type},
    java_class::{
        AttributeType, CodeAttribute, ConstantAttribute, ErrorAttribute, Field, JavaClass, Method,
        SourceFileAttribute, errors::AttributeReadError, errors::ConstantValueReadError,
    },
    java_class_file::{AttributeInfo, ConstantValueAttributeRaw, JavaClassFile},
};

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
            "Code" => CodeAttribute::read(&mut cursor, &raw_class).map(AttributeType::Code),
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

    pub(crate) fn parse_attribute(
        attribute_info: &AttributeInfo,
        raw_class: &JavaClassFile,
    ) -> AttributeType {
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
            ComponentType::Base(field_type) => match field_type {
                Type::SignedByte => {
                    let byte = raw_class
                        .constant_pool
                        .find_byte(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Byte(byte))
                }
                Type::Char => {
                    let char = raw_class
                        .constant_pool
                        .find_char(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Char(char))
                }
                Type::Double => {
                    let d = raw_class
                        .constant_pool
                        .find_double(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Double(d))
                }
                Type::Float => {
                    let f = raw_class
                        .constant_pool
                        .find_float(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Float(f))
                }
                Type::Integer => {
                    let i = raw_class
                        .constant_pool
                        .find_int(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Int(i))
                }
                Type::Long => {
                    let l = raw_class
                        .constant_pool
                        .find_long(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Long(l))
                }
                Type::Short => {
                    let s = raw_class
                        .constant_pool
                        .find_short(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Short(s))
                }
                Type::Boolean => {
                    let b = raw_class
                        .constant_pool
                        .find_bool(idx)
                        .ok_or(ConstantValueReadError::NotFoundInPool)?;
                    Ok(ConstantAttribute::Boolean(b))
                }
                Type::Void => Err(ConstantValueReadError::VoidField),
            },
            ComponentType::Object { class_name } => {
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

            let descriptor = crate::descriptors::parse_field_descriptor(descriptor_raw_string);
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

            let descriptor = crate::descriptors::parse_method_descriptor(descriptor_raw);
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
