#[derive(Debug, PartialEq, Eq)]
pub enum DescriptorError {
    InvalidChar(char),
    UnexpectedEnd,
    ClassTerminatorNotFound,
    TooManyArrayDimensions,

    MissingOpenParen,
    MissingCloseParen,
    InvalidReturnType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    SignedByte,
    Char,
    Double,
    Float,
    Integer,
    Long,
    Short,
    Boolean,
    Void,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentType {
    Base(Type),
    Object { class_name: String },
}

#[derive(Debug, PartialEq, Eq)]
pub struct FieldDescriptor {
    pub element_type: ComponentType,
    pub array_dimension: Option<u8>,
}
impl FieldDescriptor {
    pub fn new(component: ComponentType, dimension: Option<u8>) -> Self {
        if dimension.is_some_and(|d| d == 0) {
            panic!("Arrays must have a positive dimension")
        }

        FieldDescriptor {
            element_type: component,
            array_dimension: dimension,
        }
    }
    pub fn from_type(base_type: Type) -> Self {
        Self {
            element_type: ComponentType::Base(base_type),
            array_dimension: None,
        }
    }
    pub fn from_class(class_name: String) -> Self {
        Self {
            element_type: ComponentType::Object {
                class_name: class_name,
            },
            array_dimension: None,
        }
    }
    pub fn from_class_str(class_name: &str) -> Self {
        Self {
            element_type: ComponentType::Object {
                class_name: class_name.to_string(),
            },
            array_dimension: None,
        }
    }
    pub fn from_type_array(base_type: Type, dimension: u8) -> Self {
        if dimension == 0 {
            panic!("Arrays must have a positive dimension")
        }
        Self {
            element_type: ComponentType::Base(base_type),
            array_dimension: Some(dimension),
        }
    }
    pub fn from_class_array(class_name: &str, dimension: u8) -> Self {
        if dimension == 0 {
            panic!("Arrays must have a positive dimension")
        }
        Self {
            element_type: ComponentType::Object {
                class_name: class_name.to_string(),
            },
            array_dimension: Some(dimension),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MethodDescriptor {
    pub parameters: Vec<FieldDescriptor>,
    pub return_type: Option<FieldDescriptor>,
}

impl MethodDescriptor {
    pub fn new(parameters: Vec<FieldDescriptor>, return_type: Option<FieldDescriptor>) -> Self {
        Self {
            parameters,
            return_type,
        }
    }
}

fn parse_component_type(descriptor: &str) -> Result<(ComponentType, usize), DescriptorError> {
    if descriptor.is_empty() {
        return Err(DescriptorError::UnexpectedEnd);
    }

    match descriptor.chars().next().unwrap() {
        'L' => {
            let semicolon_pos = descriptor
                .find(';')
                .ok_or(DescriptorError::ClassTerminatorNotFound)?;

            let class_name = &descriptor[1..semicolon_pos];
            Ok((
                ComponentType::Object {
                    class_name: class_name.to_string(),
                },
                semicolon_pos + 1,
            ))
        }
        'B' => Ok((ComponentType::Base(Type::SignedByte), 1)),
        'C' => Ok((ComponentType::Base(Type::Char), 1)),
        'D' => Ok((ComponentType::Base(Type::Double), 1)),
        'F' => Ok((ComponentType::Base(Type::Float), 1)),
        'I' => Ok((ComponentType::Base(Type::Integer), 1)),
        'J' => Ok((ComponentType::Base(Type::Long), 1)),
        'S' => Ok((ComponentType::Base(Type::Short), 1)),
        'Z' => Ok((ComponentType::Base(Type::Boolean), 1)),
        'V' => Ok((ComponentType::Base(Type::Void), 1)),
        other => Err(DescriptorError::InvalidChar(other)),
    }
}

fn parse_field_descriptor_at_position(
    descriptor: &str,
    start: usize,
) -> Result<(FieldDescriptor, usize), DescriptorError> {
    if start >= descriptor.len() {
        return Err(DescriptorError::UnexpectedEnd);
    }

    let mut dimension_count = 0u32;
    let mut pos = start;

    while pos < descriptor.len() && descriptor.chars().nth(pos).unwrap() == '[' {
        dimension_count += 1;
        if dimension_count > u8::MAX as u32 {
            return Err(DescriptorError::TooManyArrayDimensions);
        }
        pos += 1;
    }

    if pos >= descriptor.len() {
        return Err(DescriptorError::UnexpectedEnd);
    }

    let (component_type, consumed) = parse_component_type(&descriptor[pos..])?;
    let array_dimension = if dimension_count > 0 {
        Some(dimension_count as u8)
    } else {
        None
    };

    let field_descriptor = FieldDescriptor::new(component_type, array_dimension);
    Ok((field_descriptor, pos + consumed))
}

pub fn parse_field_descriptor(descriptor: &str) -> Result<FieldDescriptor, DescriptorError> {
    let (field_descriptor, consumed) = parse_field_descriptor_at_position(descriptor, 0)?;

    // Ensure we consumed the entire descriptor
    if consumed != descriptor.len() {
        return Err(DescriptorError::InvalidChar(
            descriptor.chars().nth(consumed).unwrap(),
        ));
    }

    Ok(field_descriptor)
}

pub fn parse_method_descriptor(descriptor: &str) -> Result<MethodDescriptor, DescriptorError> {
    if descriptor.is_empty() {
        return Err(DescriptorError::UnexpectedEnd);
    }

    // Method descriptors must start with '('
    if !descriptor.starts_with('(') {
        return Err(DescriptorError::MissingOpenParen);
    }

    // Find the closing parenthesis
    let close_paren_pos = descriptor
        .find(')')
        .ok_or(DescriptorError::MissingCloseParen)?;

    let parameter_section = &descriptor[1..close_paren_pos];
    let return_section = &descriptor[close_paren_pos + 1..];

    // Parse parameters
    let mut parameters = Vec::new();
    let mut pos = 0;

    while pos < parameter_section.len() {
        let (param, consumed) = parse_field_descriptor_at_position(parameter_section, pos)?;
        parameters.push(param);
        pos = consumed;
    }

    // Parse return type
    let return_type = if return_section.is_empty() {
        return Err(DescriptorError::UnexpectedEnd);
    } else if return_section == "V" {
        None // Void return type
    } else {
        let return_descriptor = parse_field_descriptor(return_section)?;
        Some(return_descriptor)
    };

    Ok(MethodDescriptor::new(parameters, return_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_descriptor_empty() {
        let descriptor = "";
        let expected = Err(DescriptorError::UnexpectedEnd);
        let actual = parse_field_descriptor(descriptor);

        assert_eq!(actual, expected);
    }

    #[test]
    fn field_descriptor_invalid_char() {
        let descriptor = "X";
        let expected = Err(DescriptorError::InvalidChar('X'));
        let actual = parse_field_descriptor(descriptor);

        assert_eq!(actual, expected);
    }

    #[test]
    fn field_descriptor_long() {
        let descriptor = "J";
        let expected = FieldDescriptor::from_type(Type::Long);
        let actual = parse_field_descriptor(descriptor).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn field_descriptor_class_name_valid() {
        let descriptor = "Ldev/dpago/Xjvmdbgtest;";
        let expected = FieldDescriptor::from_class_str("dev/dpago/Xjvmdbgtest");
        let actual = parse_field_descriptor(descriptor).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn field_descriptor_class_name_no_terminator() {
        let descriptor = "Ldev/dpago/xjvmdbgtest";
        let expected = Err(DescriptorError::ClassTerminatorNotFound);
        let actual = parse_field_descriptor(descriptor);

        assert_eq!(actual, expected);
    }

    #[test]
    fn field_descriptor_array_long() {
        let descriptor = "[[[J";
        let expected = FieldDescriptor::from_type_array(Type::Long, 3);
        let actual = parse_field_descriptor(descriptor).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn field_descriptor_array_class() {
        let descriptor = "[[Ljava/lang/String;";
        let expected = FieldDescriptor::from_class_array("java/lang/String", 2);
        let actual = parse_field_descriptor(descriptor).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn field_descriptor_array_no_element() {
        let descriptor = "[[[";
        let expected = Err(DescriptorError::UnexpectedEnd);
        let actual = parse_field_descriptor(descriptor);
        assert_eq!(actual, expected);
    }

    #[test]
    fn field_descriptor_array_too_many_dimensions() {
        let descriptor = format!("{}I", "[".repeat(256));
        let expected = Err(DescriptorError::TooManyArrayDimensions);
        let actual = parse_field_descriptor(&descriptor);
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_empty() {
        let descriptor = "";
        let expected = Err(DescriptorError::UnexpectedEnd);
        let actual = parse_method_descriptor(descriptor);
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_missing_open_paren() {
        let descriptor = "I)V";
        let expected = Err(DescriptorError::MissingOpenParen);
        let actual = parse_method_descriptor(descriptor);
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_missing_close_paren() {
        let descriptor = "(IV";
        let expected = Err(DescriptorError::MissingCloseParen);
        let actual = parse_method_descriptor(descriptor);
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_void_no_params() {
        let descriptor = "()V";
        let expected = MethodDescriptor::new(vec![], None);
        let actual = parse_method_descriptor(descriptor).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_simple_params() {
        let descriptor = "(IJ)V";
        let expected = MethodDescriptor::new(
            vec![
                FieldDescriptor::from_type(Type::Integer),
                FieldDescriptor::from_type(Type::Long),
            ],
            None,
        );
        let actual = parse_method_descriptor(descriptor).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_with_return_type() {
        let descriptor = "(I)Ljava/lang/String;";
        let expected = MethodDescriptor::new(
            vec![FieldDescriptor::from_type(Type::Integer)],
            Some(FieldDescriptor::from_class_str("java/lang/String")),
        );
        let actual = parse_method_descriptor(descriptor).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_array_params() {
        let descriptor = "([I[[Ljava/lang/String;)V";
        let expected = MethodDescriptor::new(
            vec![
                FieldDescriptor::from_type_array(Type::Integer, 1),
                FieldDescriptor::from_class_array("java/lang/String", 2),
            ],
            None,
        );
        let actual = parse_method_descriptor(descriptor).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_complex() {
        let descriptor = "(ILjava/lang/String;[BZ)Ljava/lang/Object;";
        let expected = MethodDescriptor::new(
            vec![
                FieldDescriptor::from_type(Type::Integer),
                FieldDescriptor::from_class_str("java/lang/String"),
                FieldDescriptor::from_type_array(Type::SignedByte, 1),
                FieldDescriptor::from_type(Type::Boolean),
            ],
            Some(FieldDescriptor::from_class_str("java/lang/Object")),
        );
        let actual = parse_method_descriptor(descriptor).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_no_return_type() {
        let descriptor = "(I)";
        let expected = Err(DescriptorError::UnexpectedEnd);
        let actual = parse_method_descriptor(descriptor);
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_invalid_param() {
        let descriptor = "(X)V";
        let expected = Err(DescriptorError::InvalidChar('X'));
        let actual = parse_method_descriptor(descriptor);
        assert_eq!(actual, expected);
    }

    #[test]
    fn method_descriptor_invalid_return_type() {
        let descriptor = "(I)X";
        let expected = Err(DescriptorError::InvalidChar('X'));
        let actual = parse_method_descriptor(descriptor);
        assert_eq!(actual, expected);
    }
}
