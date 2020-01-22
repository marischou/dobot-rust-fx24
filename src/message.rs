use crate::error::{Error as DobotError, Result as DobotResult};
use getset::{CopyGetters, Getters};
use std::convert::TryInto;

#[derive(Clone, Debug, Getters, CopyGetters)]
pub struct DobotMessage {
    #[get = "pub"]
    header: [u8; 2],
    #[get_copy = "pub"]
    len: u8,
    #[get_copy = "pub"]
    id: u8,
    #[get_copy = "pub"]
    ctrl: u8,
    #[get = "pub"]
    params: Vec<u8>,
    #[get_copy = "pub"]
    checksum: u8,
}

impl DobotMessage {
    pub fn new(id: u8, ctrl: u8, params: Vec<u8>) -> DobotResult<Self> {
        if params.len() > u8::max_value() as usize + 2 {
            return Err(DobotError::ParamsTooLong);
        }

        let len = params.len() as u8 + 2;
        let checksum = Self::compute_checksum(id, ctrl, &params);

        let msg = Self {
            header: [0xaa, 0xaa],
            len,
            id,
            ctrl,
            params,
            checksum,
        };

        Ok(msg)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.header
            .iter()
            .chain([self.len].iter())
            .chain([self.id].iter())
            .chain([self.ctrl].iter())
            .chain(self.params.iter())
            .chain([self.checksum].iter())
            .map(|byte| *byte)
            .collect::<Vec<u8>>()
    }

    pub fn from_bytes(bytes: &[u8]) -> DobotResult<Self> {
        if bytes.len() < 6 {
            return Err(DobotError::DeserializeError("message is truncated".into()));
        }

        let header: [u8; 2] = bytes[0..2].try_into().unwrap();
        let len = bytes[2];

        if bytes.len() != len as usize + 6 {
            return Err(DobotError::DeserializeError("message is truncated".into()));
        }

        let id = bytes[3];
        let ctrl = bytes[4];
        let params = bytes[5..(bytes.len() - 1)]
            .into_iter()
            .map(|byte| *byte)
            .collect::<Vec<u8>>();
        let checksum = bytes[bytes.len() - 1];

        {
            let expected = Self::compute_checksum(id, ctrl, &params);
            if expected != checksum {
                return Err(DobotError::IntegrityError {
                    expected,
                    received: checksum,
                });
            }
        }

        let msg = Self {
            header,
            len,
            id,
            ctrl,
            params,
            checksum,
        };

        Ok(msg)
    }

    fn compute_checksum(id: u8, ctrl: u8, params: &[u8]) -> u8 {
        let (checksum, _) = id.overflowing_add(ctrl);
        let checksum = params.iter().fold(0u8, |prev_cksum, byte| {
            let (new_cksum, _) = prev_cksum.overflowing_add(*byte);
            new_cksum
        }) + checksum;
        let (checksum, _) = checksum.overflowing_neg();
        checksum
    }
}
