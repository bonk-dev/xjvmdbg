use binrw::BinRead;
use std::fs;
use std::io::{Cursor, Read};
use tokio::net::TcpStream;
use xjvmdbg::java_class::JavaClassContainerBuilder;
use xjvmdbg::java_class_file::JavaClassFile;
use xjvmdbg::jdwp::JdwpClient;

#[tokio::main]
async fn main() {
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
                        xjvmdbg::java_class::AttributeType::Code(code) => {
                            println!("     Code:");
                            println!("        Max stack: {}", code.max_stack);
                            println!("        Max locals: {}", code.max_locals);
                            println!("        Code length: {} bytes", code.code.len());
                            println!(
                                "        Exception table length: {}",
                                code.exception_table.len()
                            );

                            if code.attributes.is_empty() {
                                println!("        Attributes: none");
                            } else {
                                println!("        Attributes:");
                                for code_attr in code.attributes.iter() {
                                    match code_attr {
                                        xjvmdbg::java_class::AttributeType::Error(
                                            error_attribute,
                                        ) => {
                                            println!(
                                                "        -> [Error]: msg: {}",
                                                error_attribute.message
                                            )
                                        }
                                        _ => {
                                            println!(
                                                "        -> Invalid attribute (not expected on code)"
                                            )
                                        }
                                    }
                                }
                            }

                            println!("        Disassembly:");
                            let mut cursor = Cursor::new(&code.code);
                            match xjvmdbg::bytecode::parse_instructions(&mut cursor) {
                                Ok(instructions) => {
                                    for i in instructions {
                                        println!("          {:?}", i);
                                    }
                                }
                                Err(e) => {
                                    println!("          Could not read instructions: {}", e);
                                }
                            }
                        }
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

    let stream = TcpStream::connect("127.0.0.1:47239").await.unwrap();
    let mut client = JdwpClient::new(stream).await.unwrap();
    let version = client.vm_get_version().await.unwrap();
    let id_sizes = client.vm_get_id_sizes().await.unwrap();
    client.get_id_sizes().await.unwrap();

    println!("Version: {:?}", version);
    println!("ID sizes: {:?}", id_sizes);
}
