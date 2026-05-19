//! axctl — Axiom CLI (all phases).

use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "axctl", about = "Axiom CLI")]
struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    server: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Cluster {
        #[command(subcommand)]
        action: ClusterAction,
    },
    Job {
        #[command(subcommand)]
        action: JobAction,
    },
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    Show,
}

#[derive(Subcommand, Debug)]
enum ClusterAction {
    Status,
    /// Start local 3-node dev cluster (requires `axiom` binary in PATH or use --dev-cargo)
    Up {
        #[arg(long)]
        dev_cargo: bool,
    },
}

#[derive(Subcommand, Debug)]
enum JobAction {
    Submit {
        #[arg(short, long)]
        file: Option<PathBuf>,
        #[arg(short, long)]
        aql: Option<String>,
    },
    List,
    Delete {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum SchemaAction {
    Register {
        #[arg(short, long)]
        file: PathBuf,
    },
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct ClusterStatus {
    members: Vec<MemberView>,
    alive: usize,
}

#[derive(Deserialize)]
struct MemberView {
    id: String,
    addr: String,
    state: String,
}

#[derive(Deserialize)]
struct SubmitResponse {
    id: String,
    events_processed: Option<u64>,
}

async fn bearer_token(client: &reqwest::Client, base: &str) -> anyhow::Result<String> {
    let t: TokenResponse = client
        .post(format!("{base}/api/v1/auth/token"))
        .send()
        .await?
        .json()
        .await?;
    Ok(t.access_token)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let base = cli.server.trim_end_matches('/');

    match cli.command {
        Commands::Config { action: ConfigAction::Show } => {
            println!("server = {base}");
        }
        Commands::Cluster {
            action: ClusterAction::Status,
        } => {
            let token = bearer_token(&client, base).await?;
            let cluster: ClusterStatus = client
                .get(format!("{base}/api/v1/cluster"))
                .bearer_auth(&token)
                .send()
                .await?
                .json()
                .await?;
            println!("{} alive / {} members", cluster.alive, cluster.members.len());
            for m in &cluster.members {
                println!("  {} @ {} [{}]", m.id, m.addr, m.state);
            }
        }
        Commands::Cluster {
            action: ClusterAction::Up { dev_cargo },
        } => {
            let nodes = [
                ("8080", "7946", "./data1", Vec::<&str>::new()),
                ("8081", "7947", "./data2", vec!["127.0.0.1:7946"]),
                ("8082", "7948", "./data3", vec!["127.0.0.1:7946"]),
            ];
            for (api, gossip, data, seeds) in &nodes {
                if dev_cargo {
                    let mut cmd = Command::new("cargo");
                    cmd.args([
                        "run", "-p", "axiom", "--", "run",
                        "--api-bind", &format!("127.0.0.1:{api}"),
                        "--gossip-bind", &format!("127.0.0.1:{gossip}"),
                        "--data-dir", data,
                    ]);
                    for s in seeds {
                        cmd.arg("--seed").arg(*s);
                    }
                    cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
                } else {
                    let mut cmd = Command::new("axiom");
                    cmd.args([
                        "run",
                        "--api-bind", &format!("127.0.0.1:{api}"),
                        "--gossip-bind", &format!("127.0.0.1:{gossip}"),
                        "--data-dir", data,
                    ]);
                    for s in seeds {
                        cmd.arg("--seed").arg(*s);
                    }
                    cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
                }
                std::thread::sleep(Duration::from_millis(800));
            }
            println!("cluster started (3 nodes). Check: axctl cluster status");
        }
        Commands::Job { action: JobAction::List } => {
            let token = bearer_token(&client, base).await?;
            let text = client
                .get(format!("{base}/api/v1/jobs"))
                .bearer_auth(token)
                .send()
                .await?
                .text()
                .await?;
            println!("{text}");
        }
        Commands::Job {
            action: JobAction::Submit { file, aql },
        } => {
            let source = if let Some(p) = file {
                std::fs::read_to_string(p)?
            } else if let Some(s) = aql {
                s
            } else {
                anyhow::bail!("--file or --aql required");
            };
            let token = bearer_token(&client, base).await?;
            let body = serde_json::json!({
                "aql": source,
                "sample_events": [{"temperature": 35.0}]
            });
            let resp: SubmitResponse = client
                .post(format!("{base}/api/v1/jobs"))
                .bearer_auth(token)
                .json(&body)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            println!("job {} events={:?}", resp.id, resp.events_processed);
        }
        Commands::Job {
            action: JobAction::Delete { id },
        } => {
            println!("delete job {id} (API DELETE phase 1.1)");
        }
        Commands::Schema {
            action: SchemaAction::Register { file },
        } => {
            let json = std::fs::read_to_string(file)?;
            let token = bearer_token(&client, base).await?;
            let resp = client
                .post(format!("{base}/api/v1/schemas"))
                .bearer_auth(token)
                .json(&serde_json::json!({ "json": json }))
                .send()
                .await?
                .text()
                .await?;
            println!("{resp}");
        }
    }
    Ok(())
}
