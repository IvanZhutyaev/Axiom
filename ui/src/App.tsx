import { useCallback, useEffect, useState } from "react";
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

export function App() {
  const [aql, setAql] = useState(DEFAULT_AQL);
  const [token, setToken] = useState<string | null>(null);
  const [status, setStatus] = useState<string>("");
  const [nodes, , onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  useEffect(() => {
    fetch("/api/v1/auth/token", { method: "POST" })
      .then((r) => r.json())
      .then((d: { access_token: string }) => setToken(d.access_token))
      .catch(() => setStatus("API offline"));
  }, []);

  const onConnect = useCallback(
    (c: Connection) => setEdges((eds) => addEdge(c, eds)),
    [setEdges],
  );

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
        sample_events: [{ temperature: 35.0, sensor: "ui" }],
      }),
    });
    const body = await res.json();
    setStatus(res.ok ? `job ${body.id}` : `error ${res.status}`);
  };

  return (
    <div className="app">
      <header>
        <h1>Axiom</h1>
        <p>Pipeline studio</p>
        <button type="button" onClick={submitJob} disabled={!token}>
          Run pipeline
        </button>
        {status && <span className="status">{status}</span>}
      </header>
      <div className="grid">
        <section className="panel">
          <h2>AQL (Monaco)</h2>
          <Editor
            height="280px"
            defaultLanguage="plaintext"
            value={aql}
            onChange={(v) => setAql(v ?? "")}
            theme="vs-dark"
            options={{ minimap: { enabled: false }, fontSize: 13 }}
          />
        </section>
        <section className="panel flow-panel">
          <h2>Graph (@xyflow)</h2>
          <div style={{ width: "100%", height: 280 }}>
            <ReactFlow
              nodes={nodes}
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
