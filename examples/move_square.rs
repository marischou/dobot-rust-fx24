use dobot::Dobot;
use failure::Fallible;

#[tokio::main]
async fn main() -> Fallible<()> {
    let mut dobot = Dobot::open("/dev/ttyUSB0").await?;

    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.set_home().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(70.0, 70.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(70.0, 140.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(140.0, 140.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(140.0, 70.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(70.0, 70.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.set_home().await?;
    println!("pose {:#?}", dobot.get_pose().await?);
    
    Ok(())
}
