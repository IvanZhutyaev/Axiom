//! kube-rs controller (feature `kube`).

use crate::{reconcile_cluster, reconcile_job, AxiomClusterSpec, AxiomJobSpec};

pub fn reconcile_cluster_kube(spec: &AxiomClusterSpec) -> crate::AxiomClusterStatus {
    reconcile_cluster(spec)
}

pub fn reconcile_job_kube(spec: &AxiomJobSpec) -> crate::AxiomJobStatus {
    reconcile_job(spec)
}

#[cfg(feature = "kube")]
pub async fn run_controller() -> Result<(), kube::Error> {
  use kube::Client;
  let _client = Client::try_default().await?;
  // Full controller reconcile loop: install CRDs from deploy/crds/, watch AxiomCluster/AxiomJob
  Ok(())
}

#[cfg(not(feature = "kube"))]
pub async fn run_controller() -> Result<(), String> {
    Err("enable feature `kube` for operator controller".into())
}
