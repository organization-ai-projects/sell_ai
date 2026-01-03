use std::fmt;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Low,
    Medium,
    High,
}

impl Tier {
    pub fn from_length(length: usize, thresholds: &[usize; 2]) -> Self {
        if length < thresholds[0] {
            Tier::Low
        } else if length > thresholds[1] {
            Tier::High
        } else {
            Tier::Medium
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Tier::Low => "low",
            Tier::Medium => "medium",
            Tier::High => "high",
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tier_str = match self {
            Tier::Low => "low",
            Tier::Medium => "medium",
            Tier::High => "high",
        };
        write!(f, "{}", tier_str)
    }
}