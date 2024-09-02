use crate::structs::packages::ChromePackage;
use anyhow::{self, Ok};
use indicatif::{ProgressBar, ProgressStyle};
use std::{env::consts, fs::File, io::Write};

pub fn get_latest_chrome_package(chrome_packages: &Vec<ChromePackage>) -> Option<ChromePackage> {
    let mut latest_package: Option<ChromePackage> = None;

    for package in chrome_packages {
        if latest_package.is_none() {
            latest_package = Some(package.clone());
        } else {
            let latest_package_version = &latest_package.as_ref().unwrap().version;
            let package_version = &package.version;

            if package_version > latest_package_version {
                latest_package = Some(package.clone());
            }
        }
    }

    latest_package
}

pub async fn create_progressbar(length: u64, msg: String) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
        .progress_chars("#>-")
    );
    pb.set_message(msg);

    pb
}

pub async fn write_file(
    mut file: &File,
    mut response: reqwest::Response,
    msg: String,
) -> anyhow::Result<()> {
    let file_size = response.content_length().unwrap_or(0);
    let progress_bar = create_progressbar(file_size, msg).await;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;

        let increment = chunk.len() as u64; // Convert to MB
        progress_bar.inc(increment);
    }

    file.flush()?;

    Ok(())
}

pub fn get_platform() -> String {
    let platform = match consts::OS {
        "windows" => "win64",
        "linux" => "linux64",
        "macos" => "mac-x64",
        _ => panic!("Unsupported OS"),
    };

    platform.to_string()
}
