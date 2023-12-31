use crate::utils::{
    appdata::get_cache_dir,
    downloader::{download_chrome, download_chromedriver},
    functions::get_latest_chrome_package,
    version::Version,
};
use crate::{
    structs::{
        chrome::{ChromeDownload, DriverDownload},
        packages::ChromePackage,
    },
    loglevel::LogLevel,
};

/*
    TODO: Make platform compatable
    TODO: Make auto updater + option to set version
*/

use std::{os::windows::process::CommandExt, path::PathBuf, process::{Command, self}};
use thirtyfour::ChromeCapabilities;

const CHROME_DOWNLOADS_URL: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";
const PLATFORM: &str = "win64";

pub struct Handler {
    client: reqwest::Client
}

impl Handler {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new()
        }
    }

    // TODO: Make appdata configurable
    fn get_default_paths(&self) -> (PathBuf, PathBuf) {
        let chrome_path = get_cache_dir().join("chrome-win64");
        let driver_path = get_cache_dir().join("chromedriver-win64");

        (chrome_path, driver_path)
    }

    // TODO: Make platform configurable
    fn package_downloaded(&self) -> bool {
        let (chrome_path, driver_path) = self.get_default_paths();

        if chrome_path.exists() && driver_path.exists() {
            return true;
        }

        return false;
    }

    async fn get_packages(&self) -> anyhow::Result<Vec<ChromePackage>> {
        let client = reqwest::Client::new();

        let response: serde_json::Value = client
            .get(CHROME_DOWNLOADS_URL)
            .send()
            .await?
            .text()
            .await?
            .parse()?;

        let chrome_packages: serde_json::Value = response["versions"].clone();
        let chrome_packages: Vec<ChromePackage> = serde_json::from_value(chrome_packages)?;

        Ok(chrome_packages)
    }

    // TODO: Allow users to specify version
    async fn get_selected_package(
        &self,
        packages: &Vec<ChromePackage>,
    ) -> anyhow::Result<ChromePackage> {
        let latest_package = get_latest_chrome_package(&packages).unwrap();

        Ok(latest_package)
    }

    // async fn download_files(
    //     &self,
    //     version: &Version,
    //     chrome: &ChromeDownload,
    //     driver: &DriverDownload,
    // ) -> anyhow::Result<()> {

    //     Ok(())
    // }

    async fn download_files(&self) -> anyhow::Result<(PathBuf, PathBuf)> {
        let chrome_packages = self.get_packages().await?;
        let selected_package = self.get_selected_package(&chrome_packages).await?;

        // TODO: Make platform configurable
        let chrome_download: &ChromeDownload = &selected_package
            .get_chrome_download(PLATFORM)
            .expect("Chrome download not found");

        let chromedriver_download: &DriverDownload = &selected_package
            .get_chromedriver_download(PLATFORM)
            .expect("Chromedriver download not found");

        // Download Chrome and Chromedriver

        let version = &selected_package.version;
        println!(
            "Installing chrome version {}.{}.{}.{}...\n",
            version.major, version.minor, version.patch, version.build
        );

        download_chrome(&self.client, chrome_download).await?;
        print!("\n");
        download_chromedriver(&self.client, chromedriver_download).await?;

        let chrome_path = chrome_download.to_folder_path();
        let driver_path = chromedriver_download.to_folder_path();

        return Ok((chrome_path, driver_path));
    }

    pub async fn launch_chromedriver(
        &mut self,
        capabilities: &mut ChromeCapabilities,
        port: &str,
        loglevel: LogLevel,
    ) -> Result<process::Child, anyhow::Error> {
        self.client = reqwest::Client::new();

        let chrome_exe: PathBuf;
        let chromedriver_exe: PathBuf;

        if !self.package_downloaded() {
            let (chrome_path, driver_path) = self.download_files().await?;

            chrome_exe = chrome_path.join("chrome.exe");
            chromedriver_exe = driver_path.join("chromedriver.exe");
        } else {
            let (default_chrome_path, default_driver_path) = self.get_default_paths();

            chrome_exe = default_chrome_path.join("chrome.exe");
            chromedriver_exe = default_driver_path.join("chromedriver.exe");
        }

        capabilities.set_binary(chrome_exe.to_str().unwrap())?;

        let chromedriver_exe = chromedriver_exe.to_str().unwrap();

        let mut command = Command::new(chromedriver_exe);
        let mut command = command
            .arg(format!("--port={}", port))
            .arg(format!("--log-level={}", loglevel.to_string()));

        // TODO: Make creation_flags only apply to Windows
        if loglevel == LogLevel::Off {
            command = command.creation_flags(0x08000000);
        }

        Ok(command.spawn()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{manager::Handler, loglevel::LogLevel};
    use thirtyfour::prelude::*;

    #[tokio::test]
    async fn test_launch_chromedriver() -> anyhow::Result<()> {
        let mut caps = DesiredCapabilities::chrome();
        caps.set_headless()?;

        let mut chromedriver = Handler::new()
            .launch_chromedriver(&mut caps, "9515", LogLevel::Off)
            .await?;

        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        driver.goto("https://www.gimkit.com/join").await?;

        std::thread::sleep(std::time::Duration::from_secs(10));

        chromedriver.kill()?;
        Ok(())
    }
}
