use std::env;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;

    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.overwrite = true;

    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("../res/");
    fs_extra::copy_items(&paths_to_copy, out_dir, &copy_options)?;
    let resources_root: std::path::PathBuf = "../res/assets".into();
    if !resources_root.is_dir() {
        let path = std::path::PathBuf::from("/tmp/mc-jar-cache");

        if !path.is_dir() {
            std::fs::create_dir("/tmp/mc-jar-cache")?;
        }

        let response = reqwest::blocking::get("https://launcher.mojang.com/v1/objects/37fd3c903861eeff3bc24b71eed48f828b5269c8/client.jar").unwrap();
        let content = io::Cursor::new(response.bytes()?);
        let mut zip = zip::read::ZipArchive::new(content)?;
        zip.extract(&path)?;
        fs_extra::dir::copy(path.join("assets"), "../res/", &copy_options)?;

        std::fs::remove_dir_all(path)?;
    }

    Ok(())
}
