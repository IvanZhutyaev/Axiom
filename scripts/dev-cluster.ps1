# Start 3-node Axiom dev cluster (requires built binaries or cargo run)
param(
    [switch]$Build
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

if ($Build) {
    cargo build -p axiom
}

function Start-Node($api, $gossip, $data, $seeds) {
    $seedArgs = @()
    foreach ($s in $seeds) { $seedArgs += @("--seed", $s) }
    Start-Process -FilePath "cargo" -ArgumentList @(
        "run", "-p", "axiom", "--",
        "run",
        "--api-bind", $api,
        "--gossip-bind", $gossip,
        "--data-dir", $data
    ) + $seedArgs -NoNewWindow
}

Start-Node "127.0.0.1:8080" "127.0.0.1:7946" "./data1" @()
Start-Sleep -Seconds 2
Start-Node "127.0.0.1:8081" "127.0.0.1:7947" "./data2" @("127.0.0.1:7946")
Start-Node "127.0.0.1:8082" "127.0.0.1:7948" "./data3" @("127.0.0.1:7946")

Write-Host "Cluster started. Check: cargo run -p axctl -- --server http://127.0.0.1:8080 cluster status"
