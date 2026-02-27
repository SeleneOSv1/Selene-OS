# Selene Adapter (HTTP + gRPC)

Small network adapter crate that routes app/server voice requests into Selene OS ingress:

- `AppServerIngressRuntime::run_voice_turn(...)`
- which calls `SimulationExecutor::execute_os_voice_live_turn(...)` as default live path.

## Run

HTTP adapter:

```bash
cargo run -p selene_adapter --bin selene_adapter_http
```

gRPC adapter:

```bash
cargo run -p selene_adapter --bin selene_adapter_grpc
```

Optional bind overrides:

```bash
SELENE_HTTP_BIND=127.0.0.1:8080 cargo run -p selene_adapter --bin selene_adapter_http
SELENE_GRPC_BIND=127.0.0.1:50051 cargo run -p selene_adapter --bin selene_adapter_grpc
```

Persistent adapter store journal (default path is used when unset):

```bash
SELENE_ADAPTER_STORE_PATH=.selene/adapter/voice_turns.jsonl
```

Continuous sync worker loop (enabled by default for iOS/Android/Desktop adapter traffic):

```bash
SELENE_ADAPTER_SYNC_WORKER_ENABLED=true
SELENE_ADAPTER_SYNC_WORKER_INTERVAL_MS=1000
```

`SELENE_ADAPTER_SYNC_WORKER_INTERVAL_MS` must be between `100` and `60000`.

Automatic improvement + builder handoff from sync failures (enabled by default):

```bash
SELENE_ADAPTER_AUTO_BUILDER_ENABLED=true
```

Optional Voice-ID embedding gate overrides (platform/channel hard gate profile):

```bash
SELENE_VID_GATE_GLOBAL_DEFAULT=required \
SELENE_VID_GATE_IOS_EXPLICIT=required \
SELENE_VID_GATE_IOS_WAKE=required \
SELENE_VID_GATE_ANDROID_EXPLICIT=required \
SELENE_VID_GATE_ANDROID_WAKE=required \
SELENE_VID_GATE_DESKTOP_EXPLICIT=optional \
SELENE_VID_GATE_DESKTOP_WAKE=optional \
cargo run -p selene_adapter --bin selene_adapter_http
```

Accepted gate values: `required` or `optional`.

## HTTP example (`curl`)

```bash
curl -sS \
  -X POST http://127.0.0.1:8080/v1/voice/turn \
  -H 'content-type: application/json' \
  -d '{
    "correlation_id": 1001,
    "turn_id": 2001,
    "app_platform": "IOS",
    "trigger": "EXPLICIT",
    "actor_user_id": "tenant_a:user_1",
    "tenant_id": "tenant_a",
    "device_id": "device_ios_1",
    "now_ns": 3,
    "audio_capture_ref": {
      "stream_id": 777,
      "pre_roll_buffer_id": 1,
      "t_start_ns": 1,
      "t_end_ns": 3,
      "t_candidate_start_ns": 2,
      "t_confirmed_ns": 3
    }
  }'
```

Desktop request example:

```bash
curl -sS \
  -X POST http://127.0.0.1:8080/v1/voice/turn \
  -H 'content-type: application/json' \
  -d '{
    "correlation_id": 1002,
    "turn_id": 2002,
    "app_platform": "DESKTOP",
    "trigger": "EXPLICIT",
    "actor_user_id": "tenant_a:user_desktop",
    "tenant_id": "tenant_a",
    "device_id": "device_desktop_1",
    "now_ns": 4,
    "audio_capture_ref": {
      "stream_id": 778,
      "pre_roll_buffer_id": 1,
      "t_start_ns": 2,
      "t_end_ns": 4,
      "t_candidate_start_ns": 3,
      "t_confirmed_ns": 4
    }
  }'
```

## gRPC example (`grpcurl`)

```bash
grpcurl -plaintext \
  -import-path crates/selene_adapter/proto \
  -proto voice_ingress.proto \
  -d '{
    "correlation_id": 3001,
    "turn_id": 4001,
    "app_platform": "ANDROID",
    "trigger": "WAKE_WORD",
    "actor_user_id": "tenant_a:user_android",
    "tenant_id": "tenant_a",
    "device_id": "device_android_1",
    "now_ns": 5,
    "audio_capture_ref": {
      "stream_id_hi": 0,
      "stream_id_lo": 889,
      "pre_roll_buffer_id": 1,
      "t_start_ns": 1,
      "t_end_ns": 5,
      "t_candidate_start_ns": 2,
      "t_confirmed_ns": 4
    }
  }' \
  127.0.0.1:50051 \
  selene.adapter.v1.VoiceIngress/RunVoiceTurn
```

## Notes

- Accepted `app_platform`: `IOS`, `ANDROID`, `DESKTOP`.
- Accepted `trigger`: `EXPLICIT`, `WAKE_WORD`.
- For live PH1.C STT from capture evidence, provide `audio_capture_ref` (HTTP: `stream_id`; gRPC: `stream_id_hi` + `stream_id_lo`).
- The adapter replays/persists voice turns in `SELENE_ADAPTER_STORE_PATH` (default `.selene/adapter/voice_turns.jsonl`) and auto-seeds actor identity/device if missing.
- `GET /healthz` returns sync health counters for rollout gating (`acked`, `retry`, `dead-letter`, `replay-due`) plus improvement/build counters.
