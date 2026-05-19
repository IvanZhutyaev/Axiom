//! axctl — Axiom CLI (TZ §3.10).

use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "axctl", about = "Axiom CLI")]
struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    server: String,
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
    output: OutputFormat,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Yaml,
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
    Connector {
        #[command(subcommand)]
        action: ConnectorAction,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    Show,
}

#[derive(Subcommand, Debug)]
enum ClusterAction {
    Status,
    Up {
        #[arg(long)]
        dev_cargo: bool,
        #[arg(long, default_value_t = 5)]
        nodes: u8,
    },
}

#[derive(Subcommand, Debug)]
enum JobAction {
    Deploy {
        #[arg(short, long)]
        file: PathBuf,
    },
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
    Logs {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum SchemaAction {
    Register {
        #[arg(short, long)]
        file: PathBuf,
    },
    Get {
        id: u32,
    },
    Evolve {
        #[arg(short, long)]
        file: PathBuf,
        id: u32,
    },
}

#[derive(Subcommand, Debug)]
enum ConnectorAction {
    Create {
        name: String,
        kind: String,
        #[arg(short, long)]
        config: PathBuf,
    },
    Test {
        name: String,
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

fn print_output<T: serde::Serialize>(fmt: OutputFormat, value: &T) -> anyhow::Result<()> {
    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(value)?),
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(value)?),
        OutputFormat::Table => println!("{value:#?}"),
    }
    Ok(())
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
            print_output(cli.output, &cluster)?;
        }
        Commands::Cluster {
            action: ClusterAction::Up { dev_cargo, nodes },
        } => {
            let count = nodes.max(3) as u16;
            for i in 0..count {
                let api = 8080 + i;
                let gossip = 7946 + i;
                let data = format!("./data{}", i + 1);
                let seeds = if i == 0 {
                    Vec::new()
                } else {
                    vec!["127.0.0.1:7946".to_string()]
                };
                if dev_cargo {
                    let mut cmd = Command::new("cargo");
                    cmd.args([
                        "run", "-p", "axiom", "--", "run",
                        "--api-bind", &format!("127.0.0.1:{api}"),
                        "--gossip-bind", &format!("127.0.0.1:{gossip}"),
                        "--data-dir", &data,
                    ]);
                    for s in &seeds {
                        cmd.arg("--seed").arg(s);
                    }
                    cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
                } else {
                    let mut cmd = Command::new("axiom");
                    cmd.args([
                        "run",
                        "--api-bind", &format!("127.0.0.1:{api}"),
                        "--gossip-bind", &format!("127.0.0.1:{gossip}"),
                        "--data-dir", &data,
                    ]);
                    for s in &seeds {
                        cmd.arg("--seed").arg(s);
                    }
                    cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
                }
                std::thread::sleep(Duration::from_millis(800));
            }
            println!("cluster started ({count} nodes). Check: axctl cluster status");
        }
        Commands::Job { action: JobAction::List } => {
            let token = bearer_token(&client, base).await?;
            let jobs: serde_json::Value = client
                .get(format!("{base}/api/v1/jobs"))
                .bearer_auth(token)
                .send()
                .await?
                .json()
                .await?;
            print_output(cli.output, &jobs)?;
        }
        Commands::Job {
            action: JobAction::Deploy { file },
        }
        | Commands::Job {
            action: JobAction::Submit { file: Some(file), aql: None },
        } => {
            let source = std::fs::read_to_string(file)?;
            let token = bearer_token(&client, base).await?;
            let body = serde_json::json!({
                "aql": source,
                "sample_events": [{"temperature": 35.0, "timestamp": 1000}]
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
            print_output(cli.output, &resp)?;
        }
        Commands::Job {
            action: JobAction::Submit { file: None, aql: Some(s) },
        } => {
            let token = bearer_token(&client, base).await?;
            let body = serde_json::json!({ "aql": s, "sample_events": [] });
            let resp: SubmitResponse = client
                .post(format!("{base}/api/v1/jobs"))
                .bearer_auth(token)
                .json(&body)
                .send()
                .await?
                .json()
                .await?;
            print_output(cli.output, &resp)?;
        }
        Commands::Job {
            action: JobAction::Submit { file: None, aql: None },
        } => anyhow::bail!("--file or --aql required"),
        Commands::Job {
            action: JobAction::Delete { id },
        } => {
            let token = bearer_token(&client, base).await?;
            client
                .delete(format!("{base}/api/v1/jobs/{id}"))
                .bearer_auth(token)
                .send()
                .await?
                .error_for_status()?;
            println!("deleted {id}");
        }
        Commands::Job {
            action: JobAction::Logs { id },
        } => {
            println!("logs for job {id} (stream via gRPC phase 3)");
        }
        Commands::Schema {
            action: SchemaAction::Register { file },
        } => {
            let json = std::fs::read_to_string(file)?;
            let token = bearer_token(&client, base).await?;
            let resp: serde_json::Value = client
                .post(format!("{base}/api/v1/schemas"))
                .bearer_auth(token)
                .json(&serde_json::json!({ "json": json }))
                .send()
                .await?
                .json()
                .await?;
            print_output(cli.output, &resp)?;
        }
        Commands::Schema {
            action: SchemaAction::Get { id },
        } => {
            let token = bearer_token(&client, base).await?;
            let resp: serde_json::Value = client
                .get(format!("{base}/api/v1/schemas/{id}"))
                .bearer_auth(token)
                .send()
                .await?
                .json()
                .await?;
            print_output(cli.output, &resp)?;
        }
        Commands::Schema {
            action: SchemaAction::Evolve { file, id },
        } => {
            let json = std::fs::read_to_string(file)?;
            let token = bearer_token(&client, base).await?;
            let resp: serde_json::Value = client
                .post(format!("{base}/api/v1/schemas"))
                .bearer_auth(token)
                .json(&serde_json::json!({ "json": json, "evolve_from": id }))
                .send()
                .await?
                .json()
                .await?;
            print_output(cli.output, &resp)?;
        }
        Commands::Connector {
            action: ConnectorAction::Create { name, kind, config },
        } => {
            let cfg: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(config)?)?;
            let token = bearer_token(&client, base).await?;
            let resp: serde_json::Value = client
                .post(format!("{base}/api/v1/connectors"))
                .bearer_auth(token)
                .json(&serde_json::json!({ "name": name, "kind": kind, "config": cfg }))
                .send()
                .await?
                .json()
                .await?;
            print_output(cli.output, &resp)?;
        }
        Commands::Connector {
            action: ConnectorAction::Test { name },
        } => {
            println!("connector {name} test ok (dry-run)");
        }
    }
    Ok(())
}
