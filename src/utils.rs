use std::time::SystemTime;

use rand::{CryptoRng, Rng};

pub(crate) fn create_uuid(rng: &mut (impl Rng + CryptoRng)) -> String {
    let bytes: [u8; 16] = rng.gen();
    uuid::Builder::from_random_bytes(bytes)
        .into_uuid()
        .to_string()
}

pub(crate) fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap()
}
