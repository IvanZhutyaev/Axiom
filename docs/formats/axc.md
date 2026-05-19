# AXC bytecode container format (v1)

## Header

| Offset | Size | Field |
|--------|------|-------|
| 0 | 4 | Magic `AXC\x01` |
| 4 | 2 | Format version (LE u16), current `1` |
| 6 | 4 | Payload length (LE u32) |
| 10 | 4 | CRC32 of payload (LE u32) |
| 14 | N | JSON payload |

## Payload (JSON)

- `version`, `pipeline_name`, `operators[]`, `code[]`, `constants[]`, `sources[]`, `sinks[]`
- Each instruction: `{ "op": "<Opcode>", "operand": ... }`

Backward compatibility: readers accept `version <= supported` and ignore unknown JSON fields.
