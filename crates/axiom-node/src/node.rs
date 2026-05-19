//! Node lifecycle: API, gRPC, gossip, Raft metadata, jobs.

use crate::config::{NodeConfig, NodeRole};
use aql_compile::compile;
use avm_bytecode::save_axc;
use axiom_api::{ApiState, ClusterMemberView, JobRecord, JobStatus};
use axiom_engine::{
    BarrierCoordinator, EventKey, IdempotencyStore, PipelineRunner, Scheduler,
};
use axiom_gossip::network::{encode_message, GossipMesh};
use axiom_gossip::{GossipMessage, Member, MemberState, SwimNode};
use axiom_raft::{RaftNode, Role as RaftRole};
use axiom_schema::SchemaRegistry;
use axiom_storage_log::EventLog;
use axiom_storage_lsm::LsmStore;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Node {
    config: NodeConfig,
    state: Arc<RwLock<ApiState>>,
    swim: Arc<SwimNode>,
    raft: Arc<RwLock<RaftNode>>,
    scheduler: Arc<RwLock<Scheduler>>,
    barriers: Arc<RwLock<BarrierCoordinator>>,
    idempotency: Arc<RwLock<IdempotencyStore>>,
    log: Arc<RwLock<EventLog>>,
    lsm: Arc<RwLock<LsmStore>>,
    _schemas: Arc<RwLock<SchemaRegistry>>,
    gossip_mesh: Arc<GossipMesh>,
    seeds: Vec<SocketAddr>,
}

impl Node {
    pub async fn new(config: NodeConfig) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        let log = EventLog::open(config.data_dir.join("log"))?;
        let lsm = LsmStore::open(config.data_dir.join("lsm"))?;
        let gossip_addr: SocketAddr = config.gossip_bind.parse()?;
        let node_id = format!("axiom-{}", gossip_addr.port());
        let swim = Arc::new(SwimNode::new(node_id.clone(), gossip_addr));
        let gossip_mesh = Arc::new(GossipMesh::new(gossip_addr, 1));
        let seeds = config.seeds.clone();

        let mut members = vec![ClusterMemberView {
            id: node_id.clone(),
            addr: gossip_addr.to_string(),
            state: "alive".into(),
        }];
        for seed in &seeds {
            swim.join(Member::new(format!("axiom-{}", seed.port()), *seed))
                .await;
            members.push(ClusterMemberView {
                id: format!("axiom-{}", seed.port()),
                addr: seed.to_string(),
                state: "alive".into(),
            });
        }

        let raft_id = gossip_addr.port() as u64;
        let peer_ids: Vec<u64> = seeds.iter().map(|s| s.port() as u64).collect();
        let raft = Arc::new(RwLock::new(RaftNode::new(raft_id, peer_ids)));

        let mut api = ApiState::default();
        api.node_id = node_id;
        api.members = members;
        let state = Arc::new(RwLock::new(api));

        Ok(Self {
            config,
            state,
            swim,
            raft,
            scheduler: Arc::new(RwLock::new(Scheduler::default())),
            barriers: Arc::new(RwLock::new(BarrierCoordinator::new(3))),
            idempotency: Arc::new(RwLock::new(IdempotencyStore::new())),
            log: Arc::new(RwLock::new(log)),
            lsm: Arc::new(RwLock::new(lsm)),
            _schemas: Arc::new(RwLock::new(SchemaRegistry::new())),
            gossip_mesh,
            seeds,
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!(role = ?self.config.role, "axiom node starting");

        if matches!(self.config.role, NodeRole::Master | NodeRole::AllInOne) {
            self.raft.write().await.become_leader();
        }

        if matches!(
            self.config.role,
            NodeRole::AllInOne | NodeRole::Master | NodeRole::Worker
        ) {
            self.seed_demo_job().await?;
            self.checkpoint_state().await?;
        }

        self.spawn_gossip();
        self.spawn_grpc();

        let api_state = self.state.clone();
        axiom_api::serve(&self.config.api_bind, api_state).await
    }

    fn spawn_grpc(&self) {
        let addr: SocketAddr = self.config.grpc_bind.parse().expect("grpc bind");
        tokio::spawn(async move {
            if let Err(e) = axiom_grpc::serve_grpc(addr).await {
                tracing::warn!("gRPC server stopped: {e}");
            }
        });
    }

    fn spawn_gossip(&self) {
        let swim = self.swim.clone();
        let state = self.state.clone();
        let mesh = self.gossip_mesh.clone();
        let seeds = self.seeds.clone();

        tokio::spawn(async move {
            loop {
                swim.run_probe_round().await;
                let members = swim
                    .members_snapshot()
                    .await
                    .into_iter()
                    .map(|m| ClusterMemberView {
                        id: m.id,
                        addr: m.addr.to_string(),
                        state: match m.state {
                            MemberState::Alive => "alive",
                            MemberState::Suspect => "suspect",
                            MemberState::Dead => "dead",
                        }
                        .into(),
                    })
                    .collect();
                state.write().await.members = members;
                let _ = encode_message(&swim.make_ping());
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        for seed in seeds {
            let swim = swim.clone();
            let mesh = mesh.clone();
            tokio::spawn(async move {
                let (mut local, mut peer) = mesh.connect_peer(seed);
                loop {
                    let ping = GossipMessage::Ping {
                        from: swim.local_id.clone(),
                        seq: 0,
                    };
                    if axiom_gossip::network::sim_send(&local, &ping).await.is_ok() {
                        if let Ok(msg) = axiom_gossip::network::sim_recv(&mut peer).await {
                            swim.handle_message(msg).await;
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            });
        }
    }

    async fn checkpoint_state(&self) -> anyhow::Result<()> {
        let barrier = self.barriers.write().await.inject_barrier();
        self.barriers.write().await.ack(barrier, "source");
        self.barriers.write().await.ack(barrier, "filter");
        self.barriers.write().await.ack(barrier, "sink");
        if let Some(cp) = self.barriers.write().await.finalize(barrier) {
            let mut lsm = self.lsm.write().await;
            lsm.put(
                b"checkpoint.id".to_vec(),
                cp.0.as_bytes().to_vec(),
            )?;
            lsm.checkpoint()?;
        }
        let offset = self.log.read().await.latest_offset();
        self.log
            .write()
            .await
            .flush_checkpoint_marker(&offset.to_le_bytes())?;
        Ok(())
    }

    async fn seed_demo_job(&self) -> anyhow::Result<()> {
        const DEMO: &str = include_str!("../../../examples/sensor.aql");
        let compiled = compile(DEMO)?;
        let mut buf = Vec::new();
        save_axc(&compiled.module, &mut Cursor::new(&mut buf))?;

        let nodes: Vec<String> = self
            .state
            .read()
            .await
            .members
            .iter()
            .map(|m| m.id.clone())
            .collect();
        let spec = self
            .scheduler
            .write()
            .await
            .submit(buf, 2);
        self.scheduler.write().await.assign_tasks(&nodes);

        if self.raft.read().await.state.role == RaftRole::Leader {
            self.raft
                .write()
                .await
                .propose(DEMO.as_bytes().to_vec());
        }

        let runner = PipelineRunner::new(compiled.module);
        let events = [
            serde_json::json!({"temperature": 25.0}),
            serde_json::json!({"temperature": 35.0}),
        ];
        let mut emitted = Vec::new();
        for ev in events {
            let key = EventKey {
                checkpoint_id: uuid::Uuid::new_v4(),
                sequence: emitted.len() as u64,
            };
            if !self.idempotency.write().await.should_process(key) {
                continue;
            }
            let r = runner.run_event(PipelineRunner::json_to_event(&ev))?;
            emitted.extend(r.emitted);
        }

        {
            let mut log = self.log.write().await;
            for e in &emitted {
                log.append(serde_json::to_vec(e)?)?;
            }
        }

        let job_id = uuid::Uuid::new_v4().to_string();
        let mut st = self.state.write().await;
        st.metrics.inc_events(emitted.len() as u64);
        st.jobs.insert(
            job_id.clone(),
            JobRecord {
                id: job_id,
                status: JobStatus::Running,
                aql: DEMO.to_string(),
                events_processed: emitted.len() as u64,
            },
        );
        let _ = spec;
        Ok(())
    }
}
