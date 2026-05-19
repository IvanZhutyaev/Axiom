"""Minimal Axiom REST client (phase 3)."""

from __future__ import annotations

import json
import urllib.request


class AxiomClient:
    def __init__(self, base_url: str = "http://127.0.0.1:8080") -> None:
        self.base = base_url.rstrip("/")
        self._token: str | None = None

    def _token_headers(self) -> dict[str, str]:
        if not self._token:
            req = urllib.request.Request(
                f"{self.base}/api/v1/auth/token",
                method="POST",
                headers={"Content-Type": "application/json"},
                data=b"{}",
            )
            with urllib.request.urlopen(req) as resp:
                data = json.loads(resp.read())
            self._token = data["access_token"]
        return {"Authorization": f"Bearer {self._token}"}

    def submit_job(self, aql: str, sample_events: list | None = None) -> dict:
        body = {"aql": aql, "sample_events": sample_events or []}
        req = urllib.request.Request(
            f"{self.base}/api/v1/jobs",
            method="POST",
            headers={**self._token_headers(), "Content-Type": "application/json"},
            data=json.dumps(body).encode(),
        )
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read())
