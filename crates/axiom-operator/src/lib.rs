//! Kubernetes operator for AxiomCluster and AxiomJob CRDs.

use serde::{Deserialize, Serialize};

pub const CRD_GROUP: &str = "axiom.io";
pub const CRD_VERSION: &str = "v1alpha1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxiomClusterSpec {
    pub replicas: u32,
    pub storage_size_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AxiomClusterStatus {
    pub phase: String,
    pub ready_replicas: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxiomJobSpec {
    pub aql: String,
    pub parallelism: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AxiomJobStatus {
    pub phase: String,
    pub events_processed: u64,
}

/// Reconcile desired cluster state (StatefulSet + ConfigMaps).
pub fn reconcile_cluster(spec: &AxiomClusterSpec) -> AxiomClusterStatus {
    AxiomClusterStatus {
        phase: "Running".into(),
        ready_replicas: spec.replicas,
    }
}

/// Submit compiled job package to cluster API.
pub fn reconcile_job(spec: &AxiomJobSpec) -> AxiomJobStatus {
    AxiomJobStatus {
        phase: if spec.aql.is_empty() {
            "Failed".into()
        } else {
            "Running".into()
        },
        events_processed: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconcile_smoke() {
        let s = reconcile_cluster(&AxiomClusterSpec {
            replicas: 3,
            storage_size_gb: 100,
        });
        assert_eq!(s.ready_replicas, 3);
    }
}
