use std::fmt;
use std::fs;
use std::path::Path;
use toml::Value;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::num::ParseIntError;


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
    TomlVanillaDeserialStrError(String), // use without serede crate (good)
    ParseIntError(ParseIntError),
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
            ThisProjectError::TomlVanillaDeserialStrError(err) => write!(f, "TOML Error: {}", err),
            ThisProjectError::ParseIntError(err) => write!(f, "Parse Int Error: {}", err),
        }
    }
}

/// Vanilla-Rust File Deserialization
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
/// The function extracts the following fields from one TOML file:
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
/// 
/// # Use with:
/// // Specify the username of the collaborator to read
/// let username = "alice";
///
/// /// Read the collaborator data from the TOML file
/// match read_one_collaborator_setup_toml(username) {
///     Ok(collaborator) => {
///         // Print the collaborator data
///         println!("Collaborator Data for {}:", username);
///         println!("{:#?}", collaborator); /// Use {:#?} for pretty-printing
///     }
///     Err(e) => {
///         // Print an error message if there was an error reading or parsing the TOML file
///         println!("Error reading collaborator data for {}: {}", username, e);
///     }
/// }
fn read_one_collaborator_setup_toml(collaborator_name: &str) -> Result<CollaboratorTomlData, ThisProjectError> {

    // 1. Construct File Path
    let file_path = Path::new("project_graph_data/collaborator_files_address_book")
        .join(format!("{}__collaborator.toml", collaborator_name));

    // 2. Read TOML File
    let toml_string = fs::read_to_string(&file_path)?; 

    // 3. Parse TOML Data
    // 3. Parse TOML Data (handle potential toml::de::Error)
    let toml_value = match toml::from_str::<Value>(&toml_string) {
        Ok(value) => value,
        Err(e) => return Err(ThisProjectError::TomlVanillaDeserialStrError(e.to_string())), 
    };

    // 4. Extract Data from TOML Value (similar to your previous code)
    if let Value::Table(table) = toml_value {

        // Extract user_name
        let user_name = if let Some(Value::String(s)) = table.get("user_name") {
            s.clone()
        } else {
            return Err(ThisProjectError::TomlVanillaDeserialStrError("Missing user_name".into()));
        };

        // Extract user_salt_list
        let user_salt_list = if let Some(Value::Array(arr)) = table.get("user_salt_list") {
            arr.iter()
                .map(|val| {
                    if let Value::String(s) = val {
                        u128::from_str_radix(s.trim_start_matches("0x"), 16)
                            .map_err(|e| ThisProjectError::ParseIntError(e))
                    } else {
                        Err(ThisProjectError::TomlVanillaDeserialStrError("Invalid salt format: Expected string".into()))
                    }
                })
                .collect::<Result<Vec<u128>, ThisProjectError>>()?
        } else {
            return Err(ThisProjectError::TomlVanillaDeserialStrError("Missing user_salt_list".into()));
        };

        // Extract ipv4_addresses
        let ipv4_addresses = extract_ipv4_addresses(&table, "ipv4_addresses")?;

        // Extract ipv6_addresses
        let ipv6_addresses = extract_ipv6_addresses(&table, "ipv6_addresses")?;

        // Extract gpg_key_public
        let gpg_key_public = if let Some(Value::String(s)) = table.get("gpg_key_public") {
            s.clone()
        } else {
            return Err(ThisProjectError::TomlVanillaDeserialStrError("Missing or invalid gpg_key_public".into()));
        };

        // Extract sync_interval
        let sync_interval = extract_u64(&table, "sync_interval")?;

        // Extract updated_at_timestamp
        let updated_at_timestamp = extract_u64(&table, "updated_at_timestamp")?;

        // 5. Return CollaboratorTomlData 
        Ok(CollaboratorTomlData {
            user_name,
            user_salt_list,
            ipv4_addresses,
            ipv6_addresses,
            gpg_key_public,
            sync_interval,
            updated_at_timestamp,
        })
    } else {
        Err(ThisProjectError::TomlVanillaDeserialStrError("Invalid TOML structure: Expected a table".into()))
    }
}

fn extract_ipv4_addresses(table: &toml::map::Map<String, Value>, key: &str) -> Result<Option<Vec<Ipv4Addr>>, ThisProjectError> {
    if let Some(Value::Array(arr)) = table.get(key) {
        let mut addresses = Vec::new();
        for val in arr {
            if let Value::String(s) = val {
                match s.parse::<Ipv4Addr>() {
                    Ok(ip) => addresses.push(ip),
                    Err(e) => return Err(ThisProjectError::TomlVanillaDeserialStrError(format!("Invalid {} format: {}. Skipping this address.", key, e))), 
                }
            } else {
                return Err(ThisProjectError::TomlVanillaDeserialStrError(format!("Invalid {} format: Expected string. Skipping this address.", key)));
            }
        }

        if addresses.is_empty() { 
            Ok(None)
        } else {
            Ok(Some(addresses))
        }
    } else {
        Ok(None) 
    }
}

fn extract_ipv6_addresses(table: &toml::map::Map<String, Value>, key: &str) -> Result<Option<Vec<Ipv6Addr>>, ThisProjectError> {
    if let Some(Value::Array(arr)) = table.get(key) {
        let mut addresses = Vec::new();
        for val in arr {
            if let Value::String(s) = val {
                match s.parse::<Ipv6Addr>() {
                    Ok(ip) => addresses.push(ip),
                    Err(e) => return Err(ThisProjectError::TomlVanillaDeserialStrError(format!("Invalid {} format: {}. Skipping this address.", key, e))), 
                }
            } else {
                return Err(ThisProjectError::TomlVanillaDeserialStrError(format!("Invalid {} format: Expected string. Skipping this address.", key)));
            }
        }

        if addresses.is_empty() { 
            Ok(None)
        } else {
            Ok(Some(addresses))
        }
    } else {
        Ok(None) 
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
// Helper function to extract a u64 from a toml::Value::Table
fn extract_u64(table: &toml::map::Map<String, Value>, key: &str) -> Result<u64, ThisProjectError> {
    if let Some(Value::Integer(i)) = table.get(key) {
        if *i >= 0 && *i <= i64::MAX {
            Ok(*i as u64) 
        } else {
            Err(ThisProjectError::TomlVanillaDeserialStrError(format!("Invalid {}: Out of range for u64", key)))
        }
    } else {
        Err(ThisProjectError::TomlVanillaDeserialStrError(format!("Missing or invalid {}", key)))
    }
}


/// new version using ONE USER
fn main() {
    // Specify the username of the collaborator to read
    let username = "alice";

    // Read the collaborator data from the TOML file
    match read_one_collaborator_setup_toml(username) {
        Ok(collaborator) => {
            // Print the collaborator data
            println!("Collaborator Data for {}:", username);
            println!("{:#?}", collaborator); // Use {:#?} for pretty-printing
        }
        Err(e) => {
            // Print an error message if there was an error reading or parsing the TOML file
            println!("Error reading collaborator data for {}: {}", username, e);
        }
    }
}
