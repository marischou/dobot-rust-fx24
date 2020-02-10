use crate::{
    error::{Error as DobotError, Result as DobotResult},
    message::DobotMessage,
};
use std::{convert::TryInto, path::Path, time::Duration};
use tokio::io::AsyncWriteExt;
use tokio_serial::{DataBits, FlowControl, Parity, Serial, SerialPortSettings, StopBits};

#[derive(Debug, Clone)]
pub enum Mode {
    #[allow(non_camel_case_types)]
    MODE_PTP_JUMP_XYZ = 0x00,
    #[allow(non_camel_case_types)]
    MODE_PTP_MOVJ_XYZ = 0x01,
    #[allow(non_camel_case_types)]
    MODE_PTP_MOVL_XYZ = 0x02,
    #[allow(non_camel_case_types)]
    MODE_PTP_JUMP_ANGLE = 0x03,
    #[allow(non_camel_case_types)]
    MODE_PTP_MOVJ_ANGLE = 0x04,
    #[allow(non_camel_case_types)]
    MODE_PTP_MOVL_ANGLE = 0x05,
    #[allow(non_camel_case_types)]
    MODE_PTP_MOVJ_INC = 0x06,
    #[allow(non_camel_case_types)]
    MODE_PTP_MOVL_INC = 0x07,
    #[allow(non_camel_case_types)]
    MODE_PTP_MOVJ_XYZ_INC = 0x08,
    #[allow(non_camel_case_types)]
    MODE_PTP_JUMP_MOVL_XYZ = 0x09,
}

#[derive(Debug, Clone)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub j1: f32,
    pub j2: f32,
    pub j3: f32,
    pub j4: f32,
}

pub struct Dobot {
    serial: Serial,
}

impl Dobot {
    pub async fn open<P>(path: P) -> DobotResult<Self>
    where
        P: AsRef<Path>,
    {
        let serial = {
            let settings = SerialPortSettings {
                baud_rate: 115200,
                data_bits: DataBits::Eight,
                flow_control: FlowControl::None,
                parity: Parity::None,
                stop_bits: StopBits::One,
                timeout: Duration::from_secs(1),
            };

            let serial = Serial::from_path(path, &settings)?;
            serial
        };

        let mut dobot = Self { serial };

        dobot.set_queued_cmd_start_exec().await?;
        dobot.set_queued_cmd_clear().await?;
        dobot
            .set_ptp_joint_params(200.0, 200.0, 200.0, 200.0, 200.0, 200.0, 200.0, 200.0)
            .await?;
        dobot.set_ptp_coordinate_params(200.0, 200.0).await?;
        dobot.set_ptp_jump_params(10.0, 200.0).await?;
        dobot.set_ptp_common_params(100.0, 100.0).await?;
        dobot.get_pose().await?;

        Ok(dobot)
    }

    pub async fn set_ptp_joint_params(
        &mut self,
        v_x: f32,
        v_y: f32,
        v_z: f32,
        v_r: f32,
        a_x: f32,
        a_y: f32,
        a_z: f32,
        a_r: f32,
    ) -> DobotResult<()> {
        let params = [
            v_x.to_le_bytes(),
            v_y.to_le_bytes(),
            v_z.to_le_bytes(),
            v_r.to_le_bytes(),
            a_x.to_le_bytes(),
            a_y.to_le_bytes(),
            a_z.to_le_bytes(),
            a_r.to_le_bytes(),
        ]
        .iter()
        .flatten()
        .map(|byte| *byte)
        .collect::<Vec<u8>>();

        self.send_command(DobotMessage::new(80, 0x03, params).unwrap())
            .await?;
        Ok(())
    }

    pub async fn set_cp_cmd(&mut self, x: f32, y: f32, z: f32) -> DobotResult<()> {
        let params = [0x01]
            .iter()
            .chain(
                [x.to_le_bytes(), y.to_le_bytes(), z.to_le_bytes()]
                    .iter()
                    .flatten(),
            )
            .chain([0x00].iter())
            .map(|byte| *byte)
            .collect::<Vec<u8>>();

        self.send_command(DobotMessage::new(91, 0x03, params).unwrap())
            .await?;
        Ok(())
    }

    pub async fn set_ptp_coordinate_params(
        &mut self,
        velocity: f32,
        acceleration: f32,
    ) -> DobotResult<()> {
        let params = [
            velocity.to_le_bytes(),
            velocity.to_le_bytes(),
            acceleration.to_le_bytes(),
            acceleration.to_le_bytes(),
        ]
        .iter()
        .flatten()
        .map(|byte| *byte)
        .collect::<Vec<u8>>();

        self.send_command(DobotMessage::new(81, 0x03, params).unwrap())
            .await?;
        Ok(())
    }

    pub async fn set_ptp_jump_params(&mut self, jump: f32, limit: f32) -> DobotResult<()> {
        let params = [jump.to_le_bytes(), limit.to_le_bytes()]
            .iter()
            .flatten()
            .map(|byte| *byte)
            .collect::<Vec<u8>>();

        self.send_command(DobotMessage::new(82, 0x03, params).unwrap())
            .await?;
        Ok(())
    }

    pub async fn set_ptp_common_params(
        &mut self,
        velocity: f32,
        acceleration: f32,
    ) -> DobotResult<()> {
        let params = [velocity.to_le_bytes(), acceleration.to_le_bytes()]
            .iter()
            .flatten()
            .map(|byte| *byte)
            .collect::<Vec<u8>>();

        self.send_command(DobotMessage::new(83, 0x03, params).unwrap())
            .await?;
        Ok(())
    }

    pub async fn set_ptp_cmd(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        r: f32,
        mode: Mode,
        wait: bool,
    ) -> DobotResult<()> {
        let request_msg = {
            let params = [mode as u8]
                .iter()
                .chain(
                    [
                        x.to_le_bytes(),
                        y.to_le_bytes(),
                        z.to_le_bytes(),
                        r.to_le_bytes(),
                    ]
                    .iter()
                    .flatten(),
                )
                .map(|byte| *byte)
                .collect::<Vec<u8>>();
            DobotMessage::new(84, 0x03, params).unwrap()
        };

        let response_msg = self.send_command(request_msg).await?;

        if !wait {
            return Ok(());
        }

        let expected_index = {
            let params = response_msg.params();
            u32::from_le_bytes(params[0..4].try_into().unwrap())
        };

        loop {
            let current_index = self.get_queued_cmd_current_index().await?;
            if current_index == expected_index {
                break;
            }
        }

        Ok(())
    }

    pub async fn set_end_effector_suction_cup(&mut self, enable: bool) -> DobotResult<()> {
        let params = vec![0x01, enable as u8];
        self.send_command(DobotMessage::new(62, 0x03, params).unwrap())
            .await?;
        Ok(())
    }

    pub async fn set_end_effector_gripper(&mut self, enable: bool) -> DobotResult<()> {
        let params = vec![0x01, enable as u8];
        self.send_command(DobotMessage::new(63, 0x03, params).unwrap())
            .await?;
        Ok(())
    }

    pub async fn set_queued_cmd_start_exec(&mut self) -> DobotResult<()> {
        self.send_command(DobotMessage::new(240, 0x01, vec![]).unwrap())
            .await?;
        Ok(())
    }

    pub async fn set_queued_cmd_stop_exec(&mut self) -> DobotResult<()> {
        self.send_command(DobotMessage::new(241, 0x01, vec![]).unwrap())
            .await?;
        Ok(())
    }

    pub async fn set_queued_cmd_clear(&mut self) -> DobotResult<()> {
        self.send_command(DobotMessage::new(245, 0x01, vec![]).unwrap())
            .await?;
        Ok(())
    }

    pub async fn get_queued_cmd_current_index(&mut self) -> DobotResult<u32> {
        let request_msg = DobotMessage::new(246, 0x00, vec![]).unwrap();
        let response_msg = self.send_command(request_msg).await?;
        let params = response_msg.params();
        let index = u32::from_le_bytes(params[0..4].try_into().unwrap());
        Ok(index)
    }

    pub async fn get_pose(&mut self) -> DobotResult<Pose> {
        let request_msg = DobotMessage::new(10, 0x00, vec![]).unwrap();
        let response_msg = self.send_command(request_msg).await?;

        let params = {
            let params = response_msg.params();
            if params.len() != 32 {
                return Err(DobotError::DeserializeError("message is truncated".into()));
            }
            params
        };

        let x = f32::from_le_bytes(params[0..4].try_into().unwrap());
        let y = f32::from_le_bytes(params[4..8].try_into().unwrap());
        let z = f32::from_le_bytes(params[8..12].try_into().unwrap());
        let r = f32::from_le_bytes(params[12..16].try_into().unwrap());
        let j1 = f32::from_le_bytes(params[16..20].try_into().unwrap());
        let j2 = f32::from_le_bytes(params[20..24].try_into().unwrap());
        let j3 = f32::from_le_bytes(params[24..28].try_into().unwrap());
        let j4 = f32::from_le_bytes(params[28..32].try_into().unwrap());

        let pose = Pose {
            x,
            y,
            z,
            r,
            j1,
            j2,
            j3,
            j4,
        };

        Ok(pose)
    }

    pub async fn move_to(&mut self, x: f32, y: f32, z: f32, r: f32, wait: bool) -> DobotResult<()> {
        self.set_ptp_cmd(x, y, z, r, Mode::MODE_PTP_MOVL_XYZ, wait)
            .await?;
        Ok(())
    }

    async fn send_command(&mut self, request_msg: DobotMessage) -> DobotResult<DobotMessage> {
        // send message
        self.serial
            .write_all(request_msg.to_bytes().as_slice())
            .await?;

        // recive message
        let response_msg = DobotMessage::from_async_reader(&mut self.serial).await?;

        Ok(response_msg)
    }
}
