use crate::log::Log;
use amazon_log::{AmazonBrowser, AmazonBrowserResult};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::MysqlConnection;
use range::Range;
use transaction::prelude::*;
use transaction_diesel_mysql::{with_conn, DieselContext};

type Ctx<'a> = DieselContext<'a, MysqlConnection>;
type BoxTx<'a, T> = Box<dyn Transaction<Ctx = Ctx<'a>, Item = T, Err = Error> + 'a>;

async fn wakeup_browser() -> AmazonBrowserResult<AmazonBrowser> {
    use dotenv::dotenv;
    use std::env;
    dotenv().ok();
    let email = env::var("AMAZON_EMAIL").expect("AMAZON_EMAIL must be set");
    let pass = env::var("AMAZON_PASSWORD").expect("AMAZON_PASSWORD must be set");
    let browser = AmazonBrowser::new(&email, &pass, "air2_release").await?;
    Ok(browser)
}
pub fn most_recently_log<'a>() -> BoxTx<'a, Log> {
    use crate::schema::logs::dsl;
    use crate::schema::logs::table;
    with_conn(move |cn| table.order(dsl::purchased_at.desc()).limit(1).first(cn)).boxed()
}
pub fn next_day(date_str: String) -> String {
    use chrono::{Duration, NaiveDate};
    (NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap() + Duration::days(1))
        .format("%Y-%m-%d")
        .to_string()
}
pub fn yesterday() -> String {
    use chrono::{Duration, Local};
    (Local::today() + Duration::days(-1))
        .naive_local()
        .format("%Y-%m-%d")
        .to_string()
}
pub async fn most_formerly_date() -> AmazonBrowserResult<String> {
    let mut browser = wakeup_browser().await?;
    let result = browser.most_formerly_date().await?;
    browser.quit().await?;
    Ok(result)
}
pub async fn difference_log(diff_range: Range) -> AmazonBrowserResult<Vec<amazon_log::Log>> {
    let mut browser = wakeup_browser().await?;
    let logs = browser.extract(&diff_range).await?;
    browser.quit().await?;

    Ok(logs)
}
