static SUPER_SECRET : Option<[u8; 32]> = None;

fn random_bytes() -> [u8; 32] {
    let mut secret = [0u8; 32];
    thread_rng().fill_bytes(&mut secret);
    secret
}

fn receive_super_secret(config: &Config) -> Result<[u8; 32], MyError> {
    if let Some(super_secret) = SUPER_SECRET {
        super_secret
    } else {
        let bytes = File::open(config.super_secret_file)?.read();
        let result = <&[u8; 32]>::try_from(&bytes)?;
        Ok(*result)
    }
}
