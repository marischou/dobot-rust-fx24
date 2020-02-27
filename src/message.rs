use crate::{
    base::CommandID,
    error::{Error as DobotError, Result as DobotResult},
};
use getset::{CopyGetters, Getters};
use num_traits::FromPrimitive;
use std::{array::FixedSizeArray, convert::TryInto, io::prelude::*, marker::Unpin};
use tokio::io::AsyncReadExt;

/// The message format of Dobot protocol.
#[derive(Clone, Debug, Getters, CopyGetters)]
pub struct DobotMessage {
    #[get = "pub"]
    header: [u8; 2],
    #[get_copy = "pub"]
    len: u8,
    #[get_copy = "pub"]
    id: CommandID,
    #[get_copy = "pub"]
    rw: bool,
    #[get_copy = "pub"]
    is_queued: bool,
    #[get = "pub"]
    params: Vec<u8>,
    #[get_copy = "pub"]
    checksum: u8,
}

impl DobotMessage {
    /// Create message object.
    pub fn new(id: CommandID, rw: bool, is_queued: bool, params: Vec<u8>) -> DobotResult<Self> {
        if params.len() > u8::max_value() as usize + 2 {
            return Err(DobotError::ParamsTooLong);
        }

        let len = params.len() as u8 + 2;
        let checksum = Self::compute_checksum(id, rw, is_queued, &params);

        let msg = Self {
            header: [0xaa, 0xaa],
            len,
            id,
            rw,
            is_queued,
            params,
            checksum,
        };

        Ok(msg)
    }

    /// Serialize message to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let ctrl = ((self.is_queued as u8) << 1) | (self.rw as u8);
        self.header
            .iter()
            .chain([self.len].iter())
            .chain([self.id as u8].iter())
            .chain([ctrl].iter())
            .chain(self.params.iter())
            .chain([self.checksum].iter())
            .map(|byte| *byte)
            .collect::<Vec<u8>>()
    }

    /// Create message from serialized bytes.
    pub fn from_bytes<B>(bytes: B) -> DobotResult<Self>
    where
        B: AsRef<[u8]>,
    {
        let as_ref = bytes.as_ref();
        if as_ref.len() < 6 {
            return Err(DobotError::DeserializeError("message is truncated".into()));
        }

        let header: [u8; 2] = as_ref[0..2].try_into().unwrap();
        let len = as_ref[2];

        if as_ref.len() != len as usize + 4 {
            return Err(DobotError::DeserializeError("message is truncated".into()));
        }

        let id = CommandID::from_u8(as_ref[3]).ok_or(DobotError::DeserializeError(format!(
            "unrecognized command ID {}",
            as_ref[3]
        )))?;
        let ctrl = as_ref[4];
        let rw = (ctrl & 0x01) != 0;
        let is_queued = (ctrl & 0x02) != 0;
        let params = as_ref[5..(as_ref.len() - 1)]
            .into_iter()
            .map(|byte| *byte)
            .collect::<Vec<u8>>();
        let checksum = as_ref[as_ref.len() - 1];

        {
            let expected = Self::compute_checksum(id, rw, is_queued, &params);
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
            rw,
            is_queued,
            params,
            checksum,
        };

        Ok(msg)
    }

    /// Create message by synchronously reading bytes from reader.
    pub fn from_reader<R>(mut reader: R) -> DobotResult<Self>
    where
        R: Read,
    {
        let header_buffer = {
            let mut header = [0; 2];
            reader.read_exact(&mut header)?;
            header
        };
        let len_buffer = {
            let mut len = [0; 1];
            reader.read_exact(&mut len)?;
            len
        };
        let len = len_buffer[0];
        let data_buffer = {
            let mut data = vec![0; len as usize];
            reader.read_exact(&mut data)?;
            data
        };
        let bytes = [
            header_buffer.as_slice(),
            len_buffer.as_slice(),
            data_buffer.as_slice(),
        ]
        .concat();
        let msg = Self::from_bytes(bytes)?;
        Ok(msg)
    }

    /// Create message by asynchronously reading bytes from reader.
    pub async fn from_async_reader<R>(mut reader: R) -> DobotResult<Self>
    where
        R: AsyncReadExt + Unpin,
    {
        let prefix = {
            let mut prefix = [0u8; 5];
            reader.read_exact(&mut prefix).await?;
            prefix
        };
        let len = prefix[2];
        let suffix = {
            let mut suffix = vec![0u8; len as usize - 1];
            reader.read_exact(suffix.as_mut_slice()).await?;
            suffix
        };
        let bytes = [prefix.as_slice(), suffix.as_slice()].concat();
        let msg = Self::from_bytes(bytes)?;
        Ok(msg)
    }

    fn compute_checksum(id: CommandID, rw: bool, is_queued: bool, params: &[u8]) -> u8 {
        let ctrl = ((is_queued as u8) << 1) | (rw as u8);
        let (checksum, _) = (id as u8).overflowing_add(ctrl);
        let (checksum, _) = params
            .iter()
            .fold(0u8, |prev_cksum, byte| {
                let (new_cksum, _) = prev_cksum.overflowing_add(*byte);
                new_cksum
            })
            .overflowing_add(checksum);
        let (checksum, _) = checksum.overflowing_neg();
        checksum
    }
}
