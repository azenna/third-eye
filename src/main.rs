use third_eye::{
    app::App,
    config::Config,
};

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Config parsing
    let toml_string =
        std::fs::read_to_string("third_eye.toml").context("Couldn't open 'third_eye.toml'")?;
    let config: Config = toml::from_str(&toml_string).context("Couldn't parse config")?;

    let mut app = App::new(config);
    app.run().await?;
    Ok(())
}
