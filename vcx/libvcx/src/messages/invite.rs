use settings;
use messages::*;
use messages::message_type::MessageTypes;
use utils::{httpclient, error};
use utils::constants::*;

#[derive(Clone, Serialize, Debug, PartialEq)]
enum SendInvitePayloads {
    SendInvitePayloadV0(SendInvitePayloadV0),
    SendInvitePayloadV1(SendInvitePayloadV1)
}

#[derive(Clone, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SendInvitePayloadV0 {
    create_payload: CreateMessage,
    msg_detail_payload: SendInviteMessageDetails,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SendInviteMessageDetails {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "keyDlgProof")]
    key_dlg_proof: KeyDlgProof,
    #[serde(rename = "targetName")]
    target_name: Option<String>,
    #[serde(rename = "phoneNo")]
    phone_no: Option<String>,
    #[serde(rename = "usePublicDID")]
    include_public_did: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SendInvitePayloadV1 {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "sendMsg")]
    send_msg: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,
    #[serde(rename = "replyToMsgId")]
    reply_to_msg_id: Option<String>,
    #[serde(rename = "keyDlgProof")]
    key_dlg_proof: KeyDlgProof,
    #[serde(rename = "targetName")]
    target_name: Option<String>,
    #[serde(rename = "phoneNo")]
    phone_no: Option<String>,
    #[serde(rename = "usePublicDID")]
    include_public_did: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ConnectionRequestResponse {
    #[serde(rename = "@type")]
    pub msg_type: MessageTypes,
    pub uid: String,
    #[serde(rename = "inviteDetail")]
    pub invite_detail: InviteDetail,
    #[serde(rename = "urlToInviteDetail")]
    pub url_to_invite_detail: String,
    pub sent: bool,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
enum AcceptInvitePayloads {
    AcceptInvitePayloadV0(AcceptInvitePayloadV0),
    AcceptInvitePayloadV1(AcceptInvitePayloadV1)
}

#[derive(Clone, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct AcceptInvitePayloadV0 {
    create_payload: CreateMessage,
    msg_detail_payload: AcceptInviteMessageDetails,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AcceptInviteMessageDetails {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "keyDlgProof")]
    key_dlg_proof: KeyDlgProof,
    #[serde(rename = "senderDetail")]
    sender_detail: Option<SenderDetail>,
    #[serde(rename = "senderAgencyDetail")]
    sender_agency_detail: Option<SenderAgencyDetail>,
    #[serde(rename = "answerStatusCode")]
    answer_status_code: Option<MessageStatusCode>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AcceptInvitePayloadV1 {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "sendMsg")]
    send_msg: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,
    #[serde(rename = "replyToMsgId")]
    reply_to_msg_id: Option<String>,
    #[serde(rename = "keyDlgProof")]
    key_dlg_proof: KeyDlgProof,
    #[serde(rename = "senderDetail")]
    sender_detail: Option<SenderDetail>,
    #[serde(rename = "senderAgencyDetail")]
    sender_agency_detail: Option<SenderAgencyDetail>,
    #[serde(rename = "answerStatusCode")]
    answer_status_code: Option<MessageStatusCode>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct KeyDlgProof {
    #[serde(rename = "agentDID")]
    pub agent_did: String,
    #[serde(rename = "agentDelegatedKey")]
    pub agent_delegated_key: String,
    #[serde(rename = "signature")]
    pub signature: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SenderDetail {
    pub name: Option<String>,
    pub agent_key_dlg_proof: KeyDlgProof,
    #[serde(rename = "DID")]
    pub did: String,
    pub logo_url: Option<String>,
    #[serde(rename = "verKey")]
    pub verkey: String,
    #[serde(rename = "publicDID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_did: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SenderAgencyDetail {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "verKey")]
    pub verkey: String,
    pub endpoint: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InviteDetail {
    status_code: String,
    pub conn_req_id: String,
    pub sender_detail: SenderDetail,
    pub sender_agency_detail: SenderAgencyDetail,
    target_name: String,
    status_msg: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SendInviteMessageDetailsResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "inviteDetail")]
    pub invite_detail: InviteDetail,
    #[serde(rename = "urlToInviteDetail")]
    pub url_to_invite_detail: String,
}

#[derive(Debug)]
pub struct SendInviteBuilder {
    to_did: String,
    to_vk: String,
    payload: SendInvitePayloads,
    agent_did: String,
    agent_vk: String,
    public_did: Option<String>,
}

impl SendInviteMessageDetailsResponse {
    fn from_a2a_message(message: A2AMessage) -> Result<SendInviteMessageDetailsResponse, u32> {
        match message {
            A2AMessage::MessageDetail(MessageDetail::ConnectionRequestResp(msg)) => Ok(msg),
            _ => Err(error::INVALID_HTTP_RESPONSE.code_num)
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ConnectionRequestAnswerResponse {
    #[serde(rename = "@type")]
    pub msg_type: MessageTypes,
    pub uid: String,
    pub sent: bool,
}

impl InviteDetail {
    pub fn new() -> InviteDetail {
        InviteDetail {
            status_code: String::new(),
            conn_req_id: String::new(),
            sender_detail: SenderDetail {
                name: Some(String::new()),
                agent_key_dlg_proof: KeyDlgProof {
                    agent_did: String::new(),
                    agent_delegated_key: String::new(),
                    signature: String::new(),
                },
                did: String::new(),
                logo_url: Some(String::new()),
                verkey: String::new(),
                public_did: None,
            },
            sender_agency_detail: SenderAgencyDetail {
                did: String::new(),
                verkey: String::new(),
                endpoint: String::new(),
            },
            target_name: String::new(),
            status_msg: String::new(),
        }
    }
}

impl SendInviteBuilder {
    pub fn create() -> SendInviteBuilder {
        trace!("SendInvite::create_message >>>");

        let payload =
            match settings::get_protocol_type() {
                settings::ProtocolTypes::V1 => {
                    SendInvitePayloads::SendInvitePayloadV0(
                        SendInvitePayloadV0 {
                            create_payload: CreateMessage {
                                msg_type: MessageTypes::build(A2AMessageKinds::CreateMessage),
                                mtype: CredentialExchangeMessageType::ConnReq,
                                reply_to_msg_id: None,
                                send_msg: true,
                                uid: None,
                            },
                            msg_detail_payload: SendInviteMessageDetails {
                                msg_type: MessageTypes::build(A2AMessageKinds::MessageDetail),
                                key_dlg_proof: KeyDlgProof { agent_did: String::new(), agent_delegated_key: String::new(), signature: String::new() },
                                target_name: None,
                                phone_no: None,
                                include_public_did: false,
                            },
                        }
                    )
                }
                settings::ProtocolTypes::V2 => {
                    SendInvitePayloads::SendInvitePayloadV1(SendInvitePayloadV1 {
                        msg_type: MessageTypes::build(A2AMessageKinds::ConnectionRequest),
                        send_msg: true,
                        uid: None,
                        reply_to_msg_id: None,
                        key_dlg_proof: KeyDlgProof { agent_did: String::new(), agent_delegated_key: String::new(), signature: String::new() },
                        target_name: None,
                        phone_no: None,
                        include_public_did: false,
                    })
                }
            };

        SendInviteBuilder {
            to_did: String::new(),
            to_vk: String::new(),
            payload,
            agent_did: String::new(),
            agent_vk: String::new(),
            public_did: None,
        }
    }

    pub fn key_delegate(&mut self, key: &str) -> Result<&mut Self, u32> {
        validation::validate_key_delegate(key)?;
        match self.payload {
            SendInvitePayloads::SendInvitePayloadV0(ref mut payload) => payload.msg_detail_payload.key_dlg_proof.agent_delegated_key = key.to_string(),
            SendInvitePayloads::SendInvitePayloadV1(ref mut payload) => payload.key_dlg_proof.agent_delegated_key = key.to_string(),
        }
        Ok(self)
    }

    pub fn public_did(&mut self, did: Option<&str>) -> Result<&mut Self, u32> {
        if did.is_some() {
            match self.payload {
                SendInvitePayloads::SendInvitePayloadV0(ref mut payload) => payload.msg_detail_payload.include_public_did = true,
                SendInvitePayloads::SendInvitePayloadV1(ref mut payload) => payload.include_public_did = true,
            }
        }
        self.public_did = did.map(String::from);
        Ok(self)
    }

    pub fn phone_number(&mut self, phone_number: &Option<String>) -> Result<&mut Self, u32> {
        if let &Some(ref p_num) = phone_number {
            validation::validate_phone_number(p_num.as_str())?;
            match self.payload {
                SendInvitePayloads::SendInvitePayloadV0(ref mut payload) => payload.msg_detail_payload.phone_no = Some(p_num.to_string()),
                SendInvitePayloads::SendInvitePayloadV1(ref mut payload) => payload.phone_no = Some(p_num.to_string())
            }
        }
        Ok(self)
    }

    pub fn generate_signature(&mut self) -> Result<(), u32> {
        let prepare_signature = |(key_dlg_proof, to_vk): (&KeyDlgProof, &str)| {
            let signature = format!("{}{}", key_dlg_proof.agent_did, key_dlg_proof.agent_delegated_key);
            let signature = ::utils::libindy::crypto::sign(to_vk, signature.as_bytes()).unwrap(); //TODO: FIXME
            let signature = base64::encode(&signature);
            signature
        };

        match self.payload {
            SendInvitePayloads::SendInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.key_dlg_proof.signature = prepare_signature((&payload.msg_detail_payload.key_dlg_proof, &self.to_vk))
            }
            SendInvitePayloads::SendInvitePayloadV1(ref mut payload) => {
                payload.key_dlg_proof.signature = prepare_signature((&payload.key_dlg_proof, &self.to_vk))
            }
        }
        Ok(())
    }

    pub fn send_secure(&mut self) -> Result<(InviteDetail, String), u32> {
        trace!("SendInvite::send >>>");

        if settings::test_agency_mode_enabled() {
            return SendInviteBuilder::parse_response(SEND_INVITE_RESPONSE.to_vec());
        }

        let data = self.prepare()?;

        let response = httpclient::post_u8(&data).or(Err(error::POST_MSG_FAILURE.code_num))?;

        let (invite, url) = SendInviteBuilder::parse_response(response)?;

        Ok((invite, url))
    }

    fn parse_response(response: Vec<u8>) -> Result<(InviteDetail, String), u32> {
        let mut data = parse_response_from_agency(&response)?;

        match settings::get_protocol_type() {
            settings::ProtocolTypes::V1 => {
                let response: SendInviteMessageDetailsResponse = SendInviteMessageDetailsResponse::from_a2a_message(data.remove(1))?;
                Ok((response.invite_detail, response.url_to_invite_detail))
            }
            settings::ProtocolTypes::V2 => {
                let response: ConnectionRequestResponse = ConnectionRequestResponse::from_a2a_message(data.remove(0))?;
                Ok((response.invite_detail, response.url_to_invite_detail))
            }
        }
    }
}

#[derive(Debug)]
pub struct AcceptInviteBuilder {
    to_did: String,
    to_vk: String,
    payload: AcceptInvitePayloads,
    agent_did: String,
    agent_vk: String
}

impl AcceptInviteBuilder {
    pub fn create() -> AcceptInviteBuilder {
        trace!("AcceptInvite::create_message >>>");

        let payload =
            match settings::get_protocol_type() {
                settings::ProtocolTypes::V1 => {
                    AcceptInvitePayloads::AcceptInvitePayloadV0(
                        AcceptInvitePayloadV0 {
                            create_payload: CreateMessage {
                                msg_type: MessageTypes::build(A2AMessageKinds::CreateMessage),
                                mtype: CredentialExchangeMessageType::ConnReqAnswer,
                                reply_to_msg_id: None,
                                send_msg: true,
                                uid: None,
                            },
                            msg_detail_payload: AcceptInviteMessageDetails {
                                msg_type: MessageTypes::build(A2AMessageKinds::MessageDetail),
                                key_dlg_proof: KeyDlgProof { agent_did: String::new(), agent_delegated_key: String::new(), signature: String::new() },
                                sender_detail: None,
                                sender_agency_detail: None,
                                answer_status_code: None,
                            },
                        }
                    )
                }
                settings::ProtocolTypes::V2 => {
                    AcceptInvitePayloads::AcceptInvitePayloadV1(AcceptInvitePayloadV1 {
                        msg_type: MessageTypes::build(A2AMessageKinds::ConnectionRequestAnswer),
                        send_msg: true,
                        uid: None,
                        reply_to_msg_id: None,
                        key_dlg_proof: KeyDlgProof { agent_did: String::new(), agent_delegated_key: String::new(), signature: String::new() },
                        sender_detail: None,
                        sender_agency_detail: None,
                        answer_status_code: None,
                    })
                }
            };

        AcceptInviteBuilder {
            to_did: String::new(),
            to_vk: String::new(),
            payload,
            agent_did: String::new(),
            agent_vk: String::new(),
        }
    }

    pub fn key_delegate(&mut self, key: &str) -> Result<&mut Self, u32> {
        validation::validate_key_delegate(key)?;
        match self.payload {
            AcceptInvitePayloads::AcceptInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.key_dlg_proof.agent_delegated_key = key.to_string()
            }
            AcceptInvitePayloads::AcceptInvitePayloadV1(ref mut payload) => {
                payload.key_dlg_proof.agent_delegated_key = key.to_string()
            }
        }
        Ok(self)
    }

    pub fn sender_details(&mut self, details: &SenderDetail) -> Result<&mut Self, u32> {
        match self.payload {
            AcceptInvitePayloads::AcceptInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.sender_detail = Some(details.clone());
            }
            AcceptInvitePayloads::AcceptInvitePayloadV1(ref mut payload) => {
                payload.sender_detail = Some(details.clone());
            }
        };
        Ok(self)
    }

    pub fn sender_agency_details(&mut self, details: &SenderAgencyDetail) -> Result<&mut Self, u32> {
        match self.payload {
            AcceptInvitePayloads::AcceptInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.sender_agency_detail = Some(details.clone());
            }
            AcceptInvitePayloads::AcceptInvitePayloadV1(ref mut payload) => {
                payload.sender_agency_detail = Some(details.clone());
            }
        };
        Ok(self)
    }

    pub fn answer_status_code(&mut self, code: &MessageStatusCode) -> Result<&mut Self, u32> {
        match self.payload {
            AcceptInvitePayloads::AcceptInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.answer_status_code = Some(code.clone());
            }
            AcceptInvitePayloads::AcceptInvitePayloadV1(ref mut payload) => {
                payload.answer_status_code = Some(code.clone());
            }
        }
        Ok(self)
    }

    pub fn reply_to(&mut self, id: &str) -> Result<&mut Self, u32> {
        match self.payload {
            AcceptInvitePayloads::AcceptInvitePayloadV0(ref mut payload) => {
                payload.create_payload.reply_to_msg_id = Some(id.to_owned());
            }
            AcceptInvitePayloads::AcceptInvitePayloadV1(ref mut payload) => {
                payload.reply_to_msg_id = Some(id.to_owned());
            }
        }
        Ok(self)
    }

    pub fn generate_signature(&mut self) -> Result<(), u32> {
        let prepare_signature = |key_dlg_proof: &KeyDlgProof, to_vk: &str| {
            let signature = format!("{}{}", key_dlg_proof.agent_did, key_dlg_proof.agent_delegated_key);
            let signature = crypto::sign(to_vk, signature.as_bytes()).unwrap(); // TODO: FIXME
            base64::encode(&signature)
        };

        match self.payload {
            AcceptInvitePayloads::AcceptInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.key_dlg_proof.signature = prepare_signature(&payload.msg_detail_payload.key_dlg_proof, &self.to_vk);
            }
            AcceptInvitePayloads::AcceptInvitePayloadV1(ref mut payload) => {
                payload.key_dlg_proof.signature = prepare_signature(&payload.key_dlg_proof, &self.to_vk);
            }
        }
        Ok(())
    }

    pub fn send_secure(&mut self) -> Result<String, u32> {
        trace!("AcceptInvite::send >>>");

        if settings::test_agency_mode_enabled() {
            return AcceptInviteBuilder::parse_response(ACCEPT_INVITE_RESPONSE.to_vec());
        }

        let data = self.prepare()?;

        let response = httpclient::post_u8(&data).or(Err(error::POST_MSG_FAILURE.code_num))?;

        AcceptInviteBuilder::parse_response(response)
    }

    fn parse_response(response: Vec<u8>) -> Result<String, u32> {
        let mut data = parse_response_from_agency(&response)?;

        let uid =
            match settings::get_protocol_type() {
                settings::ProtocolTypes::V1 => {
                    let response: MessageCreated = MessageCreated::from_a2a_message(data.remove(0))?;
                    response.uid
                }
                settings::ProtocolTypes::V2 => {
                    let response: ConnectionRequestAnswerResponse = ConnectionRequestAnswerResponse::from_a2a_message(data.remove(0))?;
                    response.uid
                }
            };
        Ok(uid)
    }
}


//Todo: Every GeneralMessage extension, duplicates code
impl GeneralMessage for SendInviteBuilder {
    type Msg = SendInviteBuilder;

    fn set_agent_did(&mut self, did: String) {
        self.agent_did = did;
        match self.payload {
            SendInvitePayloads::SendInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.key_dlg_proof.agent_did = self.agent_did.to_string()
            }
            SendInvitePayloads::SendInvitePayloadV1(ref mut payload) => {
                payload.key_dlg_proof.agent_did = self.agent_did.to_string()
            }
        }
    }

    fn set_agent_vk(&mut self, vk: String) {
        self.agent_vk = vk;
        match self.payload {
            SendInvitePayloads::SendInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.key_dlg_proof.agent_delegated_key = self.agent_vk.to_string();
            }
            SendInvitePayloads::SendInvitePayloadV1(ref mut payload) => {
                payload.key_dlg_proof.agent_delegated_key = self.agent_vk.to_string();
            }
        }
    }

    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }

    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }

    fn prepare(&mut self) -> Result<Vec<u8>, u32> {
        self.generate_signature()?;

        let messages =
            match self.payload {
                SendInvitePayloads::SendInvitePayloadV0(ref mut payload) => vec![A2AMessage::CreateMessage(payload.create_payload.clone()),
                                                                                 A2AMessage::MessageDetail(MessageDetail::ConnectionRequest(payload.msg_detail_payload.clone()))],
                SendInvitePayloads::SendInvitePayloadV1(ref mut payload) => vec![A2AMessage::ConnectionRequest(payload.clone())]
            };

        prepare_message_for_agent(messages, &self.to_vk, &self.agent_did, &self.agent_vk)
    }
}

impl GeneralMessage for AcceptInviteBuilder {
    type Msg = AcceptInviteBuilder;

    fn set_agent_did(&mut self, did: String) {
        self.agent_did = did;

        match self.payload {
            AcceptInvitePayloads::AcceptInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.key_dlg_proof.agent_did = self.agent_did.to_string();
            }
            AcceptInvitePayloads::AcceptInvitePayloadV1(ref mut payload) => {
                payload.key_dlg_proof.agent_did = self.agent_did.to_string();
            }
        }
    }

    fn set_agent_vk(&mut self, vk: String) {
        self.agent_vk = vk;

        match self.payload {
            AcceptInvitePayloads::AcceptInvitePayloadV0(ref mut payload) => {
                payload.msg_detail_payload.key_dlg_proof.agent_delegated_key = self.agent_vk.to_string();
            }
            AcceptInvitePayloads::AcceptInvitePayloadV1(ref mut payload) => {
                payload.key_dlg_proof.agent_delegated_key = self.agent_vk.to_string();
            }
        }
    }

    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }
    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }

    fn prepare(&mut self) -> Result<Vec<u8>, u32> {
        self.generate_signature()?;

        let messages =
            match self.payload {
                AcceptInvitePayloads::AcceptInvitePayloadV0(ref mut payload) => vec![A2AMessage::CreateMessage(payload.create_payload.clone()),
                                                                                     A2AMessage::MessageDetail(MessageDetail::ConnectionRequestAnswer(payload.msg_detail_payload.clone()))],
                AcceptInvitePayloads::AcceptInvitePayloadV1(ref mut payload) => vec![A2AMessage::ConnectionRequestAnswer(payload.clone())]
            };

        prepare_message_for_agent(messages, &self.to_vk, &self.agent_did, &self.agent_vk)
    }
}

pub fn parse_invitation_acceptance_details(payload: Vec<u8>) -> Result<SenderDetail, u32> {
    #[serde(rename_all = "camelCase")]
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct Details {
        sender_detail: SenderDetail,
    }

    debug!("parsing invitation acceptance details: {:?}", payload);
    let response: Details = rmp_serde::from_slice(&payload[..]).or(Err(error::INVALID_MSGPACK.code_num))?;
    Ok(response.sender_detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use messages::send_invite;
    use utils::libindy::signus::create_and_store_my_did;

    #[test]
    fn test_send_invite_set_values_and_post() {
        init!("false");
        let (user_did, user_vk) = create_and_store_my_did(None).unwrap();
        let (agent_did, agent_vk) = create_and_store_my_did(Some(MY2_SEED)).unwrap();
        let (my_did, my_vk) = create_and_store_my_did(Some(MY1_SEED)).unwrap();
        let (agency_did, agency_vk) = create_and_store_my_did(Some(MY3_SEED)).unwrap();

        settings::set_config_value(settings::CONFIG_AGENCY_VERKEY, &agency_vk);
        settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agent_vk);
        settings::set_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY, &my_vk);

        let msg = send_invite()
            .to(&user_did).unwrap()
            .to_vk(&user_vk).unwrap()
            .agent_did(&agent_did).unwrap()
            .agent_vk(&agent_vk).unwrap()
            .phone_number(&Some("phone".to_string())).unwrap()
            .key_delegate("key").unwrap()
            .prepare().unwrap();

        assert!(msg.len() > 0);
    }

    #[test]
    fn test_parse_send_invite_response() {
        init!("indy");
        let (result, url) = SendInviteBuilder::parse_response(SEND_INVITE_RESPONSE.to_vec()).unwrap();
        let invite = serde_json::from_str(INVITE_DETAIL_STRING).unwrap();

        assert_eq!(result, invite);
        assert_eq!(url, "http://localhost:9001/agency/invite/WRUzXXuFVTYkT8CjSZpFvT?uid=NjcwOWU");
    }

    #[test]
    fn test_parse_invitation_acceptance_details() {
        let payload = vec![129, 172, 115, 101, 110, 100, 101, 114, 68, 101, 116, 97, 105, 108, 131, 163, 68, 73, 68, 182, 67, 113, 85, 88, 113, 53, 114, 76, 105, 117, 82, 111, 100, 55, 68, 67, 52, 97, 86, 84, 97, 115, 166, 118, 101, 114, 75, 101, 121, 217, 44, 67, 70, 86, 87, 122, 118, 97, 103, 113, 65, 99, 117, 50, 115, 114, 68, 106, 117, 106, 85, 113, 74, 102, 111, 72, 65, 80, 74, 66, 111, 65, 99, 70, 78, 117, 49, 55, 113, 117, 67, 66, 57, 118, 71, 176, 97, 103, 101, 110, 116, 75, 101, 121, 68, 108, 103, 80, 114, 111, 111, 102, 131, 168, 97, 103, 101, 110, 116, 68, 73, 68, 182, 57, 54, 106, 111, 119, 113, 111, 84, 68, 68, 104, 87, 102, 81, 100, 105, 72, 49, 117, 83, 109, 77, 177, 97, 103, 101, 110, 116, 68, 101, 108, 101, 103, 97, 116, 101, 100, 75, 101, 121, 217, 44, 66, 105, 118, 78, 52, 116, 114, 53, 78, 88, 107, 69, 103, 119, 66, 56, 81, 115, 66, 51, 109, 109, 109, 122, 118, 53, 102, 119, 122, 54, 85, 121, 53, 121, 112, 122, 90, 77, 102, 115, 74, 56, 68, 122, 169, 115, 105, 103, 110, 97, 116, 117, 114, 101, 217, 88, 77, 100, 115, 99, 66, 85, 47, 99, 89, 75, 72, 49, 113, 69, 82, 66, 56, 80, 74, 65, 43, 48, 51, 112, 121, 65, 80, 65, 102, 84, 113, 73, 80, 74, 102, 52, 84, 120, 102, 83, 98, 115, 110, 81, 86, 66, 68, 84, 115, 67, 100, 119, 122, 75, 114, 52, 54, 120, 87, 116, 80, 43, 78, 65, 68, 73, 57, 88, 68, 71, 55, 50, 50, 103, 113, 86, 80, 77, 104, 117, 76, 90, 103, 89, 67, 103, 61, 61];
        println!("payload: {:?}", payload);
        let response = parse_invitation_acceptance_details(payload).unwrap();
        println!("response: {:?}", response);
    }
}
