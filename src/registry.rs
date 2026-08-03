//! Local device registry (discovered / claimed TerraLink nodes).

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimState {
    Pending,
    Claimed,
}

#[derive(Debug, Clone)]
pub struct DeviceRecord {
    pub identity: String,
    pub routing_addr: u16,
    pub claim: ClaimState,
    pub last_seen_seq: u16,
}

#[derive(Debug, Default)]
pub struct DeviceRegistry {
    by_identity: HashMap<String, DeviceRecord>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, record: DeviceRecord) {
        self.by_identity
            .insert(record.identity.clone(), record);
    }

    pub fn get(&self, identity: &str) -> Option<&DeviceRecord> {
        self.by_identity.get(identity)
    }

    pub fn list(&self) -> Vec<DeviceRecord> {
        self.by_identity.values().cloned().collect()
    }

    pub fn mark_claimed(&mut self, identity: &str, routing_addr: u16) -> bool {
        if let Some(rec) = self.by_identity.get_mut(identity) {
            rec.claim = ClaimState::Claimed;
            rec.routing_addr = routing_addr;
            true
        } else {
            false
        }
    }
}
