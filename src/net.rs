use std::collections::HashMap;

pub struct Net {
    idx: u32,
}

impl Net {
    pub fn new(mac: [u8; 6]) -> Self {
        Net { idx: 0 }
    }

    pub fn list() -> Result<HashMap<u32, Vec<u8>>, NetError> {
        let ifaces = netwatcher::list_interfaces()?;
        let ifaces = ifaces
            .into_iter()
            .filter_map(|(k, v)| {
                let mac: Result<Vec<u8>, _> =
                    v.hw_addr.split(':').try_fold(Vec::new(), |mut acc, hex| {
                        u8::from_str_radix(hex, 16).map(|byte| {
                            acc.push(byte);
                            acc
                        })
                    });
                mac.ok().map(|mac| (k, mac))
            })
            .collect::<HashMap<u32, Vec<u8>>>();
        Ok(ifaces)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error(transparent)]
    NetWatcher(#[from] netwatcher::Error),
}
