use std::env;
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;

    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.overwrite = true;

    let paths_to_copy: Vec<&str> = vec!["./res/"];
    fs_extra::copy_items(&paths_to_copy, out_dir, &copy_options)?;

    let resources_root: PathBuf = "./res/assets".into();
    let resources_root_tmp: PathBuf = "./res/assets_tmp".into();
    let temp_path = PathBuf::from("./tmp/mc-jar-cache");

    if !resources_root.is_dir() {
        println!(
            "building assets, because {:?} does not exist yet",
            resources_root
        );
        fs_extra::dir::create_all(&resources_root_tmp, true)?;

        if !temp_path.is_dir() {
            fs_extra::dir::create_all(&temp_path, true)
                .expect("tmp dir for downloading client.jar data");
        }
        println!("download offical mc client data");
        let response = reqwest::blocking::get("https://launcher.mojang.com/v1/objects/37fd3c903861eeff3bc24b71eed48f828b5269c8/client.jar").unwrap();
        let content = io::Cursor::new(response.bytes()?);
        println!("unpacking offical mc client data");
        let mut zip = zip::read::ZipArchive::new(content)?;
        zip.extract(&temp_path)?;

        let mut copy_content_only = fs_extra::dir::CopyOptions::new();
        copy_content_only.content_only = true;
        copy_content_only.overwrite = true;

        fs_extra::dir::copy(
            temp_path.join("assets"),
            &resources_root_tmp,
            &copy_content_only,
        )?;
        std::fs::remove_dir_all(temp_path)?;

        fs_extra::dir::move_dir(&resources_root_tmp, &resources_root, &copy_content_only)?;
    }

    println!("copy shader source code");
    fs_extra::dir::copy("./res/wgpu_mc", &resources_root, &copy_options)?;

    Ok(())
}
