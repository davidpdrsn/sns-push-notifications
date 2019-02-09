//! A library for sending iOS and Android push notifications with Amazon Simple Notification
//! Servce (SNS).
//!
//! # Example usage
//!
//! ```no_run
//! use sns_push_notifications::{Push, Region, SnsClient};
//!
//! # fn main() -> Result<(), Box<std::error::Error>> {
//! let client = SnsClient::new_checked(Region::EuWest1)?;
//!
//! let endpoint_arn = client.register_device(
//!     // not an actual token
//!     "123coi12j3vi12u3o1k23pb12e0jqpfw79g7w6fyi2o4jg293urf9q7ct9x1oi2h",
//!     // not an actual platform arn
//!     "arn:aws:sns:eu-west-1:000000000000:app/APNS/my-rusty-app",
//! )?;
//!
//! client.send_push(
//!     &Push::Alert {
//!         text: "Hello, World!".to_string(),
//!         badge: Some(1),
//!     },
//!     &endpoint_arn,
//! )?;
//!
//! # Ok(())
//! # }
//! ```

#![deny(
    missing_docs,
    unused_mut,
    trivial_casts,
    trivial_numeric_casts,
    unused_variables,
    dead_code,
    unsafe_code,
    unused_imports
)]
#![doc(html_root_url = "https://docs.rs/sns-push-notifications/0.1.1")]

use rusoto_sns::CreatePlatformEndpointInput;
use rusoto_sns::PublishInput;
use rusoto_sns::Sns;
use serde::Serialize;
use serde_json::json;
use std::fmt;

pub use rusoto_core::region::Region;
pub use rusoto_sns::CreatePlatformEndpointError;
pub use rusoto_sns::PublishError;

/// A client for interacting with SNS
pub struct SnsClient {
    client: rusoto_sns::SnsClient,
}

impl SnsClient {
    /// Create a new client for a specific region.
    ///
    /// It requires that the `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` environment variables
    /// are set. It will return an error if either of them aren't set.
    pub fn new_checked(region: Region) -> Result<Self, Error> {
        check_for_credentials()?;

        Ok(SnsClient {
            client: rusoto_sns::SnsClient::new(region),
        })
    }

    /// Register a device with SNS and get back its corresponding ID.
    ///
    /// If a device with the given token has already been registered you'll get back the same ID.
    ///
    /// You can get the `platform_application_arn` from the SNS dashboard.
    pub fn register_device(
        &self,
        token: &str,
        platform_application_arn: &str,
    ) -> Result<EndpointArn, Error> {
        let res = self
            .client
            .create_platform_endpoint(CreatePlatformEndpointInput {
                platform_application_arn: platform_application_arn.to_string(),
                token: token.to_string(),
                ..Default::default()
            })
            .sync()?;

        Ok(res.endpoint_arn.unwrap())
    }

    /// Send a push notification to a specific endpoint arn.
    pub fn send_push(&self, push: &Push, endpoint_arn: &EndpointArn) -> Result<(), Error> {
        let payload = push.to_sns_payload();

        self.client
            .publish(PublishInput {
                message: payload,
                message_structure: Some("json".to_string()),
                target_arn: Some(endpoint_arn.clone()),
                ..Default::default()
            })
            .sync()?;

        Ok(())
    }
}

/// An ID that identifies a single device.
pub type EndpointArn = String;

/// A push notification to be sent.
#[derive(Debug)]
pub enum Push {
    /// A normal alert style push.
    Alert {
        /// The text that'll be shown on screen.
        text: String,

        /// The badge count to set. Requires platform support.
        badge: Option<i32>,
    },

    /// A silent push.
    ///
    /// Useful for waking up an app and performing work in the background.
    Silent {
        /// The badge count to set. Requires platform support.
        badge: Option<i32>,
    },
}

impl Push {
    fn to_sns_payload(&self) -> String {
        let (ios, android) = match self {
            Push::Alert { text, badge } => {
                let ios = json!({
                    "aps": {
                      "alert": text,
                      "badge": badge,
                    }
                });

                let android = json!({
                  "data": {
                    "message": text,
                    "badge": badge,
                  }
                });

                (ios, android)
            }
            Push::Silent { badge } => {
                let ios = json!({
                    "aps": {
                        "content-available": 1,
                        "badge": badge,
                    }
                });

                let android = json!({
                  "data": {}
                });

                (ios, android)
            }
        };

        let payload = json!({
            "default": "",
            "APNS": json_to_string(&ios),
            "APNS_SANDBOX": json_to_string(&ios),
            "GCM": json_to_string(&android),
        });

        json_to_string(&payload)
    }
}

fn json_to_string<S: Serialize>(s: &S) -> String {
    serde_json::to_string(s).unwrap()
}

/// The errors this library might generate.
#[derive(Debug)]
pub enum Error {
    /// An error related to registering a device.
    PublishError(PublishError),

    /// An error related to publishing a push.
    RegisterDeviceError(CreatePlatformEndpointError),

    /// An error related to missing credential environment variables.
    MissingCredentials(MissingCredentials),
}

/// Error telling you which env var was missing
#[derive(Debug)]
pub enum MissingCredentials {
    /// `AWS_ACCESS_KEY_ID` as missing
    AccessKeyId,
    /// `AWS_SECRET_ACCESS_KEY` as missing
    SecretAccessKey,
    /// Both `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` are missing
    Both,
}

impl From<CreatePlatformEndpointError> for Error {
    fn from(inner: CreatePlatformEndpointError) -> Self {
        Error::RegisterDeviceError(inner)
    }
}

impl From<PublishError> for Error {
    fn from(inner: PublishError) -> Self {
        Error::PublishError(inner)
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::PublishError(inner) => write!(f, "{}", inner),
            Error::RegisterDeviceError(inner) => write!(f, "{}", inner),

            Error::MissingCredentials(MissingCredentials::AccessKeyId) => {
                write!(f, "`AWS_ACCESS_KEY_ID` env var is missing")
            }
            Error::MissingCredentials(MissingCredentials::SecretAccessKey) => {
                write!(f, "`AWS_SECRET_ACCESS_KEY` env var is missing")
            }
            Error::MissingCredentials(MissingCredentials::Both) => write!(
                f,
                "Both `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` env vars are missing"
            ),
        }
    }
}

fn check_for_credentials() -> Result<(), Error> {
    let id = std::env::var("AWS_ACCESS_KEY_ID").ok();
    let key = std::env::var("AWS_SECRET_ACCESS_KEY").ok();

    match (id, key) {
        (Some(_), Some(_)) => Ok(()),
        (Some(_), None) => Err(Error::MissingCredentials(
            MissingCredentials::SecretAccessKey,
        )),
        (None, Some(_)) => Err(Error::MissingCredentials(MissingCredentials::AccessKeyId)),
        (None, None) => Err(Error::MissingCredentials(MissingCredentials::Both)),
    }
}
