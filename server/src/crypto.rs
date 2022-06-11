use std::fs::File;
use std::io::{Read, Write};
use ethsign::Protected;
use rand::{RngCore, thread_rng};
use crate::Config;
use crate::errors::MyError;

pub fn random_bytes() -> [u8; 32] {
    let mut secret = [0u8; 32];
    thread_rng().fill_bytes(&mut secret);
    secret
}

static SUPER_SECRET: Option<Protected> = None;

pub fn receive_super_secret(config: &Config) -> Result<Protected, MyError> {
    if let Some(ref super_secret) = SUPER_SECRET {
        Ok(super_secret.clone())
    } else {
        let bytes = if let Ok(mut file) = File::open(config.super_secret_file.clone()) {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            bytes
        } else {
            let bytes = random_bytes();
            let mut file = File::create(config.super_secret_file.clone())?;
            file.write(bytes.as_slice())?;
            bytes.to_vec()
        };
        Ok(Protected::new(bytes))
    }
}
