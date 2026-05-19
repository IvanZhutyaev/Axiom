# Axiom wire protocol (UDP reliable transport)

## Packet header (16 bytes)

| Offset | Size | Field |
|--------|------|-------|
| 0 | 1 | Flags: SYN, ACK, FIN, DATA, NACK |
| 1 | 2 | Connection ID (LE u16) |
| 3 | 4 | Sequence number (LE u32) |
| 7 | 2 | Payload length (LE u16) |
| 9 | 4 | CRC32 of payload (LE u32) |
| 13 | N | Payload (max MTU − header) |

## Handshake

1. Client sends `SYN` with conn_id and window params.
2. Server replies `SYN|ACK`.
3. Data phase uses selective ACK per sequence.

## TLS

After handshake, implementations may upgrade the logical session to TLS 1.3 (rustls) on a dedicated port or encapsulation layer.
