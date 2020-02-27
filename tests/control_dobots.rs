#![cfg(feature = "dobot-test")]

use dobot::{error::Result as DobotResult, Dobot};
use failure::Fallible;
use serde::Deserialize;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
    path::PathBuf,
};
use tokio::runtime::Runtime;

#[derive(Deserialize)]
struct Config {
    pub device_file: PathBuf,
}

#[test]
fn control_dobot_test() -> Fallible<()> {
    let config: Config = {
        let mut reader = BufReader::new(File::open("tests/test_config.toml")?);
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        toml::from_str(&string)?
    };

    let Config { device_file } = config;

    let mut runtime = Runtime::new()?;
    runtime.block_on(async {
        let mut dobot = Dobot::open(&device_file).await?;

        println!("pose {:?}", dobot.get_pose().await?);

        dobot.move_to(100.0, 100.0, 0.0, 0.0).await?.wait().await?;
        println!("pose {:?}", dobot.get_pose().await?);

        dobot.move_to(100.0, 200.0, 0.0, 0.0).await?.wait().await?;
        println!("pose {:?}", dobot.get_pose().await?);

        dobot.move_to(200.0, 200.0, 0.0, 0.0).await?.wait().await?;
        println!("pose {:?}", dobot.get_pose().await?);

        dobot.move_to(200.0, 100.0, 0.0, 0.0).await?.wait().await?;
        println!("pose {:?}", dobot.get_pose().await?);

        dobot.move_to(100.0, 100.0, 0.0, 0.0).await?.wait().await?;
        println!("pose {:?}", dobot.get_pose().await?);

        DobotResult::Ok(())
    })?;
    Ok(())
}
