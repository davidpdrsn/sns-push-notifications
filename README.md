A small library for sending iOS and Android push notifications with Amazon Simple Notification
Servce (SNS).

## Example usage

```rust
use sns_push_notifications::{Push, Region, SnsClient};

let client = SnsClient::new_checked(Region::EuWest1)?;

let endpoint_arn = client.register_device(
    // not an actual token
    "123coi12j3vi12u3o1k23pb12e0jqpfw79g7w6fyi2o4jg293urf9q7ct9x1oi2h",
    // not an actual platform arn
    "arn:aws:sns:eu-west-1:000000000000:app/APNS/my-rusty-app",
)?;

client.send_push(
    &Push::Alert {
        text: "Hello, World!".to_string(),
        badge: Some(1),
    },
    &endpoint_arn,
)?;

```

---

License: MIT
