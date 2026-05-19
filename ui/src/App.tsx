import { useCallback, useEffect, useMemo, useState } from "react";
import Editor from "@monaco-editor/react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  addEdge,
  useEdgesState,
  useNodesState,
  type Connection,
  type Edge,
  type Node,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { layoutSugiyama } from "./sugiyama";
import { connectAqlLsp, lspDidChange, type LspDiagnostic } from "./lsp";

const DEFAULT_AQL = `source "sensor_data"
|> filter(temperature > 30.0)
|> window(tumbling, size=5s)
   aggregate(avg_temp = avg(temperature), count = count(*))
|> sink "alerts"`;

const initialNodes: Node[] = [
  { id: "1", position: { x: 0, y: 0 }, data: { label: "source" }, type: "input" },
  { id: "2", position: { x: 200, y: 0 }, data: { label: "filter" } },
  { id: "3", position: { x: 400, y: 0 }, data: { label: "window" } },
  { id: "4", position: { x: 600, y: 0 }, data: { label: "sink" }, type: "output" },
];
const initialEdges: Edge[] = [
  { id: "e1-2", source: "1", target: "2" },
  { id: "e2-3", source: "2", target: "3" },
  { id: "e3-4", source: "3", target: "4" },
];

function aqlToGraph(aql: string): { nodes: Node[]; edges: Edge[] } {
  const stages = aql.split("|>").map((s) => s.trim()).filter(Boolean);
  const nodes: Node[] = stages.map((s, i) => ({
    id: String(i + 1),
    position: { x: i * 200, y: 0 },
    data: { label: s.split("(")[0] || s },
    type: i === 0 ? "input" : i === stages.length - 1 ? "output" : undefined,
  }));
  const edges: Edge[] = [];
  for (let i = 0; i < nodes.length - 1; i++) {
    edges.push({ id: `e${i}`, source: nodes[i].id, target: nodes[i + 1].id });
  }
  return { nodes, edges };
}

export function App() {
  const [aql, setAql] = useState(DEFAULT_AQL);
  const [token, setToken] = useState<string | null>(null);
  const [status, setStatus] = useState<string>("");
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const [diags, setDiags] = useState<LspDiagnostic[]>([]);
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [lspWs, setLspWs] = useState<WebSocket | null>(null);

  const laidOutNodes = useMemo(() => layoutSugiyama(nodes, edges), [nodes, edges]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const code = params.get("code");
    if (code) {
      setToken(code);
      setStatus("OAuth2 code received");
      return;
    }
    fetch("/api/v1/auth/token", { method: "POST" })
      .then((r) => r.json())
      .then((d: { access_token: string }) => setToken(d.access_token))
      .catch(() => setStatus("API offline"));
  }, []);

  useEffect(() => {
    const ws = connectAqlLsp(`ws://${window.location.host}/lsp`, setDiags);
    setLspWs(ws);
    return () => ws?.close();
  }, []);

  useEffect(() => {
    lspDidChange(lspWs, aql);
    const { nodes: n, edges: e } = aqlToGraph(aql);
    setNodes(n);
    setEdges(e);
  }, [aql, lspWs, setNodes, setEdges]);

  const onConnect = useCallback(
    (c: Connection) => setEdges((eds) => addEdge(c, eds)),
    [setEdges],
  );

  const loginOidc = () => {
    const clientId = "axiom-ui";
    const redirect = encodeURIComponent(window.location.origin);
    window.location.href = `/oauth2/authorize?client_id=${clientId}&redirect_uri=${redirect}&response_type=code`;
  };

  const submitJob = async () => {
    if (!token) return;
    setStatus("submitting…");
    const res = await fetch("/api/v1/jobs", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        aql,
        sample_events: [{ temperature: 35.0, sensor: "ui", timestamp: 1000 }],
      }),
    });
    const body = await res.json();
    setStatus(res.ok ? `job ${body.id}` : `error ${res.status}`);
  };

  return (
    <div className={`app theme-${theme}`}>
      <header>
        <h1>Axiom</h1>
        <p>Pipeline studio</p>
        <button type="button" onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
          {theme === "dark" ? "Light" : "Dark"}
        </button>
        <button type="button" onClick={loginOidc}>
          OAuth2
        </button>
        <button type="button" onClick={submitJob} disabled={!token}>
          Run pipeline
        </button>
        {status && <span className="status">{status}</span>}
      </header>
      {diags.length > 0 && (
        <ul className="diagnostics">
          {diags.map((d, i) => (
            <li key={i}>
              {d.severity}: {d.message}
            </li>
          ))}
        </ul>
      )}
      <div className="grid">
        <section className="panel">
          <h2>AQL</h2>
          <Editor
            height="280px"
            defaultLanguage="plaintext"
            value={aql}
            onChange={(v) => setAql(v ?? "")}
            theme={theme === "dark" ? "vs-dark" : "light"}
            options={{ minimap: { enabled: false }, fontSize: 13 }}
          />
        </section>
        <section className="panel flow-panel">
          <h2>Graph</h2>
          <div style={{ width: "100%", height: 280 }}>
            <ReactFlow
              nodes={laidOutNodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              fitView
            >
              <Background />
              <Controls />
              <MiniMap />
            </ReactFlow>
          </div>
        </section>
      </div>
    </div>
  );
}
