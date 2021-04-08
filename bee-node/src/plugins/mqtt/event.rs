use serde::Serialize;

#[derive(Serialize)]
pub struct MilestonePayload {
    pub index: u32,
    pub timestamp: u64,
}

pub struct Message {
    pub bytes: Vec<u8>,
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        self.bytes
    }
}

#[derive(Serialize)]
pub struct MessageMetadata {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "parentMessageIds")]
    pub parent_message_ids: Vec<String>,
    #[serde(rename = "isSolid")]
    pub is_solid: bool,
    #[serde(rename = "referencedByMilestoneIndex")]
    pub referenced_by_milestone_index: u32,
    #[serde(rename = "ledgerInclusionState")]
    pub ledger_inclusion_state: LedgerInclusionState,
    #[serde(rename = "shouldPromote")]
    pub should_promote: bool,
    #[serde(rename = "shouldReattach")]
    pub should_reattach: bool,
}

pub enum LedgerInclusionState {
    NoTransaction,
    Conflicting,
    Included,
}

impl Serialize for LedgerInclusionState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use LedgerInclusionState::*;

        let s = match *self {
            NoTransaction => "noTransaction",
            Conflicting => "conflicting",
            Included => "included",
        };
        serializer.serialize_str(s)
    }
}
