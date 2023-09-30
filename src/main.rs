mod calendar;
mod config;
mod display;
mod oauth;

#[tokio::main]
async fn main() {
    let config = match config::read_config().await {
        Ok(c) => c,
        Err(e) => {
            println!("config.yamlが読み込めませんでした。{:?}", e);
            std::process::exit(1);
        }
    };
    println!("{:}\n", config);

    let token = oauth::get_access_token().await.unwrap();

    println!("\n");

    let events = calendar::get_calenders(&token.access_token, &config)
        .await
        .unwrap();

    display::display(events);
}
