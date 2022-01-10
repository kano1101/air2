use crate::history::History;
use amazon_log::{AmazonBrowser, AmazonBrowserResult, Log};
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
fn most_recently_history<'a>() -> BoxTx<'a, History> {
    use crate::schema::histories::dsl;
    use crate::schema::histories::table;
    with_conn(move |cn| table.order(dsl::purchased_at.desc()).limit(1).first(cn)).boxed()
}
fn yesterday() -> String {
    use chrono::{Duration, Local};
    (Local::today() + Duration::days(-1))
        .naive_local()
        .format("%Y-%m-%d")
        .to_string()
}
fn difference_period_range(history: &History) -> Range {
    Range::new(&history.purchased_at, &yesterday())
}
async fn most_formerly_date() -> AmazonBrowserResult<String> {
    let mut browser = wakeup_browser().await?;
    let result = browser.most_formerly_date().await?;
    browser.quit().await?;
    Ok(result)
    // Ok("2018-01-01".to_string())
}
pub async fn difference_log() -> AmazonBrowserResult<Vec<Log>> {
    use crate::utils::establish_connection;

    let tx = with_ctx(|ctx| -> Result<History, Error> { most_recently_history().run(ctx) });
    let cn = establish_connection();

    let most_formerly_date = &most_formerly_date().await?;

    assert_eq!(most_formerly_date, "2018-01-01");

    let diff_range = match transaction_diesel_mysql::run(&cn, tx) {
        Ok(history) => difference_period_range(&history),
        Err(_) => Range::new(most_formerly_date, &yesterday()),
    };

    let mut browser = wakeup_browser().await?;
    let logs = browser.extract(&diff_range).await?;
    browser.quit().await?;

    Ok(logs)
}
