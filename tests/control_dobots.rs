#![cfg(feature = "dobot-test")]

use dobot::{error::Result as DobotResult, Dobot};
use failure::Fallible;
use tokio::runtime::Runtime;

#[test]
fn control_dobot_test() -> Fallible<()> {
    let mut runtime = Runtime::new()?;
    runtime.block_on(async {
        let mut dobot = Dobot::open("/dev/ttyUSB0").await?;
        dobot.move_to(10.0, 10.0, 10.0, 0.0, false).await?;
        DobotResult::Ok(())
    })?;
    Ok(())
}
