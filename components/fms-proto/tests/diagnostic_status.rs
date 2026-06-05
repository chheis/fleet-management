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

use fms_proto::fms::{DiagnosticCode, DiagnosticStatus};
use protobuf::Message;

#[test]
fn test_diagnostic_status_round_trip() {
    let mut status = DiagnosticStatus::new();
    status.vin = "DEMO-VIN-001".to_string();

    let mut code = DiagnosticCode::new();
    code.code = "0x123456".to_string();
    code.lifecycle_state = "ACTIVE".to_string();
    status.active_codes.push(code);

    let bytes = status.write_to_bytes().expect("serialization failed");
    let parsed = DiagnosticStatus::parse_from_bytes(&bytes).expect("deserialization failed");

    assert_eq!(parsed.vin, "DEMO-VIN-001");
    assert_eq!(parsed.active_codes.len(), 1);
    assert_eq!(parsed.active_codes[0].code, "0x123456");
}
