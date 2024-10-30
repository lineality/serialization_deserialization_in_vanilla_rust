use std::fmt;
use std::fs::File;
use std::io::Write;
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


/// Serialize struct to .toml file
/// Serializes a `CollaboratorTomlData` struct into a TOML-formatted string.
///
/// This function takes a `CollaboratorTomlData` struct and manually constructs 
/// a TOML-formatted string representation of the data. 
///
/// # No `serde` Crate
///
/// This function implements TOML serialization *without* using the `serde` 
/// crate. It manually formats each field of the `CollaboratorTomlData` struct 
/// into the TOML syntax.
///
/// This approach is taken to avoid the dependency on the `serde` crate 
/// while still providing a way to generate TOML output.
///
/// # TOML Format
///
/// The function generates a TOML string with the following structure:
///
/// ```toml
/// user_name = "value"
/// user_salt_list = [
///     "0xhex_value",
///     "0xhex_value",
///     ...
/// ]
/// ipv4_addresses = [
///     "ip_address",
///     "ip_address",
///     ...
/// ]
/// ipv6_addresses = [
///     "ip_address",
///     "ip_address",
///     ...
/// ]
/// gpg_key_public = "value"
/// sync_interval = value
/// updated_at_timestamp = value
/// ```
///
/// # Helper Function
///
/// The `serialize_ip_addresses` helper function is used to format the 
/// `ipv4_addresses` and `ipv6_addresses` fields into TOML array syntax.
///
/// # Parameters
///
/// - `collaborator`: A reference to the `CollaboratorTomlData` struct to be serialized.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok`: The TOML-formatted string representation of the `CollaboratorTomlData`.
/// - `Err`: A `ThisProjectError` if an error occurs during serialization (although 
///           errors are unlikely in this simplified implementation). 
/// 
/// # use with
/// // Serialize the collaborator data to a TOML string
/// match serialize_collaborator_to_toml(&collaborator) {
///     Ok(toml_string) => {
///         println!("Serialized TOML:\n{}", toml_string);
///
///         // Write the TOML string to a file (example file path)
///         match write_toml_to_file("collaborator_data.toml", &toml_string) {
///             Ok(_) => println!("TOML data written to file successfully."),
///             Err(e) => println!("Error writing to file: {}", e),
///         }
///     }
///     Err(e) => println!("Error serializing to TOML: {}", e),
/// }
fn serialize_collaborator_to_toml(collaborator: &CollaboratorTomlData) -> Result<String, ThisProjectError> {
    let mut toml_string = String::new();

    // Add user_name
    toml_string.push_str(&format!("user_name = \"{}\"\n", collaborator.user_name));

    // Add user_salt_list
    toml_string.push_str("user_salt_list = [\n");
    for salt in &collaborator.user_salt_list {
        toml_string.push_str(&format!("    \"0x{:x}\",\n", salt));
    }
    toml_string.push_str("]\n");

    // Add ipv4_addresses
    serialize_ip_addresses(&mut toml_string, "ipv4_addresses", &collaborator.ipv4_addresses)?;

    // Add ipv6_addresses
    serialize_ip_addresses(&mut toml_string, "ipv6_addresses", &collaborator.ipv6_addresses)?;

    // Add gpg_key_public
    toml_string.push_str(&format!("gpg_key_public = \"{}\"\n", collaborator.gpg_key_public));

    // Add sync_interval
    toml_string.push_str(&format!("sync_interval = {}\n", collaborator.sync_interval));

    // Add updated_at_timestamp
    toml_string.push_str(&format!("updated_at_timestamp = {}\n", collaborator.updated_at_timestamp));

    Ok(toml_string)
}

// Helper function to serialize IP addresses to TOML array format
fn serialize_ip_addresses<T: std::fmt::Display>(
    toml_string: &mut String, 
    key: &str, 
    addresses: &Option<Vec<T>>
) -> Result<(), ThisProjectError> {
    if let Some(addr_vec) = addresses {
        toml_string.push_str(&format!("{} = [\n", key));
        for addr in addr_vec {
            toml_string.push_str(&format!("    \"{}\",\n", addr));
        }
        toml_string.push_str("]\n");
    }
    Ok(()) // Return Ok(()) if the addresses field is None
}

// Function to write a TOML string to a file
// Function to write a TOML string to a file
fn write_toml_to_file(file_path: &str, toml_string: &str) -> Result<(), ThisProjectError> {
    // Attempt to create the file. 
    let mut file = match File::create(file_path) {
        Ok(file) => file,
        Err(e) => return Err(ThisProjectError::IoError(e)), 
    };

    // Attempt to write to the file.
    if let Err(e) = file.write_all(toml_string.as_bytes()) {
        return Err(ThisProjectError::IoError(e));
    }

    // Everything successful!
    Ok(()) 
}
// fn write_toml_to_file(file_path: &str, toml_string: &str) -> IoResult<()> {
//     let mut file = File::create(file_path)?;
//     file.write_all(toml_string.as_bytes())?;
//     Ok(())
// }

fn main() {
    // Example CollaboratorTomlData instance
    let collaborator = CollaboratorTomlData {
        user_name: "Bob".to_string(),
        user_salt_list: vec![0x123456789abcdef0, 0xabcdef0123456789],
        ipv4_addresses: Some(vec![Ipv4Addr::new(192, 168, 1, 1), Ipv4Addr::new(10, 0, 0, 1)]),
        ipv6_addresses: Some(vec![Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1), Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)]),
        gpg_key_public: "-----BEGIN PGP PUBLIC KEY BLOCK----- ...".to_string(),
        sync_interval: 300,
        updated_at_timestamp: 1728308000,
    };

    // Serialize the collaborator data to a TOML string
    match serialize_collaborator_to_toml(&collaborator) {
        Ok(toml_string) => {
            println!("Serialized TOML:\n{}", toml_string);

            // Write the TOML string to a file (example file path)
            match write_toml_to_file("collaborator_data.toml", &toml_string) {
                Ok(_) => println!("TOML data written to file successfully."),
                Err(e) => println!("Error writing to file: {}", e),
            }
        }
        Err(e) => println!("Error serializing to TOML: {}", e),
    }
}
