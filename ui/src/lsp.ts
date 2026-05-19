export type LspDiagnostic = { line: number; message: string; severity: string };

export function connectAqlLsp(
  url: string,
  onDiagnostics: (d: LspDiagnostic[]) => void,
): WebSocket | null {
  try {
    const ws = new WebSocket(url);
    ws.onmessage = (ev) => {
      const data = JSON.parse(ev.data as string) as { diagnostics: LspDiagnostic[] };
      onDiagnostics(data.diagnostics);
    };
    return ws;
  } catch {
    return null;
  }
}

export function lspDidChange(ws: WebSocket | null, source: string) {
  ws?.send(JSON.stringify({ method: "textDocument/didChange", source }));
}
