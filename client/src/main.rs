
use anyhow::Result;
use std::path::Path;
use log::{error, info};
use server::launch_server;


mod fps;
mod input;
mod gui;
mod settings;
mod singleplayer;
mod ui;
mod render;
mod window;
mod world;
mod texture;
mod mainmenu;

fn main() -> Result<()>{
    env_logger::init();

    info!("Starting up..");
    let config_folder = Path::new("config");
    let config_file = config_folder.join("config/settings.toml");
    let settings = settings::load_settings(&config_folder, &config_file)?;
    info!("Loaded settings: {:?}", settings);

    let (client, server) = common::network::dummy::new();

    std::thread::spawn(move||{
        if let Err(e) = launch_server(Box::new(server)) {
            error!(
                "An error occurred while running the server. Cause: {}",
                e
            );
        }
    });
    window::open_window(
        settings,
        Box::new(singleplayer::SinglePlayer::new_factory(Box::new(client))),
    )
}