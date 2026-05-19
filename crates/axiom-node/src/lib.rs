//! Axiom node: wires storage, gossip, API, and pipeline execution.

pub mod config;
pub mod node;

pub use config::{NodeConfig, NodeRole};
pub use node::Node;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "axiom", about = "Axiom stream processing node")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run node with given role
    Run {
        #[arg(long, default_value = "all-in-one")]
        role: String,
        #[arg(long, default_value = "127.0.0.1:8080")]
        api_bind: String,
        #[arg(long, default_value = "127.0.0.1:7946")]
        gossip_bind: String,
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,
        #[arg(long, default_value = "127.0.0.1:9090")]
        grpc_bind: String,
        /// Gossip seed peers (repeatable), e.g. --seed 127.0.0.1:7947
        #[arg(long = "seed")]
        seeds: Vec<String>,
    },
}

pub fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            role,
            api_bind,
            gossip_bind,
            data_dir,
            grpc_bind,
            seeds,
        } => {
            let seeds: Vec<std::net::SocketAddr> = seeds
                .iter()
                .map(|s| s.parse())
                .collect::<Result<_, _>>()?;
            let node_config = NodeConfig {
                role: NodeRole::from_str(&role),
                api_bind,
                gossip_bind,
                data_dir,
                grpc_bind,
                seeds,
            };
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async move {
                let node = Node::new(node_config).await?;
                node.run().await
            })
        }
    }
}
