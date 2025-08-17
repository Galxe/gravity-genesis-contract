use alloy_sol_macro::sol;
use alloy_sol_types::{SolCall, SolValue};
use revm::{
    db::BundleState,
    primitives::{Env, SpecId, TxEnv},
};
use revm_primitives::hex;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::utils::{JWK_MANAGER_ADDR, execute_revm_sequential, new_system_call_txn};

// JSON structures for deserialization
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonJWK {
    pub variant: u8,
    pub data: String, // hex string
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonProviderJWKs {
    pub issuer: String,
    pub version: u64,
    pub jwks: Vec<JsonJWK>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonAllProvidersJWKs {
    pub entries: Vec<JsonProviderJWKs>,
}

sol! {
    struct JWK {
        uint8 variant; // 0: RSA_JWK, 1: UnsupportedJWK
        bytes data; // Encoded JWK data
    }

    /// @dev Provider's JWK collection
    struct ProviderJWKs {
        string issuer; // Issuer
        uint64 version; // Version number
        JWK[] jwks; // JWK array, sorted by kid
    }

    /// @dev All providers' JWK collection
    struct AllProvidersJWKs {
        ProviderJWKs[] entries; // Provider array sorted by issuer
    }

    function upsertObservedJWKs(ProviderJWKs[] calldata providerJWKsArray) external;
    function getObservedJWKs() external view returns (AllProvidersJWKs memory);
}

/// Create a test RSA JWK
pub fn create_test_rsa_jwk(kid: &str, alg: &str, e: &str, n: &str) -> JWK {
    // Create RSA JWK structure
    let rsa_jwk = RSATestJWK {
        kid: kid.to_string(),
        kty: "RSA".to_string(),
        alg: alg.to_string(),
        e: e.to_string(),
        n: n.to_string(),
    };

    // Encode the RSA JWK
    let encoded_data = rsa_jwk.abi_encode();

    JWK {
        variant: 0, // RSA_JWK
        data: encoded_data.into(),
    }
}

/// Create a test provider JWKs collection
pub fn create_provider_jwks(issuer: &str, version: u64, jwks: Vec<JWK>) -> ProviderJWKs {
    ProviderJWKs {
        issuer: issuer.to_string(),
        version,
        jwks,
    }
}

/// Call upsertObservedJWKs function
pub fn call_upsert_observed_jwks(provider_jwks_array: Vec<ProviderJWKs>) -> TxEnv {
    let call_data = upsertObservedJWKsCall {
        providerJWKsArray: provider_jwks_array,
    }
    .abi_encode();
    new_system_call_txn(JWK_MANAGER_ADDR, call_data.into())
}

/// Call getObservedJWKs function
pub fn call_get_observed_jwks() -> TxEnv {
    let call_data = getObservedJWKsCall {}.abi_encode();
    new_system_call_txn(JWK_MANAGER_ADDR, call_data.into())
}

pub fn upsert_observed_jwks(jwks_file_path: &str) -> Result<TxEnv, String> {
    info!("=== Loading JWKs from file: {} ===", jwks_file_path);

    // Read and parse the JSON file
    let jwks_content = std::fs::read_to_string(jwks_file_path)
        .map_err(|e| format!("Failed to read JWKS file: {}", e))?;

    let jwks: JsonAllProvidersJWKs = serde_json::from_str(&jwks_content)
        .map_err(|e| format!("Failed to parse JWKS file: {}", e))?;

    info!("Successfully loaded JWKs from file");
    info!("Total providers: {}", jwks.entries.len());

    for (i, provider) in jwks.entries.iter().enumerate() {
        info!("Provider {}: {}", i + 1, provider.issuer);
        info!("  Version: {}", provider.version);
        info!("  JWK count: {}", provider.jwks.len());

        for (j, jwk) in provider.jwks.iter().enumerate() {
            info!(
                "    JWK {}: variant={}, data_length={}",
                j + 1,
                jwk.variant,
                jwk.data.len()
            );
        }
    }

    // Convert JSON structure to Solidity structure
    let provider_jwks_array: Result<Vec<ProviderJWKs>, String> = jwks
        .entries
        .into_iter()
        .map(|entry| {
            let jwks: Result<Vec<JWK>, String> = entry
                .jwks
                .into_iter()
                .map(|jwk| {
                    // Convert hex string to bytes
                    let data_bytes = if jwk.data.starts_with("0x") {
                        hex::decode(&jwk.data[2..])
                            .map_err(|e| format!("Failed to decode hex data: {}", e))
                    } else {
                        hex::decode(&jwk.data)
                            .map_err(|e| format!("Failed to decode hex data: {}", e))
                    }?;

                    Ok(JWK {
                        variant: jwk.variant,
                        data: data_bytes.into(),
                    })
                })
                .collect();

            Ok(ProviderJWKs {
                issuer: entry.issuer,
                version: entry.version,
                jwks: jwks?,
            })
        })
        .collect();

    let provider_jwks_array = provider_jwks_array?;

    info!("Converted to Solidity structure");
    info!("Provider JWKs array length: {}", provider_jwks_array.len());

    // Create transaction to upsert JWKs
    let upsert_tx = call_upsert_observed_jwks(provider_jwks_array);

    info!("Created upsertObservedJWKs transaction");
    info!("Transaction data length: {} bytes", upsert_tx.data.len());

    // For now, just return success
    // In a real implementation, you would execute this transaction
    info!("JWK upsert transaction prepared successfully");

    Ok(upsert_tx)
}

/// Execute JWK management operations
pub fn execute_jwk_operations<DB>(
    db: DB,
    env: Env,
    bundle_state: Option<BundleState>,
) -> Result<(Vec<alloy_primitives::Log>, BundleState), String>
where
    DB: revm::DatabaseRef + Clone,
{
    info!("=== Starting JWK Management Operations ===");

    // Create transaction to get observed JWKs
    let get_tx = call_get_observed_jwks();

    // Execute get transaction
    info!("Executing getObservedJWKs transaction...");
    let get_result = execute_revm_sequential(db, SpecId::LATEST, env, &[get_tx], bundle_state)
        .map_err(|_| "get transaction failed".to_string())?;

    let (get_results, _) = get_result;

    // Check if get was successful and parse the result
    if let Some(result) = get_results.first() {
        if result.is_success() {
            info!("getObservedJWKs transaction successful");

            // Try to decode the result
            if let Some(output) = result.output() {
                match getObservedJWKsCall::abi_decode_returns(output, false) {
                    Ok(decoded_result) => {
                        info!("=== Retrieved JWKs ===");
                        info!("Total providers: {}", decoded_result._0.entries.len());

                        for (i, provider) in decoded_result._0.entries.iter().enumerate() {
                            info!("Provider {}: {}", i + 1, provider.issuer);
                            info!("  Version: {}", provider.version);
                            info!("  JWK count: {}", provider.jwks.len());

                            for (j, jwk) in provider.jwks.iter().enumerate() {
                                info!(
                                    "    JWK {}: variant={}, data_length={}",
                                    j + 1,
                                    jwk.variant,
                                    jwk.data.len()
                                );
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to decode getObservedJWKs result: {:?}", e);
                        debug!("Raw output: {:?}", output);
                    }
                }
            }
        } else {
            return Err(format!("getObservedJWKs failed: {:?}", result));
        }
    }

    todo!()
}

// Helper struct for RSA JWK encoding
sol! {
    struct RSATestJWK {
        string kid;
        string kty;
        string alg;
        string e;
        string n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwk_creation() {
        let jwk = create_test_rsa_jwk("test-key", "RS256", "AQAB", "test-modulus");
        assert_eq!(jwk.variant, 0);
        assert!(!jwk.data.is_empty());
    }

    #[test]
    fn test_provider_jwks_creation() {
        let jwk = create_test_rsa_jwk("test-key", "RS256", "AQAB", "test-modulus");
        let provider = create_provider_jwks("https://test.com", 1, vec![jwk]);
        assert_eq!(provider.issuer, "https://test.com");
        assert_eq!(provider.version, 1);
        assert_eq!(provider.jwks.len(), 1);
    }

    #[test]
    fn test_json_parsing() {
        // Test JSON parsing with a simple structure
        let json_content = r#"{
            "entries": [
                {
                    "issuer": "https://test.com",
                    "version": 1,
                    "jwks": [
                        {
                            "variant": 1,
                            "data": "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
                        }
                    ]
                }
            ]
        }"#;

        let jwks: JsonAllProvidersJWKs = serde_json::from_str(json_content).unwrap();
        assert_eq!(jwks.entries.len(), 1);
        assert_eq!(jwks.entries[0].issuer, "https://test.com");
        assert_eq!(jwks.entries[0].version, 1);
        assert_eq!(jwks.entries[0].jwks.len(), 1);
        assert_eq!(jwks.entries[0].jwks[0].variant, 1);
        assert_eq!(
            jwks.entries[0].jwks[0].data,
            "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
        );
    }

    #[test]
    fn test_upsert_observed_jwks() {
        // This test would require a real file, so we'll just test the function signature
        // In a real scenario, you would create a temporary file and test with it
        let result = upsert_observed_jwks("nonexistent_file.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read JWKS file"));
    }
}

// Example usage:
//
// ```rust
// use crate::jwks::upsert_observed_jwks;
//
// // Load and process JWKs from JSON file
// upsert_observed_jwks("path/to/jwks_template.json").expect("Failed to process JWKs");
// ```
