# nui Phase 0 wire protocol

A deliberately dumb protocol. The goal is to prove the reactive loop, not to be
fast or final. It will be replaced by a direct FFI boundary in Phase 3.

## Transport

- TCP over `127.0.0.1:7000` (the iOS simulator shares the host's loopback).
- **Newline-delimited JSON**: each message is one JSON object on its own line,
  terminated by `\n`.

## Roles

- **Server = the logic.** Owns the state. There is exactly one source of truth.
- **Client = the UI.** Holds only a render mirror of the state.

## Messages

### Server → client: `state` (full snapshot)

Sent immediately on connect, and again after every state change.

```json
{"type": "state", "state": {"count": 0}}
```

Phase 0 sends the **whole state** every time. Diffing is a later optimization.

### Client → server: `event`

Sent when the user interacts with the UI.

```json
{"type": "event", "name": "increment"}
```

Optional `payload` for events that carry data:

```json
{"type": "event", "name": "set", "payload": {"value": 42}}
```

## Loop

```
client connects
server → {"type":"state","state":{"count":0}}      (initial projection)
user taps "Increment"
client → {"type":"event","name":"increment"}
server mutates state (count = 1), broadcasts
server → {"type":"state","state":{"count":1}}       (to all clients)
client updates its observable mirror → SwiftUI re-renders
```

## Notes / invariants

- The client never computes state. It only reflects what the server sends.
- The server may have multiple clients; every state change is broadcast to all
  (so two simulators stay in sync — a nice demo of "logic owns state").
- Either side may disconnect at any time; the other side tolerates it.
