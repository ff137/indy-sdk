use domain::a2a::MessageType;
use domain::status::MessageStatusCode;

use utils::rand::rand_string;

#[derive(Debug, Clone)]
pub struct InternalMessage {
    pub uid: String,
    pub _type: MessageType,
    pub sender_did: String,
    pub status_code: MessageStatusCode,
    pub ref_msg_id: Option<String>,
    pub topic_id: Option<String>,
    pub seq_no: Option<String>,
    pub payload: Option<Vec<u8>>,
}

impl InternalMessage {
    pub fn new(uid: Option<&str>,
               mtype: &MessageType,
               status_code: MessageStatusCode,
               sender_did: &str,
               payload: Option<Vec<u8>>,
    ) -> InternalMessage {
        trace!("InternalMessage::new >> {:?}, {:?}, {:?}, {:?}", uid, mtype, status_code, sender_did);

        let uid = uid.map(String::from).unwrap_or(rand_string(10));

        InternalMessage {
            uid,
            _type: mtype.clone(),
            sender_did: sender_did.to_string(),
            status_code,
            ref_msg_id: None,
            topic_id: None,
            seq_no: None,
            payload,
        }
    }
}



