use std::fmt;
use std::fs;
use std::path::Path;
use std::ffi::OsStr;
use toml::Value;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug)]
struct CollaboratorTomlData {
    user_name: String,
    user_salt_list: Vec<u128>,
    ipv4_addresses: Option<Vec<Ipv4Addr>>,
    ipv6_addresses: Option<Vec<Ipv6Addr>>,
    gpg_key_public: String,
    sync_interval: u64,
    updated_at_timestamp: u64,
}

#[derive(Debug)]
enum ThisProjectError {
    IoError(std::io::Error),
    TomlError(String),
    ParseIntError(std::num::ParseIntError),
}

impl From<std::io::Error> for ThisProjectError {
    fn from(err: std::io::Error) -> Self {
        ThisProjectError::IoError(err)
    }
}

impl From<std::num::ParseIntError> for ThisProjectError {
    fn from(err: std::num::ParseIntError) -> Self {
        ThisProjectError::ParseIntError(err)
    }
}

impl fmt::Display for ThisProjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ThisProjectError::IoError(err) => write!(f, "IO Error: {}", err),
            ThisProjectError::TomlError(err) => write!(f, "TOML Error: {}", err),
            ThisProjectError::ParseIntError(err) => write!(f, "Parse Int Error: {}", err),
        }
    }
}

/// Toml Deserialization: Reads collaborator setup data from TOML files in a specified directory.
///
/// # Requires: 
/// the toml crate (use a current version)
/// 
/// [dependencies]
/// toml = "0.8"
/// 
/// # Terms:
/// Serialization: The process of converting a data structure (like your CollaboratorTomlData struct) into a textual representation (like a TOML file).
/// 
/// Deserialization: The process of converting a textual representation (like a TOML file) into a data structure (like your CollaboratorTomlData struct).
/// 
/// This function reads and parses TOML files located in the directory 
/// `project_graph_data/collaborator_files_address_book`. Each file is expected to 
/// contain data for a single collaborator in a structure that can be mapped to 
/// the `CollaboratorTomlData` struct.
///
/// # No `serde` Crate
///
/// This function implements TOML parsing *without* using the `serde` crate. 
/// It manually extracts values from the TOML data using the `toml` crate's 
/// `Value` enum and pattern matching. 
///
/// This approach is taken to avoid the dependency on the `serde` crate 
/// while still providing a way to parse TOML files.
///
/// # Data Extraction
///
/// The function extracts the following fields from each TOML file:
///
/// - `user_name` (String)
/// - `user_salt_list` (Vec<u128>): Stored as hexadecimal strings in the TOML file.
/// - `ipv4_addresses` (Option<Vec<Ipv4Addr>>): Stored as strings in the TOML file.
/// - `ipv6_addresses` (Option<Vec<Ipv6Addr>>): Stored as strings in the TOML file.
/// - `gpg_key_public` (String)
/// - `sync_interval` (u64)
/// - `updated_at_timestamp` (u64)
///
/// # Helper Functions
///
/// The following helper functions are used to extract and parse specific data types:
///
/// - `extract_ipv4_addresses`: Parses a string array into `Option<Vec<Ipv4Addr>>`.
/// - `extract_ipv6_addresses`: Parses a string array into `Option<Vec<Ipv6Addr>>`.
/// - `extract_u64`: Parses a TOML integer into a `u64` value, handling potential errors.
///
/// # Error Handling
///
/// The function returns a `Result` type to handle potential errors during file 
/// reading, TOML parsing, and data extraction. The `ThisProjectError` enum is used to 
/// represent different error types.
///
/// # Example TOML File
///
/// ```toml
/// user_name = "Alice"
/// user_salt_list = ["0x11111111111111111111111111111111", "0x11111111111111111111111111111112"]
/// ipv4_addresses = ["192.168.1.1", "10.0.0.1"]
/// ipv6_addresses = ["fe80::1", "::1"]
/// gpg_key_public = "-----BEGIN PGP PUBLIC KEY BLOCK----- ..."
/// sync_interval = 60
/// updated_at_timestamp = 1728307160
/// ```
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok`: A tuple with:
///     - A vector of successfully parsed `CollaboratorTomlData` instances.
///     - A vector of any `ThisProjectError` encountered during parsing.
/// - `Err`: A `ThisProjectError` if there was an error reading the directory or any file.
/// 
/// This was developed for the UMA project, as the naming reflects:
/// https://github.com/lineality/uma_productivity_collaboration_tool
fn read_a_collaborator_setup_toml() -> Result<(Vec<CollaboratorTomlData>, Vec<ThisProjectError>), ThisProjectError> {
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
                        // Extract user_name
                        let user_name = if let Some(Value::String(s)) = table.get("user_name") {
                            s.clone()
                        } else {
                            errors.push(ThisProjectError::TomlError("Missing user_name".into()));
                            continue;
                        };

                        // Extract user_salt_list
                        let user_salt_list = if let Some(Value::Array(arr)) = table.get("user_salt_list") {
                            arr.iter()
                                .map(|val| {
                                    if let Value::String(s) = val {
                                        u128::from_str_radix(s.trim_start_matches("0x"), 16)
                                            .map_err(|e| ThisProjectError::ParseIntError(e))
                                    } else {
                                        Err(ThisProjectError::TomlError("Invalid salt format: Expected string".into()))
                                    }
                                })
                                .collect::<Result<Vec<u128>, ThisProjectError>>()?
                        } else {
                            errors.push(ThisProjectError::TomlError("Missing user_salt_list".into()));
                            continue;
                        };

                        // Extract ipv4_addresses
                        let ipv4_addresses = extract_ipv4_addresses(&table, "ipv4_addresses", &mut errors)?;

                        // Extract ipv6_addresses
                        let ipv6_addresses = extract_ipv6_addresses(&table, "ipv6_addresses", &mut errors)?;

                        // Extract gpg_key_public
                        let gpg_key_public = if let Some(Value::String(s)) = table.get("gpg_key_public") {
                            s.clone()
                        } else {
                            errors.push(ThisProjectError::TomlError("Missing or invalid gpg_key_public".into()));
                            continue;
                        };

                        // Extract sync_interval
                        let sync_interval = extract_u64(&table, "sync_interval", &mut errors)?;

                        // Extract updated_at_timestamp
                        let updated_at_timestamp = extract_u64(&table, "updated_at_timestamp", &mut errors)?;

                        // Create CollaboratorTomlData instance
                        collaborators.push(CollaboratorTomlData {
                            user_name,
                            user_salt_list,
                            ipv4_addresses,
                            ipv6_addresses,
                            gpg_key_public,
                            sync_interval,
                            updated_at_timestamp,
                        });
                    } else {
                        errors.push(ThisProjectError::TomlError("Invalid TOML structure".into()));
                    }
                }
                Err(e) => {
                    errors.push(ThisProjectError::TomlError(e.to_string()));
                }
            }
        }
    }

    Ok((collaborators, errors))
}

// // Helper function to extract and parse IPv4 addresses from a toml::Value::Table
// fn extract_ipv4_addresses(table: &toml::map::Map<String, Value>, key: &str, errors: &mut Vec<ThisProjectError>) -> Result<Option<Vec<Ipv4Addr>>, ThisProjectError> {
//     if let Some(Value::Array(arr)) = table.get(key) {
//         let addresses = arr.iter()
//             .map(|val| {
//                 if let Value::String(s) = val {
//                     s.parse::<Ipv4Addr>()
//                         .map_err(|e| ThisProjectError::TomlError(format!("Invalid {} format: {}", key, e)))
//                 } else {
//                     Err(ThisProjectError::TomlError(format!("Invalid {} format: Expected string", key)))
//                 }
//             })
//             .collect::<Result<Vec<Ipv4Addr>, ThisProjectError>>()?;
//         Ok(Some(addresses))
//     } else {
//         Ok(None) 
//     }
// }
// Helper function to extract and parse IPv4 addresses from a toml::Value::Table
fn extract_ipv4_addresses(
    table: &toml::map::Map<String, Value>, 
    key: &str, 
    errors: &mut Vec<ThisProjectError>
) -> Result<Option<Vec<Ipv4Addr>>, ThisProjectError> {
    if let Some(Value::Array(arr)) = table.get(key) {
        let mut addresses = Vec::new(); // Create an empty vector to store addresses
        for val in arr {
            if let Value::String(s) = val {
                match s.parse::<Ipv4Addr>() {
                    Ok(ip) => addresses.push(ip), // Push successful IP address
                    Err(e) => errors.push(ThisProjectError::TomlError(format!("Invalid {} format: {}. Skipping this address.", key, e))),
                }
            } else {
                errors.push(ThisProjectError::TomlError(format!("Invalid {} format: Expected string. Skipping this address.", key)));
            }
        }

        if addresses.is_empty() { // If no valid addresses were found
            Ok(None)
        } else {
            Ok(Some(addresses))
        }
    } else {
        Ok(None) // Return None if the key is not present 
    }
}

// // Helper function to extract and parse IPv6 addresses from a toml::Value::Table
// fn extract_ipv6_addresses(table: &toml::map::Map<String, Value>, key: &str, errors: &mut Vec<ThisProjectError>) -> Result<Option<Vec<Ipv6Addr>>, ThisProjectError> {
//     if let Some(Value::Array(arr)) = table.get(key) {
//         let addresses = arr.iter()
//             .map(|val| {
//                 if let Value::String(s) = val {
//                     s.parse::<Ipv6Addr>()
//                         .map_err(|e| ThisProjectError::TomlError(format!("Invalid {} format: {}", key, e)))
//                 } else {
//                     Err(ThisProjectError::TomlError(format!("Invalid {} format: Expected string", key)))
//                 }
//             })
//             .collect::<Result<Vec<Ipv6Addr>, ThisProjectError>>()?;
//         Ok(Some(addresses))
//     } else {
//         Ok(None)
//     }
// }
// Helper function to extract and parse IPv6 addresses from a toml::Value::Table
fn extract_ipv6_addresses(table: &toml::map::Map<String, Value>, key: &str, errors: &mut Vec<ThisProjectError>) -> Result<Option<Vec<Ipv6Addr>>, ThisProjectError> {
    if let Some(Value::Array(arr)) = table.get(key) {
        let mut addresses = Vec::new(); // Create an empty vector to store addresses
        for val in arr {
            if let Value::String(s) = val {
                match s.parse::<Ipv6Addr>() {
                    Ok(ip) => addresses.push(ip), // Push successful IP address
                    Err(e) => errors.push(ThisProjectError::TomlError(format!("Invalid {} format: {}. Skipping this address.", key, e))),
                }
            } else {
                errors.push(ThisProjectError::TomlError(format!("Invalid {} format: Expected string. Skipping this address.", key)));
            }
        }

        if addresses.is_empty() { // If no valid addresses were found
            Ok(None) 
        } else {
            Ok(Some(addresses)) 
        }
    } else {
        Ok(None) // Return None if the key is not present
    }
}

// Helper function to extract a u64 from a toml::Value::Table
/// Extracts a `u64` value from a `toml::Value::Table` for a given key.
///
/// This helper function attempts to extract a `u64` value associated with the 
/// specified `key` from a `toml::map::Map` (representing a TOML table). It 
/// handles cases where the key is missing, the value is not an integer, or 
/// the integer value is outside the valid range for a `u64`.
///
/// # Parameters
///
/// - `table`: A reference to the `toml::map::Map` (TOML table) from which to extract the value.
/// - `key`: The key (as a string slice) associated with the value to extract.
/// - `errors`: A mutable reference to a vector of `ThisProjectError` to collect any errors encountered during extraction.
///
/// # Error Handling
///
/// The function uses a `Result` type to handle potential errors. It returns:
///
/// - `Ok(u64)`: If the key is found and the value can be successfully parsed as a `u64`.
/// - `Err(ThisProjectError)`: If:
///     - The key is missing from the table.
///     - The value associated with the key is not a `toml::Value::Integer`.
///     - The integer value is negative or exceeds the maximum value of a `u64`.
///
/// In case of errors, a descriptive error message is added to the `errors` vector.
///
/// # Example
///
/// ```rust
/// use toml::Value;
///
/// let mut errors = Vec::new();
/// let mut table = toml::map::Map::new();
/// table.insert("my_key".to_string(), Value::Integer(12345));
///
/// let my_value = extract_u64(&table, "my_key", &mut errors);
///
/// assert_eq!(my_value.unwrap(), 12345);
/// assert!(errors.is_empty()); // No errors
/// ```
fn extract_u64(table: &toml::map::Map<String, Value>, key: &str, errors: &mut Vec<ThisProjectError>) -> Result<u64, ThisProjectError> {
    if let Some(Value::Integer(i)) = table.get(key) {
        // Correct comparison for u64 values:
        if *i >= 0 && *i <= i64::MAX { // Compare against i64::MAX 
            Ok(*i as u64) // Safe to cast since it's within i64::MAX
        } else {
            errors.push(ThisProjectError::TomlError(format!("Invalid {}: Out of range for u64", key)));
            Err(ThisProjectError::TomlError(format!("Invalid {}: Out of range for u64", key)))
        }
    } else {
        errors.push(ThisProjectError::TomlError(format!("Missing or invalid {}", key)));
        Err(ThisProjectError::TomlError(format!("Missing or invalid {}", key)))
    }
}

fn main() {
    match read_a_collaborator_setup_toml() {
        Ok((collaborators, errors)) => {
            if !errors.is_empty() {
                println!("Errors encountered:");
                for err in errors {
                    println!("{}", err); 
                }
            }

            println!("Collaborators:");
            for collaborator in collaborators {
                println!("{:?}", collaborator); 
            }
        }
        Err(e) => {
            println!("Error reading TOML files: {}", e); 
        }
    }
}
