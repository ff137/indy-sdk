use v3::messages::{MessageId, MessageType, A2AMessageKinds};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PresentationProposal {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub comment: String,
    pub presentation_proposal: PresentationPreview
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PresentationPreview {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    pub attributes: Vec<Attribute>,
    pub predicates: Vec<Predicate>
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub cred_def_id: Option<String>,
    #[serde(rename = "mime-type")]
    pub mime_type: Option<String>, // TODO: FIXME
    pub value: Option<String> // TODO: FIXME
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Predicate {
    pub name: String,
    pub cred_def_id: Option<String>,
    pub predicate: PredicateType,
    pub threshold: i64,
    pub filter: Vec<::serde_json::Value>
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PredicateType {
    #[serde(rename = ">=")]
    GE,
    #[serde(rename = "<=")]
    LE,
    #[serde(rename = ">")]
    GT,
    #[serde(rename = "<")]
    LT
}

impl PresentationProposal {
    pub fn create() -> Self {
        PresentationProposal::default()
    }
}

impl Default for PresentationProposal {
    fn default() -> PresentationProposal {
        PresentationProposal {
            msg_type: MessageType::build(A2AMessageKinds::PresentationProposal),
            id: MessageId::new(),
            comment: String::new(),
            presentation_proposal: PresentationPreview::default(),
        }
    }
}

impl Default for PresentationPreview {
    fn default() -> PresentationPreview {
        PresentationPreview {
            msg_type: MessageType::build(A2AMessageKinds::PresentationPreview),
            attributes: Vec::new(),
            predicates: Vec::new(),
        }
    }
}
