extern crate rust_base58;
extern crate rmp_serde;

use commands::CommandExecutor;
use commands::Command;
use commands::ledger::LedgerCommand;
use errors::common::CommonError;
use errors::pool::PoolError;
use services::pool::events::RequestEvent;
use services::pool::events::PoolEvent;
use services::pool::events::NetworkerEvent;
use services::pool::networker::Networker;
use services::pool::state_proof;
use services::pool::types::HashableValue;
use services::pool::catchup::{CatchupProgress, check_nodes_responses_on_status, check_cons_proofs};

use self::rust_base58::FromBase58;
use serde_json;
use serde_json::Value as SJsonValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::rc::Rc;
use super::indy_crypto::bls::Generator;
use super::indy_crypto::bls::VerKey;
use super::indy_crypto::utils::json::JsonEncodable;
use services::ledger::merkletree::merkletree::MerkleTree;
use services::pool::merkle_tree_factory;
use services::pool::types::CatchupReq;
use services::pool::types::CatchupRep;
use services::pool::types::Message;

trait RequestState {
    fn is_terminal(&self) -> bool {
        false
    }
}

struct StartState<T: Networker> {
    networker: Rc<RefCell<T>>
}

impl<T: Networker> RequestState for StartState<T> {}

struct ConsensusState<T: Networker> {
    nack_cnt: HashSet<String>,
    replies: HashMap<HashableValue, HashSet<String>>,
    timeout_cnt: HashSet<String>,
    networker: Rc<RefCell<T>>,
}

impl<T: Networker> RequestState for ConsensusState<T> {}

struct CatchupConsensusState<T: Networker> {
    replies: HashMap<(String, usize, Option<Vec<String>>), HashSet<String>>,
    networker: Rc<RefCell<T>>,
    merkle_tree: MerkleTree,
}

impl<T: Networker> RequestState for CatchupConsensusState<T> {}

struct CatchupSingleState<T: Networker> {
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
    merkle_tree: MerkleTree,
    networker: Rc<RefCell<T>>,
    req_id: String,
}

impl<T: Networker> RequestState for CatchupSingleState<T> {}

struct SingleState<T: Networker> {
    nack_cnt: HashSet<String>,
    replies: HashMap<HashableValue, HashSet<String>>,
    networker: Rc<RefCell<T>>,
}

impl<T: Networker> RequestState for SingleState<T> {}

struct FinishState {}

impl RequestState for FinishState {
    fn is_terminal(&self) -> bool {
        true
    }
}

struct FullState<T: Networker> {
    accum_reply: Option<HashableValue>,
    networker: Rc<RefCell<T>>,
}

impl<T: Networker> RequestState for FullState<T> {}

struct RequestSM<T: RequestState> {
    f: usize,
    cmd_ids: Vec<i32>,
    nodes: HashMap<String, Option<VerKey>>,
    generator: Generator,
    pool_name: String,
    state: T,
}

impl<T: Networker> RequestSM<StartState<T>> {
    pub fn new(networker: Rc<RefCell<T>>,
               f: usize,
               cmd_ids: &Vec<i32>,
               nodes_: &HashMap<String, Option<VerKey>>,
               generator: Option<Generator>,
               pool_name: &str) -> Self {
        let mut nodes = HashMap::new();
        nodes_.clone().into_iter().for_each(|(key, value)| {
            let value = match value {
                &Some(ref val) => {
                    match VerKey::from_bytes(val.as_bytes()) {
                        Ok(vk) => Some(vk),
                        Err(_) => None
                    }
                }
                &None => None
            };
            nodes.insert(key.clone(), value);
        });
        RequestSM {
            f,
            cmd_ids: cmd_ids.clone(),
            nodes,
            pool_name: pool_name.to_string(),
            generator: generator.unwrap_or(Generator::from_bytes(&"3LHpUjiyFC2q2hD7MnwwNmVXiuaFbQx2XkAFJWzswCjgN1utjsCeLzHsKk1nJvFEaS4fcrUmVAkdhtPCYbrVyATZcmzwJReTcJqwqBCPTmTQ9uWPwz6rEncKb2pYYYFcdHa8N17HzVyTqKfgPi4X9pMetfT3A5xCHq54R2pDNYWVLDX".from_base58().unwrap()).unwrap()),
            state: StartState {
                networker
            },
        }
    }
}

impl<T: Networker> From<RequestSM<StartState<T>>> for RequestSM<SingleState<T>> {
    fn from(sm: RequestSM<StartState<T>>) -> Self {
        RequestSM {
            f: sm.f,
            cmd_ids: sm.cmd_ids,
            nodes: sm.nodes,
            generator: sm.generator,
            pool_name: sm.pool_name,
            state: SingleState {
                nack_cnt: HashSet::new(),
                replies: HashMap::new(),
                networker: sm.state.networker,
            },
        }
    }
}

impl<T: Networker> From<RequestSM<StartState<T>>> for RequestSM<ConsensusState<T>> {
    fn from(val: RequestSM<StartState<T>>) -> Self {
        RequestSM {
            f: val.f,
            cmd_ids: val.cmd_ids,
            nodes: val.nodes,
            generator: val.generator,
            pool_name: val.pool_name,
            state: ConsensusState {
                nack_cnt: HashSet::new(),
                replies: HashMap::new(),
                timeout_cnt: HashSet::new(),
                networker: val.state.networker.clone(),
            },
        }
    }
}

impl<T: Networker> From<(MerkleTree, RequestSM<StartState<T>>)> for RequestSM<CatchupConsensusState<T>> {
    fn from((merkle_tree, val): (MerkleTree, RequestSM<StartState<T>>)) -> Self {
        RequestSM {
            f: val.f,
            cmd_ids: val.cmd_ids,
            nodes: val.nodes,
            generator: val.generator,
            pool_name: val.pool_name,
            state: CatchupConsensusState {
                replies: HashMap::new(),
                networker: val.state.networker.clone(),
                merkle_tree,
            },
        }
    }
}

impl<T: Networker> From<(MerkleTree, RequestSM<StartState<T>>, Vec<u8>, usize, String)> for RequestSM<CatchupSingleState<T>> {
    fn from((merkle_tree, val, target_mt_root, target_mt_size, req_id): (MerkleTree, RequestSM<StartState<T>>, Vec<u8>, usize, String)) -> Self {
        RequestSM {
            f: val.f,
            cmd_ids: val.cmd_ids,
            nodes: val.nodes,
            generator: val.generator,
            pool_name: val.pool_name,
            state: CatchupSingleState {
                target_mt_root,
                target_mt_size,
                networker: val.state.networker.clone(),
                merkle_tree,
                req_id,
            },
        }
    }
}

impl<T: Networker> From<RequestSM<StartState<T>>> for RequestSM<FullState<T>> {
    fn from(val: RequestSM<StartState<T>>) -> Self {
        RequestSM {
            f: val.f,
            cmd_ids: val.cmd_ids,
            nodes: val.nodes,
            generator: val.generator,
            pool_name: val.pool_name,
            state: FullState {
                accum_reply: None,
                networker: val.state.networker.clone(),
            },
        }
    }
}

impl<T: Networker> From<RequestSM<SingleState<T>>> for RequestSM<FinishState> {
    fn from(val: RequestSM<SingleState<T>>) -> Self {
        //TODO: close connections in networker
        RequestSM {
            f: val.f,
            cmd_ids: val.cmd_ids,
            nodes: val.nodes,
            generator: val.generator,
            pool_name: val.pool_name,
            state: FinishState {},
        }
    }
}

impl<T: Networker> From<RequestSM<ConsensusState<T>>> for RequestSM<FinishState> {
    fn from(val: RequestSM<ConsensusState<T>>) -> Self {
        //TODO: close connections in networker
        RequestSM {
            f: val.f,
            cmd_ids: val.cmd_ids,
            nodes: val.nodes,
            generator: val.generator,
            pool_name: val.pool_name,
            state: FinishState {},
        }
    }
}

impl<T: Networker> From<RequestSM<CatchupConsensusState<T>>> for RequestSM<FinishState> {
    fn from(val: RequestSM<CatchupConsensusState<T>>) -> Self {
        //TODO: close connections in networker
        RequestSM {
            f: val.f,
            cmd_ids: val.cmd_ids,
            nodes: val.nodes,
            generator: val.generator,
            pool_name: val.pool_name,
            state: FinishState {},
        }
    }
}

impl<T: Networker> From<RequestSM<CatchupSingleState<T>>> for RequestSM<FinishState> {
    fn from(val: RequestSM<CatchupSingleState<T>>) -> Self {
        //TODO: close connections in networker
        RequestSM {
            f: val.f,
            cmd_ids: val.cmd_ids,
            nodes: val.nodes,
            generator: val.generator,
            pool_name: val.pool_name,
            state: FinishState {},
        }
    }
}

impl<T: Networker> From<RequestSM<FullState<T>>> for RequestSM<FinishState> {
    fn from(sm: RequestSM<FullState<T>>) -> Self {
        //TODO: close connections in networker
        RequestSM {
            f: sm.f,
            cmd_ids: sm.cmd_ids,
            nodes: sm.nodes,
            generator: sm.generator,
            pool_name: sm.pool_name,
            state: FinishState {},
        }
    }
}

impl<T: Networker> From<RequestSM<StartState<T>>> for RequestSM<FinishState> {
    fn from(sm: RequestSM<StartState<T>>) -> Self {
        RequestSM {
            f: sm.f,
            cmd_ids: sm.cmd_ids,
            nodes: sm.nodes,
            generator: sm.generator,
            pool_name: sm.pool_name,
            state: FinishState {},
        }
    }
}

enum RequestSMWrapper<T: Networker> {
    Start(RequestSM<StartState<T>>),
    Single(RequestSM<SingleState<T>>),
    Consensus(RequestSM<ConsensusState<T>>),
    CatchupSingle(RequestSM<CatchupSingleState<T>>),
    CatchupConsensus(RequestSM<CatchupConsensusState<T>>),
    Full(RequestSM<FullState<T>>),
    Finish(RequestSM<FinishState>),
}

impl<T: Networker> RequestSMWrapper<T> {
    fn handle_event(self, re: RequestEvent) -> (Self, Option<PoolEvent>) {
        match self {
            RequestSMWrapper::Start(request) => {
                let ne: Option<NetworkerEvent> = re.clone().into();
                match re {
                    RequestEvent::LedgerStatus(_, _, Some(merkle)) => {
                        trace!("start catchup, ne: {:?}", ne);
                        request.state.networker.borrow_mut().process_event(ne);
                        (RequestSMWrapper::CatchupConsensus((merkle, request).into()), None)
                    }
                    RequestEvent::CatchupReq(merkle, target_mt_size, target_mt_root) => {
                        let txns_cnt = target_mt_size - merkle.count();

                        if txns_cnt <= 0 {
                            warn!("No transactions to catch up!");
                            return (RequestSMWrapper::Finish(request.into()), Some(PoolEvent::Synced(merkle)));
                        }
                        let seq_no_start = merkle.count() + 1;
                        let seq_no_end = target_mt_size;

                        let cr = CatchupReq {
                            ledgerId: 0,
                            seqNoStart: seq_no_start.clone(),
                            seqNoEnd: seq_no_end.clone(),
                            catchupTill: target_mt_size,
                        };
                        let req_id = format!("{}{}", seq_no_start, seq_no_end);
                        let str = Message::CatchupReq(cr).to_json().expect("FIXME");
                        trace!("catchup_req msg: {:?}", str);
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::SendOneRequest(str, req_id.clone())));
                        (RequestSMWrapper::CatchupSingle((merkle, request, target_mt_root, target_mt_size, req_id).into()), None)
                    }
                    RequestEvent::CustomSingleRequest(msg, req_id) => {
                        match req_id {
                            Ok(req_id) => {
                                request.state.networker.borrow_mut()
                                    .process_event(Some(NetworkerEvent::SendOneRequest(msg, req_id)));
                                (RequestSMWrapper::Single(request.into()), None)
                            }
                            Err(e) => {
                                _send_replies(&request.cmd_ids, Err(PoolError::CommonError(e)));
                                (RequestSMWrapper::Finish(request.into()), None)
                            }
                        }
                    }
                    RequestEvent::CustomFullRequest(msg, req_id) => {
                        match req_id {
                            Ok(req_id) => {
                                request.state.networker.borrow_mut()
                                    .process_event(Some(NetworkerEvent::SendAllRequest(msg, req_id)));
                                (RequestSMWrapper::Full(request.into()), None)
                            }
                            Err(e) => {
                                _send_replies(&request.cmd_ids, Err(PoolError::CommonError(e)));
                                (RequestSMWrapper::Finish(request.into()), None)
                            }
                        }
                    }
                    RequestEvent::CustomConsensusRequest(msg, req_id) => {
                        match req_id {
                            Ok(req_id) => {
                                request.state.networker.borrow_mut()
                                    .process_event(Some(NetworkerEvent::SendAllRequest(msg, req_id)));
                                (RequestSMWrapper::Consensus(request.into()), None)
                            }
                            Err(e) => {
                                _send_replies(&request.cmd_ids, Err(PoolError::CommonError(e)));
                                (RequestSMWrapper::Finish(request.into()), None)
                            }
                        }
                    }
                    _ => (RequestSMWrapper::Start(request), None)
                }
            }
            RequestSMWrapper::Consensus(mut request) => {
                match re {
                    RequestEvent::Reply(_, raw_msg, node_alias, req_id) => {
                        if let Ok((_, result_without_proof)) = _get_msg_result_without_state_proof(&raw_msg) {
                            let hashable = HashableValue { inner: result_without_proof };

                            let cnt = {
                                let set = request.state.replies.entry(hashable).or_insert(HashSet::new());
                                set.insert(node_alias.clone());
                                set.len()
                            };

                            if cnt > request.f {
                                _send_ok_replies(&request.cmd_ids, &raw_msg);
                                request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                                (RequestSMWrapper::Finish(request.into()), None)
                            } else if _is_consensus_reachable(&request.state.replies, request.f, request.nodes.len(), request.state.timeout_cnt.len(), request.state.nack_cnt.len()) {
                                request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                                (RequestSMWrapper::Consensus(request), None)
                            } else {
                                //TODO: maybe we should change the error, but it was made to escape changing of ErrorCode returned to client
                                _send_replies(&request.cmd_ids, Err(PoolError::Timeout));
                                request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                                (RequestSMWrapper::Finish(request.into()), None)
                            }
                        } else {
                            (RequestSMWrapper::Consensus(request), None)
                        }
                    }
                    RequestEvent::ReqACK(_, _, node_alias, req_id) => {
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::ExtendTimeout(req_id, node_alias)));
                        (RequestSMWrapper::Consensus(request), None)
                    }
                    RequestEvent::ReqNACK(_, raw_msg, node_alias, req_id) | RequestEvent::Reject(_, raw_msg, node_alias, req_id) => {
                        if _parse_nack(&mut request.state.nack_cnt, request.f, &raw_msg, &request.cmd_ids, &node_alias) {
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                            (RequestSMWrapper::Finish(request.into()), None)
                        } else if _is_consensus_reachable(&request.state.replies, request.f, request.nodes.len(), request.state.timeout_cnt.len(), request.state.nack_cnt.len()) {
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                            (RequestSMWrapper::Consensus(request), None)
                        } else {
                            _send_replies(&request.cmd_ids, Err(PoolError::Timeout));
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                            (RequestSMWrapper::Finish(request.into()), None)
                        }
                    }
                    RequestEvent::Timeout(req_id, node_alias) => {
                        request.state.timeout_cnt.insert(node_alias.clone());
                        if _is_consensus_reachable(&request.state.replies, request.f, request.nodes.len(), request.state.timeout_cnt.len(), request.state.nack_cnt.len()) {
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                            (RequestSMWrapper::Consensus(request), None)
                        } else {
                            //TODO: maybe we should change the error, but it was made to escape changing of ErrorCode returned to client
                            _send_replies(&request.cmd_ids, Err(PoolError::Timeout));
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                            (RequestSMWrapper::Finish(request.into()), None)
                        }
                    }
                    RequestEvent::Terminate => {
                        _finish_request(&request.cmd_ids);
                        (RequestSMWrapper::Finish(request.into()), None)
                    }
                    _ => (RequestSMWrapper::Consensus(request), None)
                }
            }
            RequestSMWrapper::Single(mut request) => {
                match re {
                    RequestEvent::Reply(_, raw_msg, node_alias, req_id) => {
                        trace!("reply on single request");
                        if let Ok((result, result_without_proof)) = _get_msg_result_without_state_proof(&raw_msg) {
                            let hashable = HashableValue { inner: result_without_proof };

                            let cnt = {
                                let set = request.state.replies.entry(hashable).or_insert(HashSet::new());
                                set.insert(node_alias.clone());
                                set.len()
                            };

                            if cnt > request.f || _check_state_proof(&result, request.f, &request.generator, &request.nodes, &raw_msg) {
                                request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                                _send_ok_replies(&request.cmd_ids, &raw_msg);
                                (RequestSMWrapper::Finish(request.into()), None)
                            } else {
                                request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(req_id.clone())));
                                request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                                (RequestSMWrapper::Single(request), None)
                            }
                        } else {
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(req_id.clone())));
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                            (RequestSMWrapper::Single(request), None)
                        }
                    }
                    RequestEvent::ReqACK(_, _, node_alias, req_id) => {
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::ExtendTimeout(req_id, node_alias)));
                        (RequestSMWrapper::Single(request), None)
                    }
                    RequestEvent::ReqNACK(_, raw_msg, node_alias, req_id) | RequestEvent::Reject(_, raw_msg, node_alias, req_id) => {
                        if _parse_nack(&mut request.state.nack_cnt, request.f, &raw_msg, &request.cmd_ids, &node_alias) {
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                            (RequestSMWrapper::Finish(request.into()), None)
                        } else {
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(req_id.clone())));
                            request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                            (RequestSMWrapper::Single(request), None)
                        }
                    }
                    RequestEvent::Timeout(req_id, node_alias) => {
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(req_id.clone())));
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                        (RequestSMWrapper::Single(request), None)
                    }
                    RequestEvent::Terminate => {
                        _finish_request(&request.cmd_ids);
                        (RequestSMWrapper::Finish(request.into()), None)
                    }
                    _ => (RequestSMWrapper::Single(request), None)
                }
            }
            RequestSMWrapper::CatchupConsensus(mut request) => {
                let node_result = match re {
                    RequestEvent::LedgerStatus(ls, Some(node_alias), _) => {
                        Some((ls.merkleRoot.clone(), ls.txnSeqNo, None, node_alias, ls.merkleRoot))
                    }
                    RequestEvent::ConsistencyProof(cp, node_alias) => {
                        Some((cp.newMerkleRoot, cp.seqNoEnd, Some(cp.hashes), node_alias, cp.oldMerkleRoot))
                    }
                    RequestEvent::Timeout(req_id, node_alias) => {
                        Some(("timeout".to_string(), 0, None, node_alias, req_id))
                    }

                    RequestEvent::Terminate => {
                        _finish_request(&request.cmd_ids);
                        return (RequestSMWrapper::Finish(request.into()), None)
                    }
                    _ => None
                };

                if let Some((mt_root, sz, cons_proof, node_alias, req_id)) = node_result {
                    let (finished, result) = _process_catchup_target(mt_root, sz, cons_proof, &node_alias, &mut request);
                    if finished {
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                        (RequestSMWrapper::Finish(request.into()), result)
                    } else {
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                        (RequestSMWrapper::CatchupConsensus(request), result)
                    }
                } else {
                    (RequestSMWrapper::CatchupConsensus(request), None)
                }
            }
            RequestSMWrapper::CatchupSingle(mut request) => {
                match re {
                    RequestEvent::CatchupRep(mut cr, node_alias) => {
                        match _process_catchup_reply(&mut cr, &mut request.state.merkle_tree, &request.state.target_mt_root, request.state.target_mt_size, &request.pool_name) {
                            Ok(merkle) => {
                                request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(request.state.req_id.clone(), None)));
                                (RequestSMWrapper::Finish(request.into()), Some(PoolEvent::Synced(merkle)))
                            },
                            Err(_) => {
                                request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(request.state.req_id.clone())));
                                request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(request.state.req_id.clone(), Some(node_alias))));
                                (RequestSMWrapper::CatchupSingle(request), None)
                            }
                        }
                    }
                    RequestEvent::Timeout(req_id, node_alias) => {
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(request.state.req_id.clone())));
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                        (RequestSMWrapper::CatchupSingle(request), None)
                    }
                    RequestEvent::Terminate => {
                        _finish_request(&request.cmd_ids);
                        (RequestSMWrapper::Finish(request.into()), None)
                    }
                    _ => (RequestSMWrapper::CatchupSingle(request), None)
                }
            }
            RequestSMWrapper::Full(mut request) => {
                let single_node_result = match re {
                    RequestEvent::Reply(_, raw_msg, node_alias, req_id) |
                    RequestEvent::ReqNACK(_, raw_msg, node_alias, req_id) |
                    RequestEvent::Reject(_, raw_msg, node_alias, req_id) => Some((req_id, node_alias, raw_msg)),
                    RequestEvent::Timeout(req_id, node_alias) => Some((req_id, node_alias, "timeout".to_string())),

                    RequestEvent::ReqACK(_, _, node_alias, req_id) => {
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::ExtendTimeout(req_id, node_alias)));
                        None
                    }
                    RequestEvent::Terminate => {
                        _finish_request(&request.cmd_ids);
                        return (RequestSMWrapper::Finish(request.into()), None)
                    }
                    _ => None,
                };

                if let Some((req_id, node_alias, node_result)) = single_node_result {
                    let first_resp = request.state.accum_reply.is_none();
                    if first_resp {
                        request.state.accum_reply = Some(HashableValue {
                            inner: json!({node_alias.clone(): node_result})
                        })
                    } else {
                        request.state.accum_reply.as_mut().unwrap()
                            .inner.as_object_mut().unwrap()
                            .insert(node_alias.clone(), SJsonValue::from(node_result));
                    }

                    let reply_cnt = request.state.accum_reply.as_ref().unwrap()
                        .inner.as_object().unwrap().len();

                    if reply_cnt == request.nodes.len() {
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                        let reply = request.state.accum_reply.as_ref().unwrap().inner.to_string();
                        _send_ok_replies(&request.cmd_ids, &reply);
                        (RequestSMWrapper::Finish(request.into()), None)
                    } else {
                        request.state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                        (RequestSMWrapper::Full(request), None)
                    }
                } else {
                    (RequestSMWrapper::Full(request), None)
                }
            }
            RequestSMWrapper::Finish(request) => (RequestSMWrapper::Finish(request), None)
        }
    }

    fn is_terminal(&self) -> bool {
        match self {
            &RequestSMWrapper::Start(ref request) => request.state.is_terminal(),
            &RequestSMWrapper::Consensus(ref request) => request.state.is_terminal(),
            &RequestSMWrapper::Single(ref request) => request.state.is_terminal(),
            &RequestSMWrapper::Finish(ref request) => request.state.is_terminal(),
            &RequestSMWrapper::CatchupSingle(ref request) => request.state.is_terminal(),
            &RequestSMWrapper::CatchupConsensus(ref request) => request.state.is_terminal(),
            &RequestSMWrapper::Full(ref request) => request.state.is_terminal(),
        }
    }
}

pub trait RequestHandler<T: Networker> {
    fn new(networker: Rc<RefCell<T>>, f: usize, cmd_ids: &Vec<i32>, nodes: &HashMap<String, Option<VerKey>>, generator: Option<Generator>, pool_name: &str) -> Self;
    fn process_event(&mut self, ore: Option<RequestEvent>) -> Option<PoolEvent>;
    fn is_terminal(&self) -> bool;
}

pub struct RequestHandlerImpl<T: Networker> {
    request_wrapper: Option<RequestSMWrapper<T>>
}

impl<T: Networker> RequestHandler<T> for RequestHandlerImpl<T> {
    fn new(networker: Rc<RefCell<T>>, f: usize, cmd_ids: &Vec<i32>, nodes: &HashMap<String, Option<VerKey>>, generator: Option<Generator>, pool_name: &str) -> Self {
        RequestHandlerImpl {
            request_wrapper: Some(RequestSMWrapper::Start(RequestSM::new(networker, f, cmd_ids, nodes, generator, pool_name))),
        }
    }

    fn process_event(&mut self, ore: Option<RequestEvent>) -> Option<PoolEvent> {
        match ore {
            Some(re) => {
                if let Some((rw, res)) = self.request_wrapper.take().map(|w| w.handle_event(re)) {
                    self.request_wrapper = Some(rw);
                    res
                } else {
                    self.request_wrapper = None;
                    None
                }
            }
            None => None
        }
    }

    fn is_terminal(&self) -> bool {
        self.request_wrapper.as_ref().map(|w| w.is_terminal()).unwrap_or(true)
    }
}

#[cfg(test)]
#[derive(Debug)]
pub struct MockRequestHandler {}

#[cfg(test)]
impl<T: Networker> RequestHandler<T> for MockRequestHandler {
    fn new(_networker: Rc<RefCell<T>>, _f: usize, _cmd_ids: &Vec<i32>, _nodes: &HashMap<String, Option<VerKey>>, _generator: Option<Generator>, _pool_name: &str) -> Self {
        MockRequestHandler {}
    }

    fn process_event(&mut self, _ore: Option<RequestEvent>) -> Option<PoolEvent> {
        None
    }

    fn is_terminal(&self) -> bool {
        true
    }
}

fn _is_consensus_reachable(replies: &HashMap<HashableValue, HashSet<String>>, f: usize, node_cnt: usize, timeout_cnt: usize, nack_cnt: usize) -> bool {
    let rep_no: usize = replies.values().map(|set| set.len()).sum();
    let max_no = replies.values().map(|set| set.len()).max().unwrap_or(0);
    max_no + node_cnt - rep_no - timeout_cnt - nack_cnt > f
}

fn _parse_nack(cnt: &mut HashSet<String>, f: usize, raw_msg: &str, cmd_ids: &Vec<i32>, node_alias: &str) -> bool {
    if cnt.len() == f {
        _send_ok_replies(cmd_ids, raw_msg);
        true
    } else {
        cnt.insert(node_alias.to_string());
        false
    }
}

fn _process_catchup_target<T: Networker>(merkle_root: String,
                                         txn_seq_no: usize,
                                         hashes: Option<Vec<String>>,
                                         node_alias: &str,
                                         request: &mut RequestSM<CatchupConsensusState<T>>) -> (bool, Option<PoolEvent>) {
    let key = (merkle_root, txn_seq_no, hashes);
    let contains = request.state.replies.get_mut(&key)
        .map(|set| { set.insert(node_alias.to_string()); })
        .is_some();

    if !contains {
        request.state.replies.insert(key, HashSet::from_iter(vec![node_alias.to_string()]));
    }

    match check_nodes_responses_on_status(&request.state.replies,
                                          &request.state.merkle_tree,
                                          request.nodes.len(),
                                          request.f,
                                          &request.pool_name) {
        Ok(CatchupProgress::InProgress) => (false, None),
        Ok(CatchupProgress::NotNeeded(merkle_tree)) => (true, Some(PoolEvent::Synced(merkle_tree))),
        Ok(CatchupProgress::ShouldBeStarted(target_mt_root, target_mt_size, merkle_tree)) =>
            (true, Some(PoolEvent::CatchupTargetFound(target_mt_root, target_mt_size, merkle_tree))),
        Err(err) => (true, Some(PoolEvent::CatchupTargetNotFound(err))),
    }
}

fn _process_catchup_reply(rep: &mut CatchupRep, merkle: &MerkleTree, target_mt_root: &Vec<u8>, target_mt_size: usize, pool_name: &str) -> Result<MerkleTree, PoolError> {
    let mut txns_to_drop = vec![];
    let mut merkle = merkle.clone();
    while !rep.txns.is_empty() {
        let key = rep.min_tx()?;
        let txn = rep.txns.remove(&key.to_string()).unwrap();
        if let Ok(txn_bytes) = rmp_serde::to_vec_named(&txn) {
            merkle.append(txn_bytes.clone())?;
            txns_to_drop.push(txn_bytes);
        } else {
            return Err(PoolError::CommonError(CommonError::InvalidStructure("Invalid transaction -- can not transform to bytes".to_string()))).map_err(map_err_trace!());
        }
    }

    if let Err(err) = check_cons_proofs(&merkle, &rep.consProof, target_mt_root, target_mt_size).map_err(map_err_trace!()) {
        return Err(PoolError::CommonError(err));
    }

    merkle_tree_factory::dump_new_txns(pool_name, &txns_to_drop)?;
    Ok(merkle)
}

fn _send_ok_replies(cmd_ids: &Vec<i32>, msg: &str) {
    _send_replies(cmd_ids, Ok(msg.to_string()))
}

fn _finish_request(cmd_ids: &Vec<i32>) {
    _send_replies(cmd_ids, Err(PoolError::Terminate))
}

fn _send_replies(cmd_ids: &Vec<i32>, msg: Result<String, PoolError>) {
    cmd_ids.into_iter().for_each(|id| {
        CommandExecutor::instance().send(
            Command::Ledger(
                LedgerCommand::SubmitAck(id.clone(), msg.clone()))
        ).unwrap();
    });
}

fn _get_msg_result_without_state_proof(msg: &str) -> Result<(SJsonValue, SJsonValue), CommonError> {
    let msg_result: SJsonValue = match serde_json::from_str::<SJsonValue>(msg) {
        Ok(raw_msg) => raw_msg["result"].clone(),
        Err(err) => return Err(CommonError::InvalidStructure(format!("Invalid response structure: {:?}", err))).map_err(map_err_err!())
    };

    let mut msg_result_without_proof: SJsonValue = msg_result.clone();
    msg_result_without_proof.as_object_mut().map(|obj| obj.remove("state_proof"));
    if msg_result_without_proof["data"].is_object() {
        msg_result_without_proof["data"].as_object_mut().map(|obj| obj.remove("stateProofFrom"));
    }
    Ok((msg_result, msg_result_without_proof))
}

fn _check_state_proof(msg_result: &SJsonValue, f: usize, gen: &Generator, bls_keys: &HashMap<String, Option<VerKey>>, raw_msg: &str) -> bool {
    debug!("TransactionHandler::process_reply: Try to verify proof and signature");

    match state_proof::parse_generic_reply_for_proof_checking(&msg_result, raw_msg) {
        Some(parsed_sps) => {
            debug!("TransactionHandler::process_reply: Proof and signature are present");
            state_proof::verify_parsed_sp(parsed_sps, bls_keys, f, gen)
        }
        None => false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use services::pool::networker::MockNetworker;
    use services::pool::types::{LedgerStatus, Reply, ReplyV1, ReplyResultV1, ReplyTxnV1, ResponseMetadata, Response, ResponseV1, ConsistencyProof};
    use services::ledger::merkletree::tree::Tree;
    use utils::test::TestUtils;

    const MESSAGE: &'static str = "message";
    const REQ_ID: &'static str = "1";
    const NODE: &'static str = "n1";
    const NODE_2: &'static str = "n2";
    const SIMPLE_REPLY: &'static str = r#"{"result":{}}"#;
    const POOL: &'static str = "pool1";

    impl Default for LedgerStatus {
        fn default() -> Self {
            LedgerStatus {
                txnSeqNo: 0,
                merkleRoot: String::new(),
                ledgerId: 0,
                ppSeqNo: None,
                viewNo: None,
                protocolVersion: None,
            }
        }
    }

    impl Default for MerkleTree {
        fn default() -> Self {
            MerkleTree {
                root: Tree::Empty { hash: Vec::new() },
                height: 0,
                count: 0,
                nodes_count: 0,
            }
        }
    }

    impl Default for Reply {
        fn default() -> Self {
            Reply::ReplyV1(ReplyV1 { result: ReplyResultV1 { txn: ReplyTxnV1 { metadata: ResponseMetadata { req_id: 1 } } } })
        }
    }

    impl Default for Response {
        fn default() -> Self {
            Response::ResponseV1(ResponseV1 { metadata: ResponseMetadata { req_id: 1 } })
        }
    }

    impl Default for ConsistencyProof {
        fn default() -> Self {
            ConsistencyProof {
                seqNoEnd: 0,
                seqNoStart: 0,
                ledgerId: 0,
                hashes: Vec::new(),
                oldMerkleRoot: String::new(),
                newMerkleRoot: String::new(),
            }
        }
    }

    impl Default for CatchupRep {
        fn default() -> Self {
            CatchupRep {
                ledgerId: 0,
                consProof: Vec::new(),
                txns: HashMap::new(),
            }
        }
    }

    fn _request_handler(f: usize, nodes_cnt: usize) -> RequestHandlerImpl<MockNetworker> {
        let networker = Rc::new(RefCell::new(MockNetworker::new()));

        let mut default_nodes: HashMap<String, Option<VerKey>> = HashMap::new();
        default_nodes.insert(NODE.to_string(), None);

        let node_names = vec![NODE, NODE_2, "n3"];
        let mut nodes: HashMap<String, Option<VerKey>> = HashMap::new();

        for i in 0..nodes_cnt{
            nodes.insert(node_names[i].to_string(), None);
        }

        RequestHandlerImpl::new(networker,
                                f,
                                &vec![],
                                &nodes,
                                None,
                                POOL)
    }

    // required because of dumping txns to cache
    fn _create_pool(content: Option<String>) {
        use utils::environment::EnvironmentUtils;
        use std::fs;
        use std::fs::File;
        use std::io::Write;

        let mut path = EnvironmentUtils::pool_path(POOL);

        path.push(POOL);
        path.set_extension("txn");

        fs::create_dir_all(path.parent().unwrap()).unwrap();

        let mut file = File::create(path).unwrap();
        file.write_all(content.unwrap_or("{}".to_string()).as_bytes()).unwrap();
    }

    #[test]
    fn request_handler_new_works() {
        let request_handler = _request_handler(0, 1);
        assert_match!(RequestSMWrapper::Start(_), request_handler.request_wrapper.unwrap());
    }

    #[test]
    fn request_handler_process_event_works() {
        let mut request_handler = _request_handler(0, 1);
        request_handler.process_event(None);
    }

    mod start {
        use super::*;

        #[test]
        fn request_handler_process_ledger_status_event_from_start_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            assert_match!(RequestSMWrapper::CatchupConsensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_catchup_req_event_from_start_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            assert_match!(RequestSMWrapper::CatchupSingle(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_catchup_req_event_from_start_works_for_no_transactions_to_catchup() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 0, vec![])));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_custom_single_req_event_from_start_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            assert_match!(RequestSMWrapper::Single(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_custom_single_req_event_from_start_works_for_error() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Err(CommonError::InvalidStructure(String::new())))));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_consensus_full_req_event_from_start_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            assert_match!(RequestSMWrapper::Full(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_consensus_full_req_event_from_start_works_for_error() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(MESSAGE.to_string(), Err(CommonError::InvalidStructure(String::new())))));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_custom_consensus_req_event_from_start_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            assert_match!(RequestSMWrapper::Consensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_custom_consensus_req_event_from_start_works_for_error() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Err(CommonError::InvalidStructure(String::new())))));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_other_event_from_start_works_for_error() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestSMWrapper::Start(_), request_handler.request_wrapper.unwrap());
        }

    }

    mod consensus_state{
        use super::*;

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_consensus_reached() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), SIMPLE_REPLY.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_consensus_reachable() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), SIMPLE_REPLY.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Consensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":{}}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{}"#.to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_invalid_message() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Consensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqack_event_from_consensus_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Consensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqnack_event_from_consensus_state_works_for_consensus_reached() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqnack_event_from_consensus_state_works_for_consensus_reachable() {
            let mut request_handler = _request_handler(1, 3);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Consensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqnack_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), r#"{"result":{}}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reject_event_from_consensus_state_works_for_consensus_reached() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reject_event_from_consensus_state_works_for_consensus_reachable() {
            let mut request_handler = _request_handler(1, 3);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Consensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reject_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), r#"{"result":{}}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_timeout_event_from_consensus_state_works_for_consensus_reachable() {
            let mut request_handler = _request_handler(1, 3);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Timeout(NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Consensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_timeout_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Timeout(NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_terminate_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_other_event_from_consensus_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Ping));
            assert_match!(RequestSMWrapper::Consensus(_), request_handler.request_wrapper.unwrap());
        }
    }

    mod single {
        use super::*;

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_consensus_reached() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "{}".to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_state_proof() {
            // Register custom state proof parser
            {
                use services::pool::{PoolService, REGISTERED_SP_PARSERS};
                use api::ErrorCode;
                use std::os::raw::c_char;
                use std::ffi::CString;

                REGISTERED_SP_PARSERS.lock().unwrap().clear();

                extern fn test_sp(_reply_from_node: *const c_char, parsed_sp: *mut *const c_char) -> ErrorCode {
                    let sp: CString = CString::new("[]").unwrap();
                    unsafe { *parsed_sp = sp.into_raw(); }
                    ErrorCode::Success
                }
                extern fn test_free(_data: *const c_char) -> ErrorCode {
                    ErrorCode::Success
                }
                PoolService::register_sp_parser("test", test_sp, test_free).unwrap();
            }

            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result": {"type":"test"}}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_not_completed() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Single(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_invalid_message() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Single(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqack_event_from_single_state_works() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Single(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqnack_event_from_single_state_works_for_completed() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqnack_event_from_single_state_works_for_not_completed() {
            let mut request_handler = _request_handler(1, 3);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Single(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reject_event_from_single_state_works_for_completed() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reject_event_from_single_state_works_for_not_completed() {
            let mut request_handler = _request_handler(1, 3);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Single(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_timeout_event_from_single_state_works() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestSMWrapper::Single(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_terminate_event_from_single_state_works() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_other_event_from_single_state_works() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Pong));
            assert_match!(RequestSMWrapper::Single(_), request_handler.request_wrapper.unwrap());
        }
    }

    mod catchup_consensus {
        use super::*;

        #[test]
        fn request_handler_process_ledger_status_event_from_catchup_consensus_state_works_for_catchup_completed() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_ledger_status_event_from_catchup_consensus_state_works_for_catchup_not_completed() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            assert_match!(RequestSMWrapper::CatchupConsensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_consistency_proof_event_from_catchup_consensus_state_works_for_catchup_completed() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::ConsistencyProof(ConsistencyProof::default(), NODE.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_consistency_proof_event_from_catchup_consensus_state_works_for_catchup_not_completed() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::ConsistencyProof(ConsistencyProof::default(), NODE.to_string())));
            assert_match!(RequestSMWrapper::CatchupConsensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_timeout_event_from_catchup_consensus_state_works() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestSMWrapper::CatchupConsensus(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_timeout_event_from_catchup_consensus_state_works_for_all_timeouts() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_terminate_event_from_catchup_consensus_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_other_event_from_catchup_consensus_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::Pong));
            assert_match!(RequestSMWrapper::CatchupConsensus(_), request_handler.request_wrapper.unwrap());
        }
    }

    mod catchup_single {
        use super::*;

        #[test]
        fn request_handler_process_catchup_reply_event_from_catchup_single_state_works() {
            TestUtils::cleanup_indy_home();
            _create_pool(None);

            let mut request_handler = _request_handler(0, 1);

            let mt = MerkleTree {
                root: Tree::Leaf {
                    hash: vec![144, 26, 156, 60, 166, 79, 255, 53, 172, 15, 42, 186, 99, 222, 43, 53, 230, 243, 151, 105, 0, 233, 90, 151, 103, 149, 22, 172, 76, 124, 247, 62],
                    value: vec![132, 172, 114, 101, 113, 83, 105, 103, 110, 97, 116, 117, 114, 101, 128, 163, 116, 120, 110, 131, 164, 100, 97, 116, 97, 130, 164, 100, 97, 116, 97, 135, 165, 97, 108, 105, 97, 115, 165, 78, 111, 100, 101, 49, 166, 98, 108, 115, 107, 101, 121, 217, 175, 52, 78, 56, 97, 85, 78, 72, 83, 103, 106, 81, 86, 103, 107, 112, 109, 56, 110, 104, 78, 69, 102, 68, 102, 54, 116, 120, 72, 122, 110, 111, 89, 82, 69, 103, 57, 107, 105, 114, 109, 74, 114, 107, 105, 118, 103, 76, 52, 111, 83, 69, 105, 109, 70, 70, 54, 110, 115, 81, 54, 77, 52, 49, 81, 118, 104, 77, 50, 90, 51, 51, 110, 118, 101, 115, 53, 118, 102, 83, 110, 57, 110, 49, 85, 119, 78, 70, 74, 66, 89, 116, 87, 86, 110, 72, 89, 77, 65, 84, 110, 55, 54, 118, 76, 117, 76, 51, 122, 85, 56, 56, 75, 121, 101, 65, 89, 99, 72, 102, 115, 105, 104, 51, 72, 101, 54, 85, 72, 99, 88, 68, 120, 99, 97, 101, 99, 72, 86, 122, 54, 106, 104, 67, 89, 122, 49, 80, 50, 85, 90, 110, 50, 98, 68, 86, 114, 117, 76, 53, 119, 88, 112, 101, 104, 103, 66, 102, 66, 97, 76, 75, 109, 51, 66, 97, 169, 99, 108, 105, 101, 110, 116, 95, 105, 112, 168, 49, 48, 46, 48, 46, 48, 46, 50, 171, 99, 108, 105, 101, 110, 116, 95, 112, 111, 114, 116, 205, 37, 230, 167, 110, 111, 100, 101, 95, 105, 112, 168, 49, 48, 46, 48, 46, 48, 46, 50, 169, 110, 111, 100, 101, 95, 112, 111, 114, 116, 205, 37, 229, 168, 115, 101, 114, 118, 105, 99, 101, 115, 145, 169, 86, 65, 76, 73, 68, 65, 84, 79, 82, 164, 100, 101, 115, 116, 217, 44, 71, 119, 54, 112, 68, 76, 104, 99, 66, 99, 111, 81, 101, 115, 78, 55, 50, 113, 102, 111, 116, 84, 103, 70, 97, 55, 99, 98, 117, 113, 90, 112, 107, 88, 51, 88, 111, 54, 112, 76, 104, 80, 104, 118, 168, 109, 101, 116, 97, 100, 97, 116, 97, 129, 164, 102, 114, 111, 109, 182, 84, 104, 55, 77, 112, 84, 97, 82, 90, 86, 82, 89, 110, 80, 105, 97, 98, 100, 115, 56, 49, 89, 164, 116, 121, 112, 101, 161, 48, 171, 116, 120, 110, 77, 101, 116, 97, 100, 97, 116, 97, 130, 165, 115, 101, 113, 78, 111, 1, 165, 116, 120, 110, 73, 100, 217, 64, 102, 101, 97, 56, 50, 101, 49, 48, 101, 56, 57, 52, 52, 49, 57, 102, 101, 50, 98, 101, 97, 55, 100, 57, 54, 50, 57, 54, 97, 54, 100, 52, 54, 102, 53, 48, 102, 57, 51, 102, 57, 101, 101, 100, 97, 57, 53, 52, 101, 99, 52, 54, 49, 98, 50, 101, 100, 50, 57, 53, 48, 98, 54, 50, 163, 118, 101, 114, 161, 49]
                },
                height: 0,
                count: 1,
                nodes_count: 0,
            };

            request_handler.process_event(Some(RequestEvent::CatchupReq(mt, 2, vec![55, 104, 239, 91, 37, 160, 29, 25, 192, 253, 166, 135, 242, 53, 75, 41, 224, 4, 130, 27, 206, 133, 87, 231, 0, 133, 55, 159, 83, 105, 7, 237])));

            let mut txns: HashMap<String, SJsonValue> = HashMap::new();
            txns.insert("2".to_string(), serde_json::from_str::<SJsonValue>(r#"{"reqSignature":{},"txn":{"data":{"data":{"alias":"Node2","client_port":9704,"blskey":"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk","node_port":9703,"node_ip":"10.0.0.2","services":["VALIDATOR"],"client_ip":"10.0.0.2"},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"},"metadata":{"from":"EbP4aYNeTHL6q385GuVpRV"},"type":"0"},"txnMetadata":{"seqNo":2,"txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc"},"ver":"1"}"#).unwrap());

            let cr = CatchupRep { ledgerId: 0, consProof: Vec::new(), txns };

            request_handler.process_event(Some(RequestEvent::CatchupRep(cr, "Node1".to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
            TestUtils::cleanup_indy_home();
        }

        #[test]
        fn request_handler_process_catchup_reply_event_from_catchup_single_state_works_for_error() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            request_handler.process_event(Some(RequestEvent::CatchupRep(CatchupRep::default(), NODE.to_string())));
            assert_match!(RequestSMWrapper::CatchupSingle(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_timeout_event_from_catchup_single_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestSMWrapper::CatchupSingle(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_terminate_event_from_catchup_single_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_other_event_from_catchup_single_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            request_handler.process_event(Some(RequestEvent::Pong));
            assert_match!(RequestSMWrapper::CatchupSingle(_), request_handler.request_wrapper.unwrap());
        }
    }

    mod full {
        use super::*;

        #[test]
        fn request_handler_process_reply_event_from_full_state_works_for_completed() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reply_event_from_full_state_works_for_not_completed() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Full(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reply_event_from_full_state_works_for_different_replies() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":"11"}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":"22"}"#.to_string(), "n2".to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqnack_event_from_full_state_works_for_completed() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqnack_event_from_full_state_works_for_not_completed() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Full(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reject_event_from_full_state_works_for_completed() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reject_event_from_full_state_works_for_not_completed() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Full(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_timeout_event_from_full_state_works_for_completed() {
            let mut request_handler = _request_handler(1, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_timeout_event_from_full_state_works_for_not_completed() {
            let mut request_handler = _request_handler(1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestSMWrapper::Full(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_reqack_event_from_full_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::ReqACK(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestSMWrapper::Full(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_terminate_event_from_full_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }

        #[test]
        fn request_handler_process_other_event_from_full_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Pong));
            assert_match!(RequestSMWrapper::Full(_), request_handler.request_wrapper.unwrap());
        }
    }

    mod finish {
        use super::*;

        #[test]
        fn request_handler_process_event_from_finish_state_works() {
            let mut request_handler = _request_handler(0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), Ok(REQ_ID.to_string()))));
            request_handler.process_event(Some(RequestEvent::Terminate));
            request_handler.process_event(Some(RequestEvent::Ping));
            assert_match!(RequestSMWrapper::Finish(_), request_handler.request_wrapper.unwrap());
        }
    }
}