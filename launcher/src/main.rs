#![windows_subsystem = "windows"]

use std::{env, process::Command};

fn main() -> Result<(), std::io::Error> {
    let base = env::current_exe()?
                            .parent()
                            .unwrap()
                            .to_path_buf();

    let alacritty = base.join("alacritty/alacritty.exe");
    let alacritty_config = base.join("alacritty/alacritty.toml");
    let moody = base.join("moody/moody.exe");
    let moody_folder = base.join("moody");
    let output_folder = base.join("output");

    Command::new(&alacritty)
        .arg("--config-file")
        .arg(&alacritty_config)
        .arg("-e")
        .arg(&moody)
        .arg(&output_folder)
        .current_dir(&moody_folder)
        .spawn()?;

    Ok(())

}
