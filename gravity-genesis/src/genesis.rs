use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GenesisConfig {
    #[serde(rename = "validatorAddresses")]
    pub validator_addresses: Vec<String>,
    #[serde(rename = "consensusPublicKeys")]
    pub consensus_public_keys: Vec<String>,
    #[serde(rename = "votingPowers")]
    pub voting_powers: Vec<String>,
    #[serde(rename = "validatorNetworkAddresses")]
    pub validator_network_addresses: Vec<String>,
    #[serde(rename = "fullnodeNetworkAddresses")]
    pub fullnode_network_addresses: Vec<String>,
}