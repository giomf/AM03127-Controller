use crate::config::Config;

pub async fn run(config: &Config) {
    let client = reqwest::Client::new();
    let mut set = tokio::task::JoinSet::new();

    for panel in &config.panels {
        let client = client.clone();
        let name = panel.name.clone();
        let url = format!("http://{}", panel.address);
        set.spawn(async move {
            let reachable = client.get(&url).send().await.is_ok();
            (name, reachable)
        });
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok((name, true)) => println!("{name}: online"),
            Ok((name, false)) => println!("{name}: offline"),
            Err(e) => eprintln!("task error: {e}"),
        }
    }
}
