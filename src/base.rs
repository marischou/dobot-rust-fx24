use crate::{
    error::{Error as DobotError, Result as DobotResult},
    message::DobotMessage,
};
use num_derive::FromPrimitive;
use std::{convert::TryInto, path::Path, time::Duration};
use tokio::io::AsyncWriteExt;
use tokio_serial::{DataBits, FlowControl, Parity, SerialStream, SerialPortBuilderExt, StopBits};

/// Defines the format to describe the robot pose.
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

/// Describes the pose of robot arm.
#[derive(Debug, Clone, PartialEq)]
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

/// The Dobot robot arm controller type.
pub struct Dobot {
    serial: SerialStream,
}

impl Dobot {
    /// Create controller object from device file.
    pub async fn open<P>(path: P) -> DobotResult<Self>
    where
        P: AsRef<Path>,
    {
        let internal_path = path.as_ref().to_str().ok_or(Err(DeserializeError("path failed to be parsed".to_string()))?;
        let serial = tokio_serial::new(internal_path, 115200)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .timeout(Duration::from_secs(1)).open_native_async()?;

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

    pub async fn set_ptp_joint_params<'a>(
        &'a mut self,
        v_x: f32,
        v_y: f32,
        v_z: f32,
        v_r: f32,
        a_x: f32,
        a_y: f32,
        a_z: f32,
        a_r: f32,
    ) -> DobotResult<WaitHandle<'a>> {
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

        let response_msg = self
            .send_command(DobotMessage::new(
                CommandID::GetSetPtpJointParams,
                true,
                true,
                params,
            )?)
            .await?;
        let index = u64::from_le_bytes(response_msg.params()[0..8].try_into().unwrap());

        let handle = WaitHandle::new(self, index);
        Ok(handle)
    }

    pub async fn set_cp_cmd<'a>(
        &'a mut self,
        x: f32,
        y: f32,
        z: f32,
    ) -> DobotResult<WaitHandle<'a>> {
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

        let response_msg = self
            .send_command(DobotMessage::new(CommandID::SetCpCmd, true, true, params)?)
            .await?;
        let index = u64::from_le_bytes(response_msg.params()[0..8].try_into().unwrap());

        let handle = WaitHandle::new(self, index);
        Ok(handle)
    }

    pub async fn set_ptp_coordinate_params<'a>(
        &'a mut self,
        velocity: f32,
        acceleration: f32,
    ) -> DobotResult<WaitHandle<'a>> {
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

        let response_msg = self
            .send_command(DobotMessage::new(
                CommandID::GetSetPtpCoordinateParams,
                true,
                true,
                params,
            )?)
            .await?;
        let index = u64::from_le_bytes(response_msg.params()[0..8].try_into().unwrap());

        let handle = WaitHandle::new(self, index);
        Ok(handle)
    }

    pub async fn set_ptp_jump_params<'a>(
        &'a mut self,
        jump: f32,
        limit: f32,
    ) -> DobotResult<WaitHandle<'a>> {
        let params = [jump.to_le_bytes(), limit.to_le_bytes()]
            .iter()
            .flatten()
            .map(|byte| *byte)
            .collect::<Vec<u8>>();

        let response_msg = self
            .send_command(DobotMessage::new(
                CommandID::GetSetPtpJumpParams,
                true,
                true,
                params,
            )?)
            .await?;
        let index = u64::from_le_bytes(response_msg.params()[0..8].try_into().unwrap());

        let handle = WaitHandle::new(self, index);
        Ok(handle)
    }

    pub async fn set_ptp_common_params<'a>(
        &'a mut self,
        velocity: f32,
        acceleration: f32,
    ) -> DobotResult<WaitHandle<'a>> {
        let params = [velocity.to_le_bytes(), acceleration.to_le_bytes()]
            .iter()
            .flatten()
            .map(|byte| *byte)
            .collect::<Vec<u8>>();

        let response_msg = self
            .send_command(DobotMessage::new(
                CommandID::GetSetPtpCommonParams,
                true,
                true,
                params,
            )?)
            .await?;
        let index = u64::from_le_bytes(response_msg.params()[0..8].try_into().unwrap());

        let handle = WaitHandle::new(self, index);
        Ok(handle)
    }

    pub async fn set_ptp_cmd<'a>(
        &'a mut self,
        x: f32,
        y: f32,
        z: f32,
        r: f32,
        mode: Mode,
    ) -> DobotResult<WaitHandle<'a>> {
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
            DobotMessage::new(CommandID::SetPtpCmd, true, true, params)?
        };

        let response_msg = self.send_command(request_msg).await?;
        let params = response_msg.params();
        let index = u64::from_le_bytes(params[0..8].try_into().unwrap());

        let handle = WaitHandle::new(self, index);
        Ok(handle)
    }

    pub async fn set_end_effector_suction_cup<'a>(
        &'a mut self,
        enable: bool,
    ) -> DobotResult<WaitHandle<'a>> {
        let params = vec![0x01, enable as u8];
        let response_msg = self
            .send_command(DobotMessage::new(
                CommandID::GetSetEndEffectorSuctionCup,
                true,
                true,
                params,
            )?)
            .await?;
        let index = u64::from_le_bytes(response_msg.params()[0..8].try_into().unwrap());

        let handle = WaitHandle::new(self, index);
        Ok(handle)
    }

    pub async fn set_end_effector_gripper<'a>(
        &'a mut self,
        enable: bool,
    ) -> DobotResult<WaitHandle<'a>> {
        let params = vec![0x01, enable as u8];
        let response_msg = self
            .send_command(DobotMessage::new(
                CommandID::GetSetEndEffectorGripper,
                true,
                true,
                params,
            )?)
            .await?;
        let index = u64::from_le_bytes(response_msg.params()[0..8].try_into().unwrap());
        let handle = WaitHandle::new(self, index);
        Ok(handle)
    }

    pub async fn set_queued_cmd_start_exec(&mut self) -> DobotResult<()> {
        self.send_command(DobotMessage::new(
            CommandID::SetQueuedCmdStartExec,
            true,
            false,
            vec![],
        )?)
        .await?;
        Ok(())
    }

    pub async fn set_queued_cmd_stop_exec(&mut self) -> DobotResult<()> {
        self.send_command(DobotMessage::new(
            CommandID::SetQueuedCmdStopExec,
            true,
            false,
            vec![],
        )?)
        .await?;
        Ok(())
    }

    pub async fn set_queued_cmd_clear(&mut self) -> DobotResult<()> {
        self.send_command(DobotMessage::new(
            CommandID::SetQueuedCmdClear,
            true,
            false,
            vec![],
        )?)
        .await?;
        Ok(())
    }

    pub async fn get_queued_cmd_current_index(&mut self) -> DobotResult<u64> {
        let request_msg =
            DobotMessage::new(CommandID::SetQueuedCmdCurrentIndex, false, false, vec![])?;
        let response_msg = self.send_command(request_msg).await?;
        let params = response_msg.params();
        let index = u64::from_le_bytes(params[0..8].try_into().unwrap());
        Ok(index)
    }

    /// Grips on end effector.
    pub async fn grip<'a>(&'a mut self) -> DobotResult<WaitHandle<'a>> {
        let handle = self.set_end_effector_gripper(true).await?;
        Ok(handle)
    }

    /// Releases gripper on end effector.
    pub async fn release<'a>(&'a mut self) -> DobotResult<WaitHandle<'a>> {
        let handle = self.set_end_effector_gripper(false).await?;
        Ok(handle)
    }

    /// Starts the calibration process.
    pub async fn set_home<'a>(&'a mut self) -> DobotResult<WaitHandle<'a>> {
        let request_msg = DobotMessage::new(CommandID::SetHomeCmd, true, true, vec![])?;
        let response_msg = self.send_command(request_msg).await?;
        let params = response_msg.params();
        let index = u64::from_le_bytes(params[0..8].try_into().unwrap());
        let handle = WaitHandle::new(self, index);
        Ok(handle)
    }

    /// Get the current pose of robot.
    pub async fn get_pose(&mut self) -> DobotResult<Pose> {
        let request_msg = DobotMessage::new(CommandID::GetPose, false, false, vec![])?;
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

    /// Move to given pose.
    pub async fn move_to<'a>(
        &'a mut self,
        x: f32,
        y: f32,
        z: f32,
        r: f32,
    ) -> DobotResult<WaitHandle<'a>> {
        let handle = self
            .set_ptp_cmd(x, y, z, r, Mode::MODE_PTP_MOVL_XYZ)
            .await?;
        Ok(handle)
    }

    /// Send user-defined request to Dobot and obtain response.
    pub async fn send_command(&mut self, request_msg: DobotMessage) -> DobotResult<DobotMessage> {
        // send message
        self.serial
            .write_all(request_msg.to_bytes().as_slice())
            .await?;

        // receive message
        let response_msg = DobotMessage::from_async_reader(&mut self.serial).await?;

        Ok(response_msg)
    }
}

pub struct WaitHandle<'a> {
    command_index: u64,
    dobot: &'a mut Dobot,
}

impl<'a> WaitHandle<'a> {
    pub(crate) fn new(dobot: &'a mut Dobot, command_index: u64) -> Self {
        Self {
            command_index,
            dobot,
        }
    }

    pub async fn wait(self) -> DobotResult<()> {
        loop {
            let current_index = self.dobot.get_queued_cmd_current_index().await?;
            if current_index == self.command_index {
                break Ok(());
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive)]
pub enum CommandID {
    GetSetDeviceSn = 0,
    GetSetDeviceName = 1,
    GetDeviceVersion = 2,
    GetDeviceWithL = 3,
    GetDeviceTime = 4,
    // GetDeviceId = 0, // 4
    GetPose = 10,
    ResetPose = 11,
    GetPoseL = 13,
    GetAlarmsState = 20,
    ClearAllAlarmsState = 21,
    GetSetHomeParams = 30,
    SetHomeCmd = 31,
    // GetSetAutoLeveling = 0, // 30
    GetSetHHTTrigMode = 40,
    GetSetHHTTrigOutputEnabled = 41,
    GetHHTTrigOutput = 42,
    GetSetEndEffectorParams = 60,
    GetSetEndEffectorLaser = 61,
    GetSetEndEffectorSuctionCup = 62,
    GetSetEndEffectorGripper = 63,
    GetSetJogJointParams = 70,
    GetSetJogCoordinateParams = 71,
    GetSetJogCommonParams = 72,
    SetJogCmd = 73,
    GetSetJogLParams = 74,
    GetSetPtpJointParams = 80,
    GetSetPtpCoordinateParams = 81,
    GetSetPtpJumpParams = 82,
    GetSetPtpCommonParams = 83,
    SetPtpCmd = 84,
    GetSetPtpLParams = 85,
    SetPtpWithLCmd = 86,
    GetSetPtpJump2Params = 87,
    SetPtpPoCmd = 88,
    SetPtpPoWithLCmd = 89,
    GetSetCpParams = 90,
    SetCpCmd = 91,
    SetCpLeCmd = 92,
    GetSetArcParams = 100,
    SetSetArcCmd = 101,
    SetWaitCmd = 110,
    SetTrigCmd = 120,
    GetSetIoMultiplexing = 130,
    GetSetIoDo = 131,
    GetSetIoPwm = 132,
    GetIoDi = 133,
    GetIoAdc = 134,
    SetEMotor = 135,
    GetSetColorSensor = 137,
    GetSetIrSwitch = 138,
    GetSetAngleSensorStaticError = 140,
    GetSetWifiConfigMode = 150,
    GetSetWifiSsid = 151,
    GetSetWifiPassword = 152,
    GetSetWifiAddress = 153,
    GetSetWifiNetmask = 154,
    GetSetWifiGateway = 155,
    GetSetWifiDns = 156,
    GetSetWifiConnectStatus = 157,
    SetLostStepParams = 170,
    SetLostStepCmd = 171,
    SetQueuedCmdStartExec = 240,
    SetQueuedCmdStopExec = 241,
    SetQueuedCmdForceStopExec = 242,
    SetQueuedCmdStartDownload = 243,
    SetQueuedCmdStopDownload = 244,
    SetQueuedCmdClear = 245,
    SetQueuedCmdCurrentIndex = 246,
}
