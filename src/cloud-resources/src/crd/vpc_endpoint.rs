// Copyright Materialize, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

//! VpcEndpoint custom resource, to be reconciled into an AWS VPC Endpoint by the
//! environment-controller.
use std::fmt;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, Time};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod v1 {
    use super::*;

    /// Describes an AWS VPC endpoint to create.
    #[derive(CustomResource, Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
    #[serde(rename_all = "camelCase")]
    #[kube(
        group = "materialize.cloud",
        version = "v1",
        kind = "VpcEndpoint",
        singular = "vpcendpoint",
        plural = "vpcendpoints",
        shortname = "vpce",
        namespaced,
        status = "VpcEndpointStatus",
        printcolumn = r#"{"name": "AwsServiceName", "type": "string", "description": "Name of the VPC Endpoint Service to connect to.", "jsonPath": ".spec.awsServiceName", "priority": 1}"#,
        printcolumn = r#"{"name": "AvailabilityZoneIDs", "type": "string", "description": "Availability Zone IDs", "jsonPath": ".spec.availabilityZoneIds", "priority": 1}"#
    )]
    // If making changes to this spec,
    // you must also update src/cloud-resources/gen/vpcendpoints.crd.json
    // so that cloudtest can register the CRD.
    pub struct VpcEndpointSpec {
        /// The name of the service to connect to.
        pub aws_service_name: String,
        /// The IDs of the availability zones in which the service is available.
        pub availability_zone_ids: Vec<String>,
        /// A suffix to use in the name of the IAM role that is created.
        pub role_suffix: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct VpcEndpointStatus {
        // This will be None if the customer hasn't allowed our principal, got the name of their
        // VPC Endpoint Service wrong, or we've otherwise failed to create the VPC Endpoint.
        pub vpc_endpoint_id: Option<String>,
        pub state: Option<VpcEndpointState>,
        pub conditions: Option<Vec<Condition>>,
        pub auto_assigned_azs: Option<Vec<String>>,
    }

    impl Default for VpcEndpointStatus {
        fn default() -> Self {
            Self {
                vpc_endpoint_id: None,
                state: Some(VpcEndpointState::Unknown),
                conditions: Some(Self::default_conditions()),
                auto_assigned_azs: None,
            }
        }
    }

    impl VpcEndpointStatus {
        pub fn default_conditions() -> Vec<Condition> {
            vec![Condition {
                type_: "Available".into(),
                status: "Unknown".to_string(),
                last_transition_time: Time(chrono::offset::Utc::now()),
                message: v1::VpcEndpointState::Unknown.message().into(),
                observed_generation: None,
                reason: "".into(),
            }]
        }
    }

    /// The AWS SDK State enum is not serializable, so we have to make our own.
    #[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub enum VpcEndpointState {
        // Internal States
        PendingServiceDiscovery,
        CreatingEndpoint,
        RecreatingEndpoint,
        UpdatingEndpoint,

        // AWS States
        // Connection established to the customer's VPC Endpoint Service.
        Available,
        Deleted,
        Deleting,
        Expired,
        Failed,
        // Customer has approved the connection. It should eventually move to Available.
        Pending,
        // Waiting on the customer to approve the connection.
        PendingAcceptance,
        Rejected,
        Unknown,
        // Could not place the endpoint in a subnet with the provided AZs
        MissingAvailabilityZones,
    }

    impl VpcEndpointState {
        // These are high level messages that can be used in conditions and
        // may be forwarded to end users. It is important to make them concise
        // and at a level which can be understand by end users of materialize
        pub fn message(&self) -> &str {
            match self {
                VpcEndpointState::PendingServiceDiscovery => {
                    "Endpoint cannot be discovered, ensure the Vpc Endpoint Service is allowing discovery"
                }
                VpcEndpointState::CreatingEndpoint => "Endpoint is being created",
                VpcEndpointState::RecreatingEndpoint => "Endpoint is being re-created",
                VpcEndpointState::UpdatingEndpoint => "Endpoint is being updated",
                VpcEndpointState::Available => "Endpoint is available",
                VpcEndpointState::Deleted => "Endpoint has been deleted",
                VpcEndpointState::Deleting => "Endpoint is being deleted",
                VpcEndpointState::Expired => {
                    "The Endpoint acceptance period has lapsed, you can still manually accept the Endpoint"
                }
                VpcEndpointState::Failed => "Endpoint creation has failed",
                VpcEndpointState::Pending => {
                    "Endpoint creation is pending, this should resolve shortly"
                }
                VpcEndpointState::PendingAcceptance => {
                    "The Endpoint connection to the Endpoint Service is pending acceptance"
                }
                VpcEndpointState::Rejected => {
                    "The Endpoint connection to the Endpoint Service has been rejected"
                }
                VpcEndpointState::Unknown => {
                    "The Endpoint is in an unknown state, this should resolve momentarily"
                }
                VpcEndpointState::MissingAvailabilityZones => {
                    "The Endpoint cannot be created due to missing availability zones"
                }
            }
        }
    }

    impl fmt::Display for VpcEndpointState {
        // Internal States
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let repr = match self {
                VpcEndpointState::PendingServiceDiscovery => "pendingServiceDiscovery",
                VpcEndpointState::CreatingEndpoint => "creatingEndpoint",
                VpcEndpointState::RecreatingEndpoint => "recreatingEndpoint",
                VpcEndpointState::UpdatingEndpoint => "updatingEndpoint",
                VpcEndpointState::Available => "available",
                VpcEndpointState::Deleted => "deleted",
                VpcEndpointState::Deleting => "deleting",
                VpcEndpointState::Expired => "expired",
                VpcEndpointState::Failed => "failed",
                VpcEndpointState::Pending => "pending",
                VpcEndpointState::PendingAcceptance => "pendingAcceptance",
                VpcEndpointState::Rejected => "rejected",
                VpcEndpointState::Unknown => "unknown",
                VpcEndpointState::MissingAvailabilityZones => "missingAvailabilityZones",
            };
            write!(f, "{}", repr)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use kube::CustomResourceExt;
    use kube::core::crd::merge_crds;

    #[mz_ore::test]
    fn test_vpc_endpoint_crd_matches() {
        let crd = merge_crds(vec![super::v1::VpcEndpoint::crd()], "v1").unwrap();
        let crd_json = serde_json::to_string(&serde_json::json!(&crd)).unwrap();
        let exported_crd_json = fs::read_to_string("src/crd/generated/vpcendpoints.json").unwrap();
        let exported_crd_json = exported_crd_json.trim();
        assert_eq!(
            &crd_json, exported_crd_json,
            "VpcEndpoint CRD json does not match exported json.\n\nCRD:\n{}\n\nExported CRD:\n{}",
            &crd_json, exported_crd_json,
        );
    }
}
