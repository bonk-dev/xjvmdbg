pub(crate) enum AttributeReadError {
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
pub(crate) enum ConstantValueReadError {
    ReferenceTypeNotString,
    NotFoundInPool,
    VoidField,
}
