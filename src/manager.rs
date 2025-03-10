/*
    TODO: Make platform compatable
    TODO: Make auto updater + option to set version
*/

use crate::utils::{
    appdata::get_cache_dir,
    downloader::{download_chrome, download_chromedriver},
    functions::{get_latest_chrome_package, get_platform},
};

use crate::{
    loglevel::LogLevel,
    structs::{
        chrome::{ChromeDownload, DriverDownload},
        packages::ChromePackage,
    },
};

use std::{
    path::PathBuf,
    process::{self, Command},
};

use anyhow::{Context, Ok};
use thirtyfour::{ChromeCapabilities, ChromiumLikeCapabilities};

const CHROME_DOWNLOADS_URL: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";

pub struct Handler {
    client: reqwest::Client,
    platform: String,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            platform: get_platform(),
        }
    }
}

impl Handler {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_default_paths(&self) -> (PathBuf, PathBuf) {
        let chrome_path = get_cache_dir().join(format!("chrome-{}", self.platform));
        let driver_path = get_cache_dir().join(format!("chromedriver-{}", self.platform));

        (chrome_path, driver_path)
    }

    fn package_downloaded(&self) -> bool {
        let (chrome_path, driver_path) = self.get_default_paths();

        if chrome_path.exists() && driver_path.exists() {
            return true;
        }

        false
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
        get_latest_chrome_package(packages).context("Could not get the latest version of chrome")
    }

    async fn download_files(&self) -> anyhow::Result<(PathBuf, PathBuf)> {
        let chrome_packages = self.get_packages().await?;
        let selected_package = self.get_selected_package(&chrome_packages).await?;

        // TODO: Make platform configurable
        let chrome_download: &ChromeDownload = selected_package
            .get_chrome_download(&self.platform)
            .expect("Chrome download not found");

        let chromedriver_download: &DriverDownload = selected_package
            .get_chromedriver_download(&self.platform)
            .expect("Chromedriver download not found");

        // Download Chrome and Chromedriver

        let version = &selected_package.version;
        println!(
            "Installing chrome version {}.{}.{}.{}...\n",
            version.major, version.minor, version.patch, version.build
        );

        download_chrome(&self.client, chrome_download).await?;
        println!();
        download_chromedriver(&self.client, chromedriver_download).await?;

        let chrome_path = chrome_download.to_folder_path();
        let driver_path = chromedriver_download.to_folder_path();

        println!("Chrome path: {:?}", chrome_path);
        println!("Chromedriver path: {:?}", driver_path);

        Ok((chrome_path, driver_path))
    }

    // Return chrome.exe and chromedriver.exe if on windows, otherwise return chrome and chromedriver
    fn get_file_names(&self) -> (String, String) {
        let chrome_exe: String;
        let chromedriver_exe: String;

        if cfg!(target_os = "windows") {
            chrome_exe = "chrome.exe".to_string();
            chromedriver_exe = "chromedriver.exe".to_string();
        } else if cfg!(target_os = "macos") {
            chrome_exe = "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing".to_string();
            chromedriver_exe = "chromedriver".to_string();
        } else {
            chrome_exe = "chrome".to_string();
            chromedriver_exe = "chromedriver".to_string();
        }

        (chrome_exe, chromedriver_exe)
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

        let (chrome_exe_name, chromedriver_exe_name) = self.get_file_names();

        if !self.package_downloaded() {
            let (chrome_path, driver_path) = self.download_files().await?;

            chrome_exe = chrome_path.join(chrome_exe_name);
            chromedriver_exe = driver_path.join(chromedriver_exe_name);
        } else {
            let (default_chrome_path, default_driver_path) = self.get_default_paths();

            chrome_exe = default_chrome_path.join(chrome_exe_name);
            chromedriver_exe = default_driver_path.join(chromedriver_exe_name);
        }

        capabilities.set_binary(chrome_exe.to_str().unwrap())?;

        let chromedriver_exe = chromedriver_exe.to_str().unwrap();

        let mut command = Command::new(chromedriver_exe);
        let mut command = command
            .arg(format!("--port={}", port))
            .arg(format!("--log-level={loglevel}"));

        if loglevel == LogLevel::Off {
            // command = command.creation_flags(0x08000000);
            command = self.apply_creation_flags(command);
        }

        Ok(command.spawn()?)
    }

    #[cfg(target_os = "windows")]
    fn apply_creation_flags<'a>(&self, command: &'a mut Command) -> &'a mut Command {
        use std::os::windows::process::CommandExt;

        command.creation_flags(0x08000000)
    }

    #[cfg(not(target_os = "windows"))]
    fn apply_creation_flags<'a>(&self, command: &'a mut Command) -> &'a mut Command {
        command
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::{loglevel::LogLevel, manager::Handler};
    use thirtyfour::prelude::*;

    #[tokio::test]
    async fn test_launch_chromedriver() -> anyhow::Result<()> {
        let mut caps = DesiredCapabilities::chrome();
        //caps.set_headless()?;

        // Launch chromedriver on port 3000
        let mut chromedriver = Handler::new()
            .launch_chromedriver(&mut caps, "3000", LogLevel::Off)
            .await?;

        println!("Launched Chromedriver");

        let driver = WebDriver::new("http://localhost:3000", caps).await?;
        driver.goto("https://www.gimkit.com/join").await?;

        thread::sleep(Duration::from_secs(10));

        chromedriver.kill()?;
        Ok(())
    }
}
