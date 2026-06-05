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

use std::time::Duration;

use clap::Parser;
use http::Uri;
use kuksa_rust_sdk::kuksa::{common::ClientTraitV2, val::v2::KuksaClientV2};
use kuksa_rust_sdk::v2_proto::value::TypedValue;
use kuksa_rust_sdk::v2_proto::Value;
use log::{info, warn};

const VSS_DIAG_ACTIVE_DTC_COUNT: &str = "Vehicle.Diagnostics.ActiveDTCCount";
const VSS_DIAG_STORED_DTC_COUNT: &str = "Vehicle.Diagnostics.StoredDTCCount";
const VSS_DIAG_PENDING_DTC_COUNT: &str = "Vehicle.Diagnostics.PendingDTCCount";
const VSS_DIAG_CRITICAL_DTC_COUNT: &str = "Vehicle.Diagnostics.CriticalDTCCount";
const VSS_DIAG_WORST_SEVERITY: &str = "Vehicle.Diagnostics.WorstSeverity";
const VSS_DIAG_LAST_CODE: &str = "Vehicle.Diagnostics.LastCode";
const VSS_DIAG_LAST_DESCRIPTION: &str = "Vehicle.Diagnostics.LastDescription";
const VSS_DIAG_LAST_STATUS_MASK: &str = "Vehicle.Diagnostics.LastStatusMask";
const VSS_DIAG_LAST_LIFECYCLE_STATE: &str = "Vehicle.Diagnostics.LastLifecycleState";
const VSS_DIAG_LAST_SEVERITY: &str = "Vehicle.Diagnostics.LastSeverity";
const VSS_DIAG_SOURCE: &str = "Vehicle.Diagnostics.Source";
const VSS_DIAG_COMPONENT_ID: &str = "Vehicle.Diagnostics.ComponentId";
const VSS_DIAG_ECU: &str = "Vehicle.Diagnostics.Ecu";
const VSS_DIAG_E2EV_CRC_OK: &str = "Vehicle.Diagnostics.E2EV.CrcOk";
const VSS_DIAG_E2EV_ALIVE_COUNTER: &str = "Vehicle.Diagnostics.E2EV.AliveCounter";

/// Simulates diagnostic data by writing VSS signals to the Kuksa Databroker.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct SimulatorCommand {
    /// The HTTP(S) URI of the Eclipse Kuksa Databroker's gRPC endpoint.
    #[arg(
        long = "databroker-uri",
        value_name = "URI",
        env = "KUKSA_DATABROKER_URI",
        default_value = "http://databroker:55556"
    )]
    databroker_uri: String,

    /// Interval between state transitions (seconds).
    #[arg(
        long = "interval-secs",
        env = "SIMULATOR_INTERVAL_SECS",
        default_value = "10"
    )]
    interval_secs: u64,
}

fn make_value(typed: TypedValue) -> Value {
    Value {
        typed_value: Some(typed),
    }
}

async fn publish(client: &mut KuksaClientV2, path: &str, value: TypedValue) {
    if let Err(e) = client
        .publish_value(path.to_string(), make_value(value))
        .await
    {
        warn!("failed to write [{path}] to Databroker: {e:?}");
    }
}

async fn write_state_cleared(client: &mut KuksaClientV2, alive_counter: u32) {
    info!("Transitioning to State A (cleared)");
    publish(client, VSS_DIAG_ACTIVE_DTC_COUNT, TypedValue::Uint32(0)).await;
    publish(client, VSS_DIAG_STORED_DTC_COUNT, TypedValue::Uint32(0)).await;
    publish(client, VSS_DIAG_PENDING_DTC_COUNT, TypedValue::Uint32(0)).await;
    publish(client, VSS_DIAG_CRITICAL_DTC_COUNT, TypedValue::Uint32(0)).await;
    publish(
        client,
        VSS_DIAG_WORST_SEVERITY,
        TypedValue::String("INFO".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_LAST_LIFECYCLE_STATE,
        TypedValue::String("CLEARED".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_LAST_CODE,
        TypedValue::String(String::new()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_LAST_SEVERITY,
        TypedValue::String("INFO".to_string()),
    )
    .await;
    publish(client, VSS_DIAG_E2EV_CRC_OK, TypedValue::Bool(true)).await;
    publish(
        client,
        VSS_DIAG_E2EV_ALIVE_COUNTER,
        TypedValue::Uint32(alive_counter),
    )
    .await;
}

async fn write_state_faulted(client: &mut KuksaClientV2, alive_counter: u32) {
    info!("Transitioning to State B (faulted)");
    publish(client, VSS_DIAG_ACTIVE_DTC_COUNT, TypedValue::Uint32(1)).await;
    publish(client, VSS_DIAG_STORED_DTC_COUNT, TypedValue::Uint32(1)).await;
    publish(client, VSS_DIAG_PENDING_DTC_COUNT, TypedValue::Uint32(0)).await;
    publish(client, VSS_DIAG_CRITICAL_DTC_COUNT, TypedValue::Uint32(1)).await;
    publish(
        client,
        VSS_DIAG_WORST_SEVERITY,
        TypedValue::String("CRITICAL".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_LAST_LIFECYCLE_STATE,
        TypedValue::String("ACTIVE".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_LAST_CODE,
        TypedValue::String("0x123456".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_LAST_DESCRIPTION,
        TypedValue::String("E2EV signal validation failed".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_LAST_STATUS_MASK,
        TypedValue::String("0x2F".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_LAST_SEVERITY,
        TypedValue::String("CRITICAL".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_SOURCE,
        TypedValue::String("diagnostic-source-simulator".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_COMPONENT_ID,
        TypedValue::String("threadx-e2ev".to_string()),
    )
    .await;
    publish(
        client,
        VSS_DIAG_ECU,
        TypedValue::String("threadx-e2ev".to_string()),
    )
    .await;
    publish(client, VSS_DIAG_E2EV_CRC_OK, TypedValue::Bool(false)).await;
    publish(
        client,
        VSS_DIAG_E2EV_ALIVE_COUNTER,
        TypedValue::Uint32(alive_counter),
    )
    .await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let command = SimulatorCommand::parse();

    info!(
        "connecting to Kuksa Databroker at {}",
        command.databroker_uri
    );
    let uri = Uri::try_from(command.databroker_uri.clone())?;
    let mut client = KuksaClientV2::new(uri);

    let interval = Duration::from_secs(command.interval_secs);
    let mut ticker = tokio::time::interval(interval);
    let mut state_faulted = false;
    let mut alive_counter: u32 = 0;

    info!(
        "starting diagnostic source simulator (interval: {:?})",
        interval
    );

    loop {
        ticker.tick().await;
        alive_counter = alive_counter.wrapping_add(1);

        if state_faulted {
            write_state_cleared(&mut client, alive_counter).await;
        } else {
            write_state_faulted(&mut client, alive_counter).await;
        }
        state_faulted = !state_faulted;
    }
}
