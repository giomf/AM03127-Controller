use am03127_client::config::Panel;
use anyhow::{Context, Result};

pub fn run(panel: &Panel) -> Result<()> {
    let url = format!("http://{}", panel.address);

    println!("Opening {} ...", url);

    std::process::Command::new("xdg-open")
        .arg(&url)
        .status()
        .with_context(|| format!("failed to open '{url}'"))?;

    Ok(())
}
