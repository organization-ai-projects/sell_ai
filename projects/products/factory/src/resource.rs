use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Resource {
    pub id: String,
    pub uri: String,
    pub fetched_at: String,
    pub content: String,
    pub tier: Option<String>, // Ajout d'un champ pour le palier
}

impl Resource {
    pub fn assign_tier(&mut self, thresholds: &[usize]) {
        self.tier = match thresholds {
            [_, _] if self.content.len() < thresholds[0] => Some("low".to_string()),
            [_, _] if self.content.len() > thresholds[1] => Some("high".to_string()),
            _ => Some("medium".to_string()),
        };
    }
}
