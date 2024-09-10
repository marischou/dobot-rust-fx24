# This is a fork of dobot-rust!
This fork fixes some slight dependency and deprecation issues. Particularly `tokio_serial`, `array::FixedSizeArray` and `feature(fixed_size_array)`. fx24 stands for fix 2024.


### Usage (for fx24)
1. Add the following to your Cargo.toml.
```toml
dobot-fx24 = { path = "https://github.com/marischou/dobot-rust-fx24.git" }
failure = "0.1.8"
tokio = { version = "1.40.0", features = ["full"] }
```
2. Below is a simple example to get things started. (Working on a version without external error tools.)
```rust
use failure::Fallible;
use dobot_fx24::Dobot;

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
```

---
Below is the original github page.

---

# dobot-rust

Drive Dobot robot arms in Rust with high-level and asynchronous API.

## Usage

To include this crate in your project, add this line to your `Cargo.toml`.

```toml
dobot = "^0.1.1"
```

## Example

```rust
use dobot::Dobot;
use failure::Fallible;

#[tokio::main]
async fn main() -> Fallible<()> {
    let mut dobot = Dobot::open("/dev/ttyUSB0").await?;

    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(100.0, 100.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(100.0, 200.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(200.0, 200.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(200.0, 100.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    dobot.move_to(100.0, 100.0, 0.0, 0.0).await?.wait().await?;
    println!("pose {:#?}", dobot.get_pose().await?);

    Ok(())
}
```

## License

MIT license. See [license file](LICENSE).
