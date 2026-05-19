import type { Edge, Node } from "@xyflow/react";

/** Layered Sugiyama-style layout for pipeline DAG nodes. */
export function layoutSugiyama(nodes: Node[], edges: Edge[]): Node[] {
  const layers = new Map<string, number>();
  const out = new Map<string, string[]>();
  for (const e of edges) {
    out.set(e.source, [...(out.get(e.source) ?? []), e.target]);
  }
  const roots = nodes.filter((n) => !edges.some((e) => e.target === n.id));
  const queue = roots.map((n) => n.id);
  for (const id of queue) {
    layers.set(id, 0);
  }
  while (queue.length) {
    const id = queue.shift()!;
    const layer = layers.get(id) ?? 0;
    for (const t of out.get(id) ?? []) {
      layers.set(t, Math.max(layers.get(t) ?? 0, layer + 1));
      queue.push(t);
    }
  }
  const byLayer = new Map<number, string[]>();
  for (const n of nodes) {
    const l = layers.get(n.id) ?? 0;
    byLayer.set(l, [...(byLayer.get(l) ?? []), n.id]);
  }
  return nodes.map((n) => {
    const l = layers.get(n.id) ?? 0;
    const row = byLayer.get(l) ?? [];
    const idx = row.indexOf(n.id);
    return {
      ...n,
      position: { x: l * 220, y: idx * 100 },
    };
  });
}
