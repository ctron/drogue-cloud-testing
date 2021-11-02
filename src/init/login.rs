use crate::init::config::Config;
use crate::init::web::WebDriver;
use fantoccini::error::CmdError;
use fantoccini::Locator;
use std::time::{Duration, Instant};

/// ensure that the web driver is logged in to the console
pub async fn login(web: &mut WebDriver, config: &Config) -> anyhow::Result<()> {
    let url = config.console().await?;

    // go to the main page

    web.goto(&url).await?;

    // find either the user dropdown (logged in) or the login button (not logged in)

    let end = Instant::now() + Duration::from_secs(30);
    let login_button = loop {
        // find user dropdown

        match web.find(Locator::Id("user-dropdown")).await {
            Ok(_) => {
                // we are already logged in, return
                return Ok(());
            }
            Err(CmdError::NoSuchElement(_)) => {}
            Err(err) => return Err(err.into()),
        }

        // check login button

        match web
            .find(Locator::Css(".pf-c-login button.pf-c-button.pf-m-primary"))
            .await
        {
            Ok(login_button) => {
                break login_button;
            }
            Err(CmdError::NoSuchElement(_)) => {}
            Err(err) => return Err(err.into()),
        }

        // timeout?

        if end > Instant::now() {
            anyhow::bail!("Found neither user dropdown nor login button");
        }

        // wait a bit longer

        tokio::time::sleep(Duration::from_millis(250)).await;
    };

    login_button.click().await?;

    let mut form = web.form(Locator::Id("kc-form-login")).await?;
    form.set_by_name("username", &config.user)
        .await?
        .set_by_name("password", &config.password)
        .await?
        .submit()
        .await?;

    web.wait().for_element(Locator::Id("user-dropdown")).await?;

    Ok(())
}
