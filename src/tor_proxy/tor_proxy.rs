use libtor::{HiddenServiceVersion, Tor, TorAddress, TorFlag};

//blocking
pub fn tor(proxy_port: u16) {
    match Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::SocksPort(proxy_port))
        .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::HiddenServicePort(
            TorAddress::Port(8000),
            None.into(),
        ))
        .start()
    {
        Ok(r) =>{ println!("tor exit result: {}", r);},
        Err(e) =>{ eprintln!("tor error: {}", e); },
    };
}



