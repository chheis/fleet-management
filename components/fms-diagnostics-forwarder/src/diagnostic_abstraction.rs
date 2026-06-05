// SPDX-FileCopyrightText: 2023 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

//! Abstracts diagnostic data from the Eclipse Kuksa Databroker into a [`DiagnosticStatus`].

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::time::Duration;

use clap::Args;
use http::Uri;
use kuksa_rust_sdk::kuksa::{common::ClientTraitV2, val::v2::KuksaClientV2};
use kuksa_rust_sdk::v2_proto::value::TypedValue;
use log::{debug, error, info, warn};
use protobuf::well_known_types::timestamp::Timestamp;
use protobuf::MessageField;
use tokio::sync::mpsc::Sender;

use fms_proto::fms::{DiagnosticCode, DiagnosticStatus, DiagnosticSummary};

// ---------------------------------------------------------------------------
// VSS path constants
// ---------------------------------------------------------------------------
pub const VSS_VEHICLE_VIN: &str = "Vehicle.VehicleIdentification.VIN";
pub const VSS_DIAG_ACTIVE_DTC_COUNT: &str = "Vehicle.Diagnostics.ActiveDTCCount";
pub const VSS_DIAG_STORED_DTC_COUNT: &str = "Vehicle.Diagnostics.StoredDTCCount";
pub const VSS_DIAG_PENDING_DTC_COUNT: &str = "Vehicle.Diagnostics.PendingDTCCount";
pub const VSS_DIAG_CRITICAL_DTC_COUNT: &str = "Vehicle.Diagnostics.CriticalDTCCount";
pub const VSS_DIAG_WORST_SEVERITY: &str = "Vehicle.Diagnostics.WorstSeverity";
pub const VSS_DIAG_LAST_CODE: &str = "Vehicle.Diagnostics.LastCode";
pub const VSS_DIAG_LAST_DESCRIPTION: &str = "Vehicle.Diagnostics.LastDescription";
pub const VSS_DIAG_LAST_STATUS_MASK: &str = "Vehicle.Diagnostics.LastStatusMask";
pub const VSS_DIAG_LAST_LIFECYCLE_STATE: &str = "Vehicle.Diagnostics.LastLifecycleState";
pub const VSS_DIAG_LAST_SEVERITY: &str = "Vehicle.Diagnostics.LastSeverity";
pub const VSS_DIAG_SOURCE: &str = "Vehicle.Diagnostics.Source";
pub const VSS_DIAG_COMPONENT_ID: &str = "Vehicle.Diagnostics.ComponentId";
pub const VSS_DIAG_ECU: &str = "Vehicle.Diagnostics.Ecu";
pub const VSS_DIAG_E2EV_CRC_OK: &str = "Vehicle.Diagnostics.E2EV.CrcOk";
pub const VSS_DIAG_E2EV_ALIVE_COUNTER: &str = "Vehicle.Diagnostics.E2EV.AliveCounter";

const DIAGNOSTIC_VSS_PATHS: &[&str] = &[
    VSS_VEHICLE_VIN,
    VSS_DIAG_ACTIVE_DTC_COUNT,
    VSS_DIAG_STORED_DTC_COUNT,
    VSS_DIAG_PENDING_DTC_COUNT,
    VSS_DIAG_CRITICAL_DTC_COUNT,
    VSS_DIAG_WORST_SEVERITY,
    VSS_DIAG_LAST_CODE,
    VSS_DIAG_LAST_DESCRIPTION,
    VSS_DIAG_LAST_STATUS_MASK,
    VSS_DIAG_LAST_LIFECYCLE_STATE,
    VSS_DIAG_LAST_SEVERITY,
    VSS_DIAG_SOURCE,
    VSS_DIAG_COMPONENT_ID,
    VSS_DIAG_ECU,
    VSS_DIAG_E2EV_CRC_OK,
    VSS_DIAG_E2EV_ALIVE_COUNTER,
];

const PARAM_DATABROKER_URI: &str = "databroker-uri";
const PARAM_TIMER_INTERVAL: &str = "timer-interval";

// ---------------------------------------------------------------------------
// CLI config
// ---------------------------------------------------------------------------

/// Configuration for connecting to the Eclipse Kuksa Databroker.
#[derive(Args, Clone, Debug)]
pub struct DiagnosticDatabrokerClientConfig {
    /// The HTTP(S) URI of the Eclipse Kuksa Databroker's gRPC endpoint.
    #[arg(
        long = PARAM_DATABROKER_URI,
        value_name = "URI",
        env = "KUKSA_DATABROKER_URI",
        default_value = "http://databroker:55556",
        value_parser = clap::builder::NonEmptyStringValueParser::new()
    )]
    pub databroker_uri: String,

    /// The time period to wait between polling the Databroker for diagnostic data.
    #[arg(
        long = PARAM_TIMER_INTERVAL,
        value_name = "DURATION_SPEC",
        env = "FMS_DIAGNOSTICS_FORWARDER_TIMER_INTERVAL",
        default_value = "2s",
        value_parser = |s: &str| duration_str::parse(s)
    )]
    pub timer_interval: Duration,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct DatabrokerError {
    description: String,
}

impl Error for DatabrokerError {}

impl Display for DatabrokerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error invoking Databroker: {}", self.description)
    }
}

// ---------------------------------------------------------------------------
// Helpers: convert TypedValue to Rust types
// ---------------------------------------------------------------------------

fn get_string(data: &HashMap<String, TypedValue>, key: &str) -> String {
    data.get(key)
        .and_then(|v| String::try_from(v).ok())
        .unwrap_or_default()
}

fn get_u32(data: &HashMap<String, TypedValue>, key: &str) -> u32 {
    data.get(key)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Build DiagnosticStatus from VSS data map
// ---------------------------------------------------------------------------

pub fn build_diagnostic_status(data: HashMap<String, TypedValue>) -> DiagnosticStatus {
    let vin = data
        .get(VSS_VEHICLE_VIN)
        .and_then(|v| String::try_from(v).ok())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "UNKNOWN-VIN".to_string());

    let source = {
        let s = get_string(&data, VSS_DIAG_SOURCE);
        if s.is_empty() {
            "diagnostic-source-simulator".to_string()
        } else {
            s
        }
    };

    let component_id = {
        let s = get_string(&data, VSS_DIAG_COMPONENT_ID);
        if s.is_empty() {
            "unknown-component".to_string()
        } else {
            s
        }
    };

    let active_count = get_u32(&data, VSS_DIAG_ACTIVE_DTC_COUNT);
    let stored_count = get_u32(&data, VSS_DIAG_STORED_DTC_COUNT);
    let pending_count = get_u32(&data, VSS_DIAG_PENDING_DTC_COUNT);
    let critical_count = get_u32(&data, VSS_DIAG_CRITICAL_DTC_COUNT);
    let worst_severity = {
        let s = get_string(&data, VSS_DIAG_WORST_SEVERITY);
        if s.is_empty() {
            "UNKNOWN".to_string()
        } else {
            s
        }
    };

    let created_ts = Timestamp::now();

    let mut summary = DiagnosticSummary::new();
    summary.active_count = active_count;
    summary.stored_count = stored_count;
    summary.pending_count = pending_count;
    summary.critical_count = critical_count;
    summary.has_active_faults = active_count > 0;
    summary.worst_severity = worst_severity;

    let mut status = DiagnosticStatus::new();
    status.vin = vin;
    status.source = source.clone();
    status.component_id = component_id.clone();
    status.created = MessageField::some(created_ts.clone());
    status.summary = MessageField::some(summary);

    // Build a single DiagnosticCode from the "last code" fields if present
    let last_code = get_string(&data, VSS_DIAG_LAST_CODE);
    if !last_code.is_empty() {
        let lifecycle_state = {
            let s = get_string(&data, VSS_DIAG_LAST_LIFECYCLE_STATE);
            if s.is_empty() {
                "UNKNOWN".to_string()
            } else {
                s
            }
        };
        let severity = {
            let s = get_string(&data, VSS_DIAG_LAST_SEVERITY);
            if s.is_empty() {
                "UNKNOWN".to_string()
            } else {
                s
            }
        };
        let ecu = {
            let s = get_string(&data, VSS_DIAG_ECU);
            if s.is_empty() {
                "UNKNOWN".to_string()
            } else {
                s
            }
        };
        let protocol = {
            let src = source.to_lowercase();
            if src.contains("opensovd") || src.contains("openbsw") {
                "UDS".to_string()
            } else {
                "INTERNAL".to_string()
            }
        };

        let mut code = DiagnosticCode::new();
        code.code = last_code.clone();
        code.raw_uds_dtc = last_code;
        code.protocol = protocol;
        code.status_mask = get_string(&data, VSS_DIAG_LAST_STATUS_MASK);
        code.description = get_string(&data, VSS_DIAG_LAST_DESCRIPTION);
        code.severity = severity;
        code.lifecycle_state = lifecycle_state.clone();
        code.ecu = ecu;
        code.component_id = component_id;
        code.source = source;
        code.first_seen = MessageField::some(created_ts.clone());
        code.last_seen = MessageField::some(created_ts);

        match lifecycle_state.as_str() {
            "ACTIVE" => status.active_codes.push(code),
            "STORED" => status.stored_codes.push(code),
            "PENDING" => status.pending_codes.push(code),
            _ => status.active_codes.push(code),
        }
    }

    status
}

// ---------------------------------------------------------------------------
// KuksaValDatabroker wrapper
// ---------------------------------------------------------------------------

struct KuksaDiagnosticDatabroker {
    client: Box<KuksaClientV2>,
}

impl KuksaDiagnosticDatabroker {
    async fn new(config: &DiagnosticDatabrokerClientConfig) -> Result<Self, DatabrokerError> {
        info!(
            "creating diagnostic client for Eclipse Kuksa Databroker at {}",
            config.databroker_uri
        );
        Uri::try_from(config.databroker_uri.clone())
            .map_err(|err| {
                error!("invalid Databroker URI: {err}");
                DatabrokerError {
                    description: err.to_string(),
                }
            })
            .map(|uri| {
                let client = KuksaClientV2::new(uri);
                KuksaDiagnosticDatabroker {
                    client: Box::new(client),
                }
            })
    }

    pub async fn get_diagnostic_status(&mut self) -> Result<DiagnosticStatus, DatabrokerError> {
        let paths = DIAGNOSTIC_VSS_PATHS.iter().map(|v| v.to_string()).collect();

        match self.client.get_values(paths).await {
            Err(kuksa_rust_sdk::kuksa::common::ClientError::Connection(msg)) => {
                warn!("failed to retrieve diagnostic data points from Databroker: {msg}");
                Err(DatabrokerError { description: msg })
            }
            Err(kuksa_rust_sdk::kuksa::common::ClientError::Status(status)) => {
                warn!(
                    "failed to retrieve diagnostic data points from Databroker: {}",
                    status.message()
                );
                Err(DatabrokerError {
                    description: status.message().to_string(),
                })
            }
            Err(kuksa_rust_sdk::kuksa::common::ClientError::Function(errors)) => {
                errors.iter().for_each(|error| {
                    warn!("failed to retrieve diagnostic data points from Databroker: {error:?}");
                });
                Err(DatabrokerError {
                    description: "multiple errors while retrieving diagnostic data".to_string(),
                })
            }
            Ok(get_response) => {
                let mut vss_data = HashMap::new();
                let mut idx = 0usize;
                get_response.iter().for_each(|data_entry| {
                    if let (name, Some(value)) = (
                        DIAGNOSTIC_VSS_PATHS[idx],
                        data_entry
                            .value
                            .as_ref()
                            .and_then(|v| v.typed_value.as_ref()),
                    ) {
                        debug!("got value [path: {name}]: {value:?}");
                        vss_data.insert(name.to_owned(), value.to_owned());
                    }
                    idx += 1;
                });
                Ok(build_diagnostic_status(vss_data))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public init function
// ---------------------------------------------------------------------------

/// Sets up a connection to the Databroker and starts a timer-based polling loop,
/// sending `DiagnosticStatus` messages through `status_publisher`.
pub async fn init(
    config: &DiagnosticDatabrokerClientConfig,
    status_publisher: Sender<DiagnosticStatus>,
) -> Result<(), DatabrokerError> {
    let timer_interval = config.timer_interval;

    let mut databroker = KuksaDiagnosticDatabroker::new(config).await?;

    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(timer_interval);
        loop {
            interval.tick().await;
            match databroker.get_diagnostic_status().await {
                Err(e) => {
                    warn!(
                        "failed to retrieve current diagnostic status from databroker: {}",
                        e
                    );
                }
                Ok(status) => {
                    if let Err(e) = status_publisher.send(status).await {
                        warn!("failed to send diagnostic status: {}", e);
                    }
                }
            }
        }
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use kuksa_rust_sdk::v2_proto::value::TypedValue;

    fn make_string_value(s: &str) -> TypedValue {
        TypedValue::String(s.to_string())
    }

    fn make_uint32_value(v: u32) -> TypedValue {
        TypedValue::Uint32(v)
    }

    #[test]
    fn test_summary_counts_set_correctly() {
        let mut data = HashMap::new();
        data.insert(VSS_VEHICLE_VIN.to_string(), make_string_value("TEST-VIN"));
        data.insert(VSS_DIAG_ACTIVE_DTC_COUNT.to_string(), make_uint32_value(2));
        data.insert(VSS_DIAG_STORED_DTC_COUNT.to_string(), make_uint32_value(3));
        data.insert(VSS_DIAG_PENDING_DTC_COUNT.to_string(), make_uint32_value(1));
        data.insert(
            VSS_DIAG_CRITICAL_DTC_COUNT.to_string(),
            make_uint32_value(1),
        );
        data.insert(
            VSS_DIAG_WORST_SEVERITY.to_string(),
            make_string_value("CRITICAL"),
        );

        let status = build_diagnostic_status(data);
        let summary = status.summary.as_ref().unwrap();
        assert_eq!(summary.active_count, 2);
        assert_eq!(summary.stored_count, 3);
        assert_eq!(summary.pending_count, 1);
        assert_eq!(summary.critical_count, 1);
        assert!(summary.has_active_faults);
        assert_eq!(summary.worst_severity, "CRITICAL");
    }

    #[test]
    fn test_active_code_routing() {
        let mut data = HashMap::new();
        data.insert(
            VSS_DIAG_LAST_CODE.to_string(),
            make_string_value("0xABCDEF"),
        );
        data.insert(
            VSS_DIAG_LAST_LIFECYCLE_STATE.to_string(),
            make_string_value("ACTIVE"),
        );

        let status = build_diagnostic_status(data);
        assert_eq!(status.active_codes.len(), 1);
        assert_eq!(status.stored_codes.len(), 0);
        assert_eq!(status.pending_codes.len(), 0);
        assert_eq!(status.active_codes[0].code, "0xABCDEF");
    }

    #[test]
    fn test_stored_code_routing() {
        let mut data = HashMap::new();
        data.insert(
            VSS_DIAG_LAST_CODE.to_string(),
            make_string_value("0x111111"),
        );
        data.insert(
            VSS_DIAG_LAST_LIFECYCLE_STATE.to_string(),
            make_string_value("STORED"),
        );

        let status = build_diagnostic_status(data);
        assert_eq!(status.stored_codes.len(), 1);
        assert_eq!(status.active_codes.len(), 0);
    }

    #[test]
    fn test_missing_values_produce_safe_defaults() {
        let data = HashMap::new();
        let status = build_diagnostic_status(data);
        assert_eq!(status.vin, "UNKNOWN-VIN");
        assert_eq!(status.source, "diagnostic-source-simulator");
        assert_eq!(status.component_id, "unknown-component");
        let summary = status.summary.as_ref().unwrap();
        assert_eq!(summary.active_count, 0);
        assert!(!summary.has_active_faults);
        assert_eq!(summary.worst_severity, "UNKNOWN");
        assert!(status.active_codes.is_empty());
    }
}
