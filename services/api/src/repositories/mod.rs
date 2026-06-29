#![allow(dead_code)]

pub mod accounts;
pub mod holdings;
pub mod snapshots;
pub mod transactions;

pub fn dev_user_id() -> uuid::Uuid {
    uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("hardcoded dev user id must be a valid UUID")
}
