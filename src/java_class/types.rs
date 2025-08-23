use std::rc::Rc;

use crate::{
    descriptors::{FieldDescriptor, MethodDescriptor},
    java_class::AttributeType,
    java_class_file::{FieldAccessFlags, MethodAccessFlags, Version},
};

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
