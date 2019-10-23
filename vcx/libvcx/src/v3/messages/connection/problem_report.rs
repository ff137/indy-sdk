use v3::messages::{MessageType, MessageId, A2AMessage, A2AMessageKinds};
use messages::thread::Thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemReport {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "problem-code")]
    pub problem_code: ProblemCode,
    pub explain: String,
    #[serde(rename = "~l10n")]
    pub localization: Localization,
    #[serde(rename = "~thread")]
    pub thread: Thread
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProblemCode {
    Empty,
    #[serde(rename = "request_not_accepted")]
    RequestNotAccepted,
    #[serde(rename = "request_processing_error")]
    RequestProcessingError,
    #[serde(rename = "response_not_accepted")]
    ResponseNotAccepted,
    #[serde(rename = "response_processing_error")]
    ResponseProcessingError
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Localization {
    locale: Locales
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Locales {
    #[serde(rename = "en")]
    En,
}

impl ProblemReport {
    pub fn create() -> ProblemReport {
        ProblemReport {
            msg_type: MessageType::build(A2AMessageKinds::ConnectionProblemReport),
            id: MessageId::new(),
            problem_code: ProblemCode::Empty,
            explain: String::new(),
            localization: Localization::create(),
            thread: Thread::new(),
        }
    }
}

impl ProblemReport {
    pub fn set_problem_code(mut self, problem_code: ProblemCode) -> ProblemReport {
        self.problem_code = problem_code;
        self
    }

    pub fn set_explain(mut self, explain: String) -> ProblemReport {
        self.explain = explain;
        self
    }

    pub fn set_thread(mut self, thread: Thread) -> ProblemReport {
        self.thread = thread;
        self
    }

    pub fn to_a2a_message(&self) -> A2AMessage {
        A2AMessage::ProblemReport(self.clone()) // TODO: THINK how to avoid clone
    }
}

impl Localization {
    pub fn create() -> Localization {
        Localization { locale: Locales::En }
    }
}