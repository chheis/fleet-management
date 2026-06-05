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

//! A client for accessing an InfluxDB server.
//!
//! Supports connecting to an InfluxDB server based on parameters
//! read from the command line or from environment variables.

pub const FIELD_ALTITUDE: &str = "altitude";
pub const FIELD_AMBIENT_AIR_TEMP: &str = "ambientAirTemperature";
pub const FIELD_CATALYST_FUEL_LEVEL: &str = "catalystFuelLevel";
pub const FIELD_CREATED_DATE_TIME: &str = "createdDateTime";
pub const FIELD_DRIVER1_ID: &str = "driver1Id";
pub const FIELD_DRIVER1_CARD_ISSUER: &str = "driver1IdCardIssuer";
pub const FIELD_DRIVER1_WORKING_STATE: &str = "driver1WorkingState";
pub const FIELD_DRIVER2_ID: &str = "driver2Id";
pub const FIELD_DRIVER2_CARD_ISSUER: &str = "driver2IdCardIssuer";
pub const FIELD_DRIVER2_WORKING_STATE: &str = "driver2WorkingState";
pub const FIELD_DIRECTION_INDICATOR_LEFT: &str = "directionIndicatorLeft";
pub const FIELD_DIRECTION_INDICATOR_RIGHT: &str = "directionIndicatorRight";
pub const FIELD_BRAKE_LIGHT_STATUS: &str = "brakeLightStatus";
pub const FIELD_ENGINE_SPEED: &str = "engineSpeed";
pub const FIELD_ENGINE_TOTAL_FUEL_USED: &str = "engineTotalFuelUsed";
pub const FIELD_ESTIMATED_DIST_TO_EMPTY_FUEL: &str = "estimatedDistanceToEmptyFuel";
pub const FIELD_ESTIMATED_DIST_TO_EMPTY_TOTAL: &str = "estimatedDistanceToEmptyTotal";
pub const FIELD_FUEL_LEVEL1: &str = "fuelLevel1";
pub const FIELD_FUEL_LEVEL2: &str = "fuelLevel2";
pub const FIELD_FUEL_TYPE: &str = "fuelType";
pub const FIELD_GROSS_COMBINATION_VEHICLE_WEIGHT: &str = "grossCombinationVehicleWeight";
pub const FIELD_HEADING: &str = "heading";
pub const FIELD_HR_TOTAL_VEHICLE_DISTANCE: &str = "hrTotalVehicleDistance";
pub const FIELD_LATITUDE: &str = "latitude";
pub const FIELD_LONGITUDE: &str = "longitude";
pub const FIELD_PARKING_BREAK_SWITCH: &str = "parkingBrakeSwitch";
pub const FIELD_POSITION_DATE_TIME: &str = "positionDateTime";
pub const FIELD_SPEED: &str = "speed";
pub const FIELD_TACHOGRAPH_SPEED: &str = "tachographSpeed";
pub const FIELD_TOTAL_ELECTRIC_MOTOR_HOURS: &str = "totalElectricMotorHours";
pub const FIELD_TOTAL_ENGINE_HOURS: &str = "totalEngineHours";
pub const FIELD_WHEEL_BASED_SPEED: &str = "wheelBasedSpeed";

pub const MEASUREMENT_HEADER: &str = "header";
pub const MEASUREMENT_SNAPSHOT: &str = "snapshot";
pub const MEASUREMENT_DIAGNOSTIC_SUMMARY: &str = "diagnostic_summary";
pub const MEASUREMENT_DIAGNOSTIC_CODE: &str = "diagnostic_code";

pub const TAG_TRIGGER: &str = "trigger";
pub const TAG_VIN: &str = "vin";
pub const TAG_SOURCE: &str = "source";
pub const TAG_COMPONENT_ID: &str = "componentId";
pub const TAG_ECU: &str = "ecu";
pub const TAG_CODE: &str = "code";
pub const TAG_SEVERITY: &str = "severity";
pub const TAG_LIFECYCLE_STATE: &str = "lifecycleState";
pub const TAG_PROTOCOL: &str = "protocol";

pub const FIELD_ACTIVE_COUNT: &str = "activeCount";
pub const FIELD_STORED_COUNT: &str = "storedCount";
pub const FIELD_PENDING_COUNT: &str = "pendingCount";
pub const FIELD_CRITICAL_COUNT: &str = "criticalCount";
pub const FIELD_HAS_ACTIVE_FAULTS: &str = "hasActiveFaults";
pub const FIELD_WORST_SEVERITY: &str = "worstSeverity";
pub const FIELD_RAW_UDS_DTC: &str = "rawUdsDtc";
pub const FIELD_STATUS_MASK: &str = "statusMask";
pub const FIELD_DESCRIPTION: &str = "description";
pub const FIELD_FIRST_SEEN: &str = "firstSeen";
pub const FIELD_LAST_SEEN: &str = "lastSeen";

pub mod connection;
#[cfg(feature = "writer")]
pub mod writer;
