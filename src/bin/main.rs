use binrw::BinRead;
use std::fs;
use std::io::{Cursor, Read};
use xjvmdbg::java_class::JavaClassContainerBuilder;
use xjvmdbg::java_class_file::JavaClassFile;

fn main() {
    let jar_file = fs::File::open(
        "/home/bonk/Programowanie/jetagent-testapp/target/original-jb-hello-world-maven-0.2.0.jar",
    )
    .unwrap();
    let mut zip = zip::ZipArchive::new(jar_file).unwrap();

    let mut raw_files: Vec<JavaClassFile> = vec![];
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        if file.name().ends_with(".class") {
            println!("Reading: {}", file.name());

            let mut buffer = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buffer).unwrap();

            let mut cursor = Cursor::new(buffer);
            raw_files.push(JavaClassFile::read(&mut cursor).unwrap());
        }
    }

    let classes = JavaClassContainerBuilder::new(&raw_files).parse_classes();

    println!("Parsing done");

    for c in classes.into_values() {
        println!("Class: {}", c.name);
        println!("  Attributes:");
        for attr in c.attributes.iter() {
            match attr {
                xjvmdbg::java_class::AttributeType::ConstantValueIndex(idx) => {
                    println!("  -> Constant value index: {}", idx.value_cp_index)
                }
                xjvmdbg::java_class::AttributeType::Deprecated => {
                    println!("  -> Is deprecated")
                }
                xjvmdbg::java_class::AttributeType::SourceFile(source_file_attribute) => {
                    println!("  -> Source file: {}", source_file_attribute.file_name)
                }
                xjvmdbg::java_class::AttributeType::Error(error_attribute) => {
                    println!("  -> [Error]: msg: {}", error_attribute.message)
                }
                _ => {
                    println!("  -> Invalid attribute (not expected on a class)")
                }
            }
        }

        if c.fields.len() > 0 {
            println!("  Fields:");
            for field in c.fields.iter() {
                println!("  -> Name: {}", field.name);
                println!("     Descriptor: {:?}", field.descriptor);
                println!("     Access: {}", field.access_flags.bits().to_string());

                for attr in field.attributes.iter() {
                    match attr {
                        xjvmdbg::java_class::AttributeType::Deprecated => {
                            println!("     -> Is deprecated")
                        }
                        xjvmdbg::java_class::AttributeType::Error(error_attribute) => {
                            println!("     -> [Error]: msg: {}", error_attribute.message)
                        }
                        xjvmdbg::java_class::AttributeType::ConstantValue(cval) => {
                            println!("     -> Constant value: {}", cval.to_string())
                        }
                        _ => {
                            println!("     -> Invalid attribute (not expected on a field)")
                        }
                    }
                }
            }
        } else {
            println!("  Fields: none");
        }

        if c.methods.len() > 0 {
            println!("  Methods:");
            for method in c.methods.iter() {
                println!("  -> Name: {}", method.name);
                println!("     Descriptor: {:?}", method.descriptor);
                println!("     Access: {}", method.access_flags.bits().to_string());

                for attr in method.attributes.iter() {
                    match attr {
                        xjvmdbg::java_class::AttributeType::Deprecated => {
                            println!("     -> Is deprecated")
                        }
                        xjvmdbg::java_class::AttributeType::Error(error_attribute) => {
                            println!("     -> [Error]: msg: {}", error_attribute.message)
                        }
                        _ => {
                            println!("     -> Invalid attribute (not expected on a method)")
                        }
                    }
                }
            }
        } else {
            println!("  Methods: none");
        }
    }
}
