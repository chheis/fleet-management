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

use std::str::FromStr;
use std::sync::Arc;
use std::thread;

use clap::{Parser, Subcommand};
use fms_proto::fms::DiagnosticStatus;
use fms_zenoh::ZenohTransportConfig;
use influx_client::connection::InfluxConnectionConfig;
use influx_client::writer::InfluxWriter;
use log::info;

use up_rust::{UListener, UMessage, UTransport, UUri};
use up_transport_hono_kafka::{HonoKafkaTransport, HonoKafkaTransportConfig};
use up_transport_zenoh::UPTransportZenoh;

struct DiagnosticStatusListener {
    influx_writer: InfluxWriter,
}

#[async_trait::async_trait]
impl UListener for DiagnosticStatusListener {
    async fn on_receive(&self, msg: UMessage) {
        if let Ok(diagnostic_status) = msg.extract_protobuf::<DiagnosticStatus>() {
            self.influx_writer
                .write_diagnostic_status(&diagnostic_status)
                .await;
        } else {
            info!("ignoring event with invalid/unknown payload");
        }
    }
}

/// Receives FMS diagnostics data via Zenoh or Hono uProtocol transport
/// and writes them to an InfluxDB server.
#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct FmsDiagnosticsConsumerCommand {
    /// The topic URI pattern to use for consuming diagnostic status events.
    #[arg(long = "topic-filter", value_name = "URI", env = "DIAGNOSTIC_TOPIC_FILTER", default_value = "up://*/D110/1/D110", value_parser = up_rust::UUri::from_str)]
    diagnostic_topic_filter: UUri,

    /// The local uService address.
    #[arg(long = "uservice-uri", value_name = "URI", env = "USERVICE_URI", default_value = "up://fms-diagnostics-consumer/D111/1/0", value_parser = up_rust::UUri::from_str)]
    local_uservice_uri: UUri,

    #[command(flatten)]
    influxdb_connection: InfluxConnectionConfig,

    #[command(subcommand)]
    transport: TransportType,
}

#[derive(Subcommand)]
#[command(subcommand_required = true)]
enum TransportType {
    /// Consumes diagnostic data using the Eclipse Hono/Kafka based uProtocol transport.
    #[command(name = "hono")]
    Hono(HonoKafkaTransportConfig),

    /// Consumes diagnostic data using the Eclipse Zenoh based uProtocol transport.
    #[command(name = "zenoh")]
    Zenoh(ZenohTransportConfig),
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let command = FmsDiagnosticsConsumerCommand::parse();

    let transport: Arc<dyn UTransport> = match command.transport {
        TransportType::Hono(config) => HonoKafkaTransport::new(config).map(Arc::new)?,
        TransportType::Zenoh(config) => {
            let config = config.try_into()?;
            UPTransportZenoh::new(config, command.local_uservice_uri)
                .await
                .map(Arc::new)?
        }
    };

    let influx_writer = InfluxWriter::new(&command.influxdb_connection)?;
    let listener = Arc::new(DiagnosticStatusListener { influx_writer });
    info!(
        "Registering listener for diagnostic status events [source filter: {}]",
        &command.diagnostic_topic_filter.to_uri(false)
    );
    transport
        .register_listener(&command.diagnostic_topic_filter, None, listener)
        .await
        .map_err(Box::new)?;
    // do not let the listener go out of scope
    thread::park();

    Ok(())
}
