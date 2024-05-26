import os
import zipfile
import shutil

# Paths
BEACON_ZIP = "argochain_beacon.zip"
RUST_PROJECT_DIR = "argochain"

# Function to unzip the beacon files
def unzip_beacon(zip_path, extract_to):
    with zipfile.ZipFile(zip_path, "r") as zip_ref:
        zip_ref.extractall(extract_to)

# Function to move files to the Rust project directory
def move_files(src, dest):
    if os.path.exists(dest):
        shutil.rmtree(dest)
    shutil.move(src, dest)

# Function to integrate the necessary parts into the Rust project
def integrate_rust_project():
    # Add dependencies to Cargo.toml
    cargo_toml_path = os.path.join(RUST_PROJECT_DIR, "Cargo.toml")
    dependencies = """
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
"""
    with open(cargo_toml_path, "a") as cargo_toml:
        cargo_toml.write(dependencies)

    # Create a Rust client for interacting with the FastAPI service
    client_code = """
mod discovery_client;

use discovery_client::DiscoveryClient;
use tokio::main;
use std::error::Error;

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = DiscoveryClient::new("http://127.0.0.1:8000");

    // Example usage: setting and getting a value
    client.set_value("my_key", "my_value").await?;
    let value = client.get_value("my_key").await?;
    println!("Retrieved value: {}", value);

    Ok(())
}
"""
    main_rs_path = os.path.join(RUST_PROJECT_DIR, "src", "main.rs")
    with open(main_rs_path, "w") as main_rs:
        main_rs.write(client_code)

    # Create the Rust discovery client module
    discovery_client_code = """
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize)]
struct KeyValue {
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct KeyValueResponse {
    key: String,
    value: String,
}

pub struct DiscoveryClient {
    client: Client,
    base_url: String,
}

impl DiscoveryClient {
    pub fn new(base_url: &str) -> Self {
        DiscoveryClient {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn set_value(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let kv = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        };
        self.client
            .post(&format!("{}/set/", self.base_url))
            .json(&kv)
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_value(&self, key: &str) -> Result<String, Box<dyn Error>> {
        let response = self
            .client
            .get(&format!("{}/get/{}", self.base_url, key))
            .send()
            .await?
            .json::<KeyValueResponse>()
            .await?;
        Ok(response.value)
    }
}
"""
    discovery_client_rs_path = os.path.join(RUST_PROJECT_DIR, "src", "discovery_client.rs")
    with open(discovery_client_rs_path, "w") as discovery_client_rs:
        discovery_client_rs.write(discovery_client_code)

# Main script logic
if __name__ == "__main__":
    # Unzip the beacon files
    unzip_beacon(BEACON_ZIP, "beacon_tmp")

    # Move the extracted files to the Rust project directory
    move_files("beacon_tmp/argochain/app", os.path.join(RUST_PROJECT_DIR, "app"))

    # Integrate the necessary parts into the Rust project
    integrate_rust_project()

    print("Integration complete. The beacon service has been integrated into the Rust project.")
