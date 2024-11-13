use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Consensus {
    pub consensus_type: String,
    pub solo_consensus_is_validator: bool,
}

impl Default for Consensus {
    fn default() -> Self {
        Self {
            consensus_type: "".to_string(),
            solo_consensus_is_validator: false,
        }
    }
}

impl Clone for Consensus {
    fn clone(&self) -> Self {
        Self {
            consensus_type: self.consensus_type.clone(),
            solo_consensus_is_validator: self.solo_consensus_is_validator,
        }
    }
}
