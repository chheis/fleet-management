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

/// Response model for a single diagnostic summary entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticSummaryObject {
    #[serde(rename = "vin")]
    pub vin: String,

    #[serde(rename = "source")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(rename = "componentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,

    #[serde(rename = "createdDateTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date_time: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "activeCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_count: Option<i64>,

    #[serde(rename = "storedCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stored_count: Option<i64>,

    #[serde(rename = "pendingCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_count: Option<i64>,

    #[serde(rename = "criticalCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_count: Option<i64>,

    #[serde(rename = "hasActiveFaults")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_active_faults: Option<bool>,

    #[serde(rename = "worstSeverity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worst_severity: Option<String>,
}

/// Response model for a single diagnostic code (DTC) entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticCodeObject {
    #[serde(rename = "vin")]
    pub vin: String,

    #[serde(rename = "code")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "source")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(rename = "componentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,

    #[serde(rename = "ecu")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecu: Option<String>,

    #[serde(rename = "severity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,

    #[serde(rename = "lifecycleState")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,

    #[serde(rename = "protocol")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,

    #[serde(rename = "rawUdsDtc")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_uds_dtc: Option<String>,

    #[serde(rename = "statusMask")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_mask: Option<String>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "createdDateTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_date_time: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "firstSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "lastSeen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

/// Response wrapper listing VINs that have diagnostic data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticVehicleListResponse {
    #[serde(rename = "vins")]
    pub vins: Vec<String>,
}

/// Response wrapper for a list of diagnostic summaries.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticSummaryListResponse {
    #[serde(rename = "summaries")]
    pub summaries: Vec<DiagnosticSummaryObject>,
}

/// Response wrapper for a list of diagnostic codes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticCodeListResponse {
    #[serde(rename = "dtcs")]
    pub dtcs: Vec<DiagnosticCodeObject>,
}
