use services::pool::events::NetworkerEvent;
use services::pool::events::ConsensusCollectorEvent;

pub trait Networker {
    fn new() -> Self;
    fn process_event(&self, pe: Option<NetworkerEvent>) -> Option<ConsensusCollectorEvent>;
}

pub struct ZMQNetworker {}

impl Networker for ZMQNetworker {
    fn new() -> Self {
        Networker {}
    }

    fn process_event(&self, pe: Option<NetworkerEvent>) -> Option<ConsensusCollectorEvent> {
        match pe {
            Some(NetworkerEvent::SendAllRequest) => Some(ConsensusCollectorEvent::StartConsensus),
            Some(NetworkerEvent::SendOneRequest) => None,
            None => None
        }
    }
}

pub struct MockNetworker {}

impl Networker for MockNetworker {
    fn new() -> Self {
        unimplemented!()
    }

    fn process_event(&self, pe: Option<NetworkerEvent>) -> Option<ConsensusCollectorEvent> {
        unimplemented!()
    }
}



mod networker_tests {
    use super::*;
    
    #[test]
    pub fn networker_new_works() {
        Networker::new();
    }

    #[test]
    pub fn networker_process_event_works() {
        let networker = Networker::new();
        networker.process_event(None);
    }
}