<!--
SPDX-FileCopyrightText: 2023 Contributors to the Eclipse Foundation

See the NOTICE file(s) distributed with this work for additional
information regarding copyright ownership.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

SPDX-License-Identifier: Apache-2.0
-->

# Diagnostics Code Forwarding

This document describes the **parallel diagnostics pipeline** added alongside the existing rFMS/VehicleStatus flow.

## Baseline Status

The existing workspace build (`cargo build --workspace`) was verified to complete successfully in this environment. The `fms-proto` crate was previously not included in the workspace `members` list; it has been added as part of this feature.

Cross-compilation via `rust-musl-cross` Docker images is not available in the sandbox environment and is noted as a limitation for Docker image builds.

## Architecture

```
diagnostic-source-simulator
         │ (writes VSS paths via Kuksa gRPC)
         ▼
 Kuksa Databroker
 (Vehicle.Diagnostics.* VSS paths)
         │ (gRPC get_values poll)
         ▼
fms-diagnostics-forwarder
         │ (uProtocol Notification, topic up://fms-diagnostics-forwarder/D110/1/D110)
         ▼
   Zenoh Router
         │
         ▼
fms-diagnostics-consumer
         │ (DiagnosticStatus protobuf decode)
         ▼
      InfluxDB 2.7
  (diagnostic_summary, diagnostic_code measurements)
         │
         ├──► Grafana FMS-Diagnostics dashboard
         └──► fms-server /diagnostics/* REST API
```

The existing path is **unchanged**:
```
csv-provider → Databroker → fms-forwarder → uProtocol → fms-consumer → InfluxDB → Grafana/rFMS API
```

## uProtocol Topic

| Parameter | Value |
|-----------|-------|
| Topic (forwarder) | `up://fms-diagnostics-forwarder/D110/1/D110` |
| Topic filter (consumer) | `up://*/D110/1/D110` |
| Local uService (consumer) | `up://fms-diagnostics-consumer/D111/1/0` |

`D110` (hex `0x0110` = 272) is the resource ID for diagnostics notifications.

## Protobuf Messages (`fms.proto`)

Three new messages are added to `proto/fms/v4/fms.proto` (package `fms.v4`):

### `DiagnosticSummary`
| Field | Number | Type | Description |
|-------|--------|------|-------------|
| active_count | 1 | uint32 | Number of active DTCs |
| stored_count | 2 | uint32 | Number of stored DTCs |
| pending_count | 3 | uint32 | Number of pending DTCs |
| critical_count | 4 | uint32 | Number of critical DTCs |
| has_active_faults | 5 | bool | True when active_count > 0 |
| worst_severity | 6 | string | INFO \| WARNING \| CRITICAL \| UNKNOWN |

### `DiagnosticCode`
| Field | Number | Type | Description |
|-------|--------|------|-------------|
| code | 1 | string | DTC code e.g. "0x123456" |
| raw_uds_dtc | 2 | string | Raw UDS DTC bytes |
| protocol | 3 | string | UDS \| SOVD \| OBD \| INTERNAL |
| status_mask | 4 | string | UDS status mask hex |
| description | 5 | string | Human-readable description |
| severity | 6 | string | INFO \| WARNING \| CRITICAL \| UNKNOWN |
| lifecycle_state | 7 | string | ACTIVE \| STORED \| PENDING \| CLEARED \| UNKNOWN |
| ecu | 8 | string | ECU identifier |
| component_id | 9 | string | Component identifier |
| source | 10 | string | Diagnostic source |
| first_seen | 11 | Timestamp | First observation time |
| last_seen | 12 | Timestamp | Last observation time |

### `DiagnosticStatus`
| Field | Number | Type | Description |
|-------|--------|------|-------------|
| vin | 1 | string | Vehicle identification number |
| vehicle_id | 2 | string | Optional vehicle ID |
| source | 3 | string | Diagnostic source system |
| component_id | 4 | string | Component identifier |
| created | 5 | Timestamp | Message creation time |
| active_codes | 6 | repeated DiagnosticCode | Active DTCs |
| stored_codes | 7 | repeated DiagnosticCode | Stored DTCs |
| pending_codes | 8 | repeated DiagnosticCode | Pending DTCs |
| summary | 9 | DiagnosticSummary | Summary counts and flags |

## Vehicle.Diagnostics VSS Paths

The following paths are added to `spec/overlay/vss.json` under the `Vehicle.Diagnostics` branch:

| Path | Type | Datatype | Description |
|------|------|----------|-------------|
| Vehicle.Diagnostics | branch | — | Diagnostic data |
| Vehicle.Diagnostics.ActiveDTCCount | sensor | uint16 | Active DTC count |
| Vehicle.Diagnostics.StoredDTCCount | sensor | uint16 | Stored DTC count |
| Vehicle.Diagnostics.PendingDTCCount | sensor | uint16 | Pending DTC count |
| Vehicle.Diagnostics.CriticalDTCCount | sensor | uint16 | Critical DTC count |
| Vehicle.Diagnostics.WorstSeverity | sensor | string | INFO/WARNING/CRITICAL/UNKNOWN |
| Vehicle.Diagnostics.LastUpdate | sensor | string | ISO timestamp of last update |
| Vehicle.Diagnostics.Source | sensor | string | Source system name |
| Vehicle.Diagnostics.ComponentId | sensor | string | Component identifier |
| Vehicle.Diagnostics.Ecu | sensor | string | ECU identifier |
| Vehicle.Diagnostics.LastCode | sensor | string | Most recent DTC code |
| Vehicle.Diagnostics.LastDescription | sensor | string | Description of last DTC |
| Vehicle.Diagnostics.LastStatusMask | sensor | string | Status mask of last DTC |
| Vehicle.Diagnostics.LastLifecycleState | sensor | string | ACTIVE/STORED/PENDING/CLEARED/UNKNOWN |
| Vehicle.Diagnostics.LastSeverity | sensor | string | INFO/WARNING/CRITICAL/UNKNOWN |
| Vehicle.Diagnostics.E2EV | branch | — | End-to-End Validation signals |
| Vehicle.Diagnostics.E2EV.CrcOk | sensor | boolean | CRC check result |
| Vehicle.Diagnostics.E2EV.AliveCounter | sensor | uint16 | Alive counter |
| Vehicle.Diagnostics.E2EV.LastFault | sensor | string | Last E2EV fault description |

> **Note**: `spec/overlay/vss.json` was hand-edited because COVESA `vspec2json.py` regeneration is out of scope for this feature.

## InfluxDB Measurements

### `diagnostic_summary`

| Type | Name | Description |
|------|------|-------------|
| tag | vin | Vehicle identification number |
| tag | source | Diagnostic source |
| tag | componentId | Component identifier |
| field | createdDateTime | Milliseconds since UNIX epoch |
| field | activeCount | Active DTC count |
| field | storedCount | Stored DTC count |
| field | pendingCount | Pending DTC count |
| field | criticalCount | Critical DTC count |
| field | hasActiveFaults | Boolean flag |
| field | worstSeverity | Worst severity string |

### `diagnostic_code`

One measurement per DTC across active/stored/pending codes.

| Type | Name | Description |
|------|------|-------------|
| tag | vin | Vehicle identification number |
| tag | source | Diagnostic source |
| tag | componentId | Component identifier |
| tag | ecu | ECU identifier |
| tag | code | DTC code |
| tag | severity | Severity level |
| tag | lifecycleState | ACTIVE/STORED/PENDING/CLEARED/UNKNOWN |
| tag | protocol | UDS/SOVD/OBD/INTERNAL |
| field | createdDateTime | Milliseconds since UNIX epoch |
| field | rawUdsDtc | Raw UDS DTC bytes |
| field | statusMask | UDS status mask |
| field | description | Human-readable description |
| field | firstSeen | First seen timestamp (ms) |
| field | lastSeen | Last seen timestamp (ms) |

## Compose Services

Three new services are added to `fms-blueprint-compose.yaml`:

| Service | Network(s) | Depends On |
|---------|-----------|------------|
| `diagnostic-source-simulator` | `fms-vehicle` | `databroker` |
| `fms-diagnostics-forwarder` | `fms-backend`, `fms-vehicle` | `databroker`, `diagnostic-source-simulator` |
| `fms-diagnostics-consumer` | `fms-backend` | `influxdb` (healthy) |

Both `fms-diagnostics-forwarder` and `fms-diagnostics-consumer` get Zenoh overrides in `fms-blueprint-compose-zenoh.yaml`.

## REST API Endpoints (`fms-server`)

New routes under `/diagnostics/`:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/diagnostics/vehicles` | List all VINs with diagnostic data |
| GET | `/diagnostics/vehicles/{vin}/summary` | Latest diagnostic summary for a VIN |
| GET | `/diagnostics/vehicles/{vin}/dtcs` | All DTCs (active + stored + pending) |
| GET | `/diagnostics/vehicles/{vin}/dtcs/active` | Active DTCs only |
| GET | `/diagnostics/vehicles/{vin}/timeline` | Recent summary points over time |

### Example `curl` commands

```bash
# List vehicles with diagnostic data
curl http://localhost:8081/diagnostics/vehicles

# Get summary for a specific VIN
curl http://localhost:8081/diagnostics/vehicles/DEMO-VIN-001/summary

# Get all DTCs for a VIN
curl http://localhost:8081/diagnostics/vehicles/DEMO-VIN-001/dtcs

# Get active DTCs only
curl http://localhost:8081/diagnostics/vehicles/DEMO-VIN-001/dtcs/active

# Get timeline of summaries
curl http://localhost:8081/diagnostics/vehicles/DEMO-VIN-001/timeline
```

## Grafana Dashboard

The `FMS-Diagnostics.json` dashboard is provisioned alongside the existing `FMS-Fleet.json` via the `dashboards_from_filesystem.yaml` provisioner. Panels include:

- Active DTC Count by VIN (time series)
- Worst Severity by VIN (stat panel)
- Active DTC table (table panel)
- E2EV CRC status (stat panel)
- DTC timeline (time series)

## Known Limitations

1. **Single last-code mapping (v1)**: The forwarder maps only one DTC from the databroker (the "last code") per poll cycle. Multi-DTC representation requires a richer VSS or a dedicated adapter.
2. **Zenoh-only transport**: Diagnostics uses Zenoh transport. Hono transport parity is a follow-up task (see `TODO` in `fms-blueprint-compose-hono.yaml`).
3. **OpenSOVD/OpenBSW adapter**: An adapter that translates OpenSOVD/OpenBSW diagnostic data into the Kuksa VSS paths is a follow-up item.
4. **VSS hand-edited**: `spec/overlay/vss.json` was hand-edited because COVESA `vspec2json.py` regeneration is out of scope for this feature.
5. **Docker image builds**: Cross-compilation via `rust-musl-cross` images was not tested in the sandbox environment. The Dockerfiles follow the same pattern as existing forwarder/consumer images.

## TODO

- Add Hono transport support to `fms-diagnostics-forwarder` and `fms-diagnostics-consumer`.
- Implement OpenSOVD/OpenBSW adapter to publish real diagnostic data to the Kuksa databroker.
- Support multiple simultaneous DTCs via an extended VSS representation.
