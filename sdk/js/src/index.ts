export class AxiomClient {
  constructor(private base = "http://127.0.0.1:8080") {}

  private token?: string;

  async auth(): Promise<string> {
    const r = await fetch(`${this.base}/api/v1/auth/token`, { method: "POST" });
    const j = (await r.json()) as { access_token: string };
    this.token = j.access_token;
    return this.token;
  }

  async submitJob(aql: string, sampleEvents: unknown[] = []): Promise<unknown> {
    if (!this.token) await this.auth();
    const r = await fetch(`${this.base}/api/v1/jobs`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${this.token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ aql, sample_events: sampleEvents }),
    });
    return r.json();
  }
}
