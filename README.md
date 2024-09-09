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
