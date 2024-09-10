use failure::Fallible;
use dobot::Dobot;

#[tokio::main]
async fn main() -> Fallible<()> {
    // Change the path to your corresponding Dobot port.
    let mut dobot = Dobot::open("/dev/ttyUSB0").await?;

    print_pose(&mut dobot).await; 
    dobot.set_home().await?.wait().await?;
    print_pose(&mut dobot).await;

    dobot.move_to(100.0, 50.0, 0.0, 0.0).await?.wait().await?;
    print_pose(&mut dobot).await;
    dobot.move_to(100.0, 0.0, 0.0, 0.0).await?.wait().await?;
    print_pose(&mut dobot).await;

    Ok(())
}

async fn print_pose<'a>(internal_dobot: &'a mut Dobot) {
    let pose = internal_dobot.get_pose().await.unwrap();
    println!(
        "Pose: x: {} y: {} z: {} r: {} j1: {} j2: {} j3: {} j4: {}",
        pose.x, pose.y, pose.z, pose.r, pose.j1, pose.j2, pose.j3, pose.j4
    );
}
