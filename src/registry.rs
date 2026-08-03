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
    pub node_class: Option<u8>,
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
        self.by_identity.insert(record.identity.clone(), record);
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

    /// Pending discoveries awaiting TerraTactics cloud / admin claim.
    pub fn pending(&self) -> Vec<DeviceRecord> {
        self.by_identity
            .values()
            .filter(|d| d.claim == ClaimState::Pending)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_claimed_updates_addr() {
        let mut reg = DeviceRegistry::new();
        reg.upsert(DeviceRecord {
            identity: "TL-1".into(),
            routing_addr: 0,
            claim: ClaimState::Pending,
            last_seen_seq: 1,
            node_class: Some(1),
        });
        assert!(reg.mark_claimed("TL-1", 42));
        let rec = reg.get("TL-1").unwrap();
        assert_eq!(rec.claim, ClaimState::Claimed);
        assert_eq!(rec.routing_addr, 42);
        assert!(!reg.mark_claimed("missing", 1));
    }
}
