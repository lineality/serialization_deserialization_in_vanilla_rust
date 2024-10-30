use std::fs;
use std::path::Path; // Import Path directly
use std::ffi::OsStr; // Import OsStr directly
use std::net::{Ipv4Addr, Ipv6Addr}; // These are unused in the MVP, you can remove them
use toml::Value;
use std::fmt; // Import fmt for implementing Display

// Your CollaboratorTomlData struct (simplified for MVP)
#[derive(Debug)]
struct CollaboratorTomlData {
    user_name: String,
    user_salt_list: Vec<u128>, 
}

// Your YOURPROGRAMError type (simplified for MVP)
#[derive(Debug)]
enum YOURPROGRAMError {
    IoError(std::io::Error),
    TomlError(String), 
    ParseIntError(std::num::ParseIntError),
}

impl From<std::io::Error> for YOURPROGRAMError {
    fn from(err: std::io::Error) -> Self {
        YOURPROGRAMError::IoError(err)
    }
}

impl From<std::num::ParseIntError> for YOURPROGRAMError {
    fn from(err: std::num::ParseIntError) -> Self {
        YOURPROGRAMError::ParseIntError(err)
    }
}

impl fmt::Display for YOURPROGRAMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YOURPROGRAMError::IoError(err) => write!(f, "IO Error: {}", err),
            YOURPROGRAMError::TomlError(err) => write!(f, "TOML Error: {}", err),
            YOURPROGRAMError::ParseIntError(err) => write!(f, "Parse Int Error: {}", err),
        }
    }
}

// fn read_a_collaborator_setup_toml() -> Result<(Vec<CollaboratorTomlData>, Vec<YOURPROGRAMError>), YOURPROGRAMError> {
//     let mut collaborators = Vec::new();
//     let mut errors = Vec::new();
//     let dir_path = Path::new("project_graph_data/collaborator_files_address_book");

//     for entry in fs::read_dir(dir_path)? {
//         let entry = entry?;
//         let path = entry.path();

//         if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("toml") {
//             let toml_string = fs::read_to_string(&path)?;

//             match toml::from_str::<Value>(&toml_string) {
//                 Ok(toml_value) => {
//                     if let Value::Table(table) = toml_value {
//                         let user_name = if let Some(Value::String(s)) = table.get("user_name") {
//                             s.clone()
//                         } else {
//                             errors.push(YOURPROGRAMError::TomlError("Missing user_name".into()));
//                             continue;
//                         };

//                         let user_salt_list = if let Some(Value::Array(arr)) = table.get("user_salt_list") {
//                             arr.iter()
//                                 .map(|val| {
//                                     if let Value::String(s) = val {
//                                         // Parse the string as u128 (handle potential errors)
//                                         u128::from_str_radix(s.trim_start_matches("0x"), 16)
//                                             .map_err(|e| YOURPROGRAMError::ParseIntError(e))
//                                     } else {
//                                         Err(YOURPROGRAMError::TomlError("Invalid salt format: Expected string".into()))
//                                     }
//                                 })
//                                 .collect::<Result<Vec<u128>, YOURPROGRAMError>>()?
//                         } else {
//                             errors.push(YOURPROGRAMError::TomlError("Missing user_salt_list".into()));
//                             continue;
//                         };

//                         collaborators.push(CollaboratorTomlData { user_name, user_salt_list });
//                     } else {
//                         errors.push(YOURPROGRAMError::TomlError("Invalid TOML structure".into()));
//                     }
//                 }
//                 // Err(e) => errors.push(YOURPROGRAMError::TomlError(e.to_string())), // Handle TOML parsing errors
//                 Err(e) => {
//                     match e {
//                         YOURPROGRAMError::IoError(io_err) => println!("IO Error: {}", io_err), 
//                         YOURPROGRAMError::TomlError(toml_err) => println!("TOML Error: {}", toml_err),
//                         YOURPROGRAMError::ParseIntError(parse_err) => println!("Parse Error: {}", parse_err),
//                     }
//                 }           
//             }
//         }
//     }

//     Ok((collaborators, errors))
// }

// fn main() {
//     match read_a_collaborator_setup_toml() {
//         Ok((collaborators, errors)) => {
//             if !errors.is_empty() {
//                 println!("Errors encountered:");
//                 for err in errors {
//                     println!("{:?}", err);
//                 }
//             }

//             println!("Collaborators:");
//             for collaborator in collaborators {
//                 println!("{:?}", collaborator);
//             }
//         }
//         Err(e) => { 
//             // Convert the toml::de::Error to your custom error type
//             errors.push(YOURPROGRAMError::TomlError(e.to_string())); 
//         }    
//     }
// }

fn read_a_collaborator_setup_toml() -> Result<(Vec<CollaboratorTomlData>, Vec<YOURPROGRAMError>), YOURPROGRAMError> {
    let mut collaborators = Vec::new();
    let mut errors = Vec::new();
    let dir_path = Path::new("project_graph_data/collaborator_files_address_book");

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("toml") {
            let toml_string = fs::read_to_string(&path)?;

            match toml::from_str::<Value>(&toml_string) {
                Ok(toml_value) => {
                    if let Value::Table(table) = toml_value {
                        let user_name = if let Some(Value::String(s)) = table.get("user_name") {
                            s.clone()
                        } else {
                            errors.push(YOURPROGRAMError::TomlError("Missing user_name".into()));
                            continue;
                        };

                        let user_salt_list = if let Some(Value::Array(arr)) = table.get("user_salt_list") {
                            arr.iter()
                                .map(|val| {
                                    if let Value::String(s) = val {
                                        // Parse the string as u128 (handle potential errors)
                                        u128::from_str_radix(s.trim_start_matches("0x"), 16)
                                            .map_err(|e| YOURPROGRAMError::ParseIntError(e))
                                    } else {
                                        Err(YOURPROGRAMError::TomlError("Invalid salt format: Expected string".into()))
                                    }
                                })
                                .collect::<Result<Vec<u128>, YOURPROGRAMError>>()?
                        } else {
                            errors.push(YOURPROGRAMError::TomlError("Missing user_salt_list".into()));
                            continue;
                        };

                        collaborators.push(CollaboratorTomlData { user_name, user_salt_list });
                    } else {
                        errors.push(YOURPROGRAMError::TomlError("Invalid TOML structure".into()));
                    }
                }
                Err(e) => {
                    // Convert the toml::de::Error to your custom error type
                    errors.push(YOURPROGRAMError::TomlError(e.to_string()));
                }
            }
        }
    }

    Ok((collaborators, errors))
}

fn main() {
    match read_a_collaborator_setup_toml() {
        Ok((collaborators, errors)) => {
            if !errors.is_empty() {
                println!("Errors encountered:");
                for err in errors {
                    println!("{}", err); // Use the Display implementation for printing
                }
            }

            println!("Collaborators:");
            for collaborator in collaborators {
                println!("User Name: {}, Salt List: {:?}", collaborator.user_name, collaborator.user_salt_list);
            }
        }
        Err(e) => {
            println!("Error reading TOML files: {}", e); // Use the Display implementation for printing
        }
    }
}