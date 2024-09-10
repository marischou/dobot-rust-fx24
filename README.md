# This is a fork of dobot-rust!
This fork fixes some slight dependency and deprecation issues. Particularly `tokio_serial`, `array::FixedSizeArray` and `feature(fixed_size_array)`. Do not use this yet as I have not changed any version number or naming scheme. Should be fine to use it as a local path though. fx24 stands for fix 2024.


### Usage (fx24)
1. Add the following to your Cargo.toml.
```toml
dobot = { path = "https://github.com/marischou/dobot-rust-fx24.git" }
```
2. Below is a simple example to get things started. (Working a version without external error tools.)
```rust
todo!();

```

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
