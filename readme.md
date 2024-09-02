# Chromedriver Manager

Automatically download Chromedriver for thirtyfour instances

## Crates.io

<https://crates.io/crates/chromedriver-manager>

## Code Example

```rust
use thirtyfour::prelude::*;

// Require the Handler
use chromedriver_manager::{manager::Handler, loglevel::LogLevel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create Chrome capabilities
    let mut caps = DesiredCapabilities::chrome(); 

    // Launch chromedriver on port 9515 
    let mut chromedriver = Handler::new()
        .launch_chromedriver(&mut caps, "9515", LogLevel::Warning)
        .await?;

    // Connect to chrome on the same port
    let driver = WebDriver::new("http://localhost:9515", caps).await?; 

    // Close the proccess after tasks are finished
    chromedriver.kill()?;

    Ok(())
}
```

## Running on Windows

No additional setup is required

## Running on Debian

Ensure dependencies found [here](https://github.com/puppeteer/puppeteer/blob/main/docs/troubleshooting.md#chrome-doesnt-launch-on-linux) are installed

```bash
sudo apt install ca-certificates fonts-liberation libasound2 libatk-bridge2.0-0 libatk1.0-0 libc6 libcairo2 libcups2 libdbus-1-3 libexpat1 libfontconfig1 libgbm1 libgcc1 libglib2.0-0 libgtk-3-0 libnspr4 libnss3 libpango-1.0-0 libpangocairo-1.0-0 libstdc++6 libx11-6 libx11-xcb1 libxcb1 libxcomposite1 libxcursor1 libxdamage1 libxext6 libxfixes3 libxi6 libxrandr2 libxrender1 libxss1 libxtst6 lsb-release wget xdg-utils
```

## Other Machines

You can find the chromium dependency list [here](https://source.chromium.org/chromium/chromium/src/+/main:chrome/installer/linux/debian/dist_package_versions.json)
