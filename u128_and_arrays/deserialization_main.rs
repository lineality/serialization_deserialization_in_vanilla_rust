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
enum UmaError {
    IoError(std::io::Error),
    TomlError(String),
    ParseIntError(std::num::ParseIntError),
}

impl From<std::io::Error> for UmaError {
    fn from(err: std::io::Error) -> Self {
        UmaError::IoError(err)
    }
}

impl From<std::num::ParseIntError> for UmaError {
    fn from(err: std::num::ParseIntError) -> Self {
        UmaError::ParseIntError(err)
    }
}

impl fmt::Display for UmaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UmaError::IoError(err) => write!(f, "IO Error: {}", err),
            UmaError::TomlError(err) => write!(f, "TOML Error: {}", err),
            UmaError::ParseIntError(err) => write!(f, "Parse Int Error: {}", err),
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
/// reading, TOML parsing, and data extraction. The `UmaError` enum is used to 
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
///     - A vector of any `UmaError` encountered during parsing.
/// - `Err`: A `UmaError` if there was an error reading the directory or any file.
/// 
/// This was developed for the UMA project, as the naming reflects:
/// https://github.com/lineality/uma_productivity_collaboration_tool
fn read_a_collaborator_setup_toml() -> Result<(Vec<CollaboratorTomlData>, Vec<UmaError>), UmaError> {
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
                            errors.push(UmaError::TomlError("Missing user_name".into()));
                            continue;
                        };

                        // Extract user_salt_list
                        let user_salt_list = if let Some(Value::Array(arr)) = table.get("user_salt_list") {
                            arr.iter()
                                .map(|val| {
                                    if let Value::String(s) = val {
                                        u128::from_str_radix(s.trim_start_matches("0x"), 16)
                                            .map_err(|e| UmaError::ParseIntError(e))
                                    } else {
                                        Err(UmaError::TomlError("Invalid salt format: Expected string".into()))
                                    }
                                })
                                .collect::<Result<Vec<u128>, UmaError>>()?
                        } else {
                            errors.push(UmaError::TomlError("Missing user_salt_list".into()));
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
                            errors.push(UmaError::TomlError("Missing or invalid gpg_key_public".into()));
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
                        errors.push(UmaError::TomlError("Invalid TOML structure".into()));
                    }
                }
                Err(e) => {
                    errors.push(UmaError::TomlError(e.to_string()));
                }
            }
        }
    }

    Ok((collaborators, errors))
}

// Helper function to extract and parse IPv4 addresses from a toml::Value::Table
fn extract_ipv4_addresses(table: &toml::map::Map<String, Value>, key: &str, errors: &mut Vec<UmaError>) -> Result<Option<Vec<Ipv4Addr>>, UmaError> {
    if let Some(Value::Array(arr)) = table.get(key) {
        let addresses = arr.iter()
            .map(|val| {
                if let Value::String(s) = val {
                    s.parse::<Ipv4Addr>()
                        .map_err(|e| UmaError::TomlError(format!("Invalid {} format: {}", key, e)))
                } else {
                    Err(UmaError::TomlError(format!("Invalid {} format: Expected string", key)))
                }
            })
            .collect::<Result<Vec<Ipv4Addr>, UmaError>>()?;
        Ok(Some(addresses))
    } else {
        Ok(None) 
    }
}

// Helper function to extract and parse IPv6 addresses from a toml::Value::Table
fn extract_ipv6_addresses(table: &toml::map::Map<String, Value>, key: &str, errors: &mut Vec<UmaError>) -> Result<Option<Vec<Ipv6Addr>>, UmaError> {
    if let Some(Value::Array(arr)) = table.get(key) {
        let addresses = arr.iter()
            .map(|val| {
                if let Value::String(s) = val {
                    s.parse::<Ipv6Addr>()
                        .map_err(|e| UmaError::TomlError(format!("Invalid {} format: {}", key, e)))
                } else {
                    Err(UmaError::TomlError(format!("Invalid {} format: Expected string", key)))
                }
            })
            .collect::<Result<Vec<Ipv6Addr>, UmaError>>()?;
        Ok(Some(addresses))
    } else {
        Ok(None)
    }
}


// Helper function to extract a u64 from a toml::Value::Table
fn extract_u64(table: &toml::map::Map<String, Value>, key: &str, errors: &mut Vec<UmaError>) -> Result<u64, UmaError> {
    if let Some(Value::Integer(i)) = table.get(key) {
        // Correct comparison for u64 values:
        if *i >= 0 && *i <= i64::MAX { // Compare against i64::MAX 
            Ok(*i as u64) // Safe to cast since it's within i64::MAX
        } else {
            errors.push(UmaError::TomlError(format!("Invalid {}: Out of range for u64", key)));
            Err(UmaError::TomlError(format!("Invalid {}: Out of range for u64", key)))
        }
    } else {
        errors.push(UmaError::TomlError(format!("Missing or invalid {}", key)));
        Err(UmaError::TomlError(format!("Missing or invalid {}", key)))
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
