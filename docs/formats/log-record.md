# Event log record format

## Record layout

| Offset | Size | Field |
|--------|------|-------|
| 0 | 8 | Monotonic offset (LE u64) |
| 8 | 4 | Payload length (LE u32) |
| 12 | 4 | CRC32 of payload (LE u32) |
| 16 | N | Event bytes (schema-encoded or raw JSON in dev) |

Segments are named `segment-{id}.log` and may be compressed with Snappy/LZ4 in later phases.
