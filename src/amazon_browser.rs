use chrono::Local;
use range::Range;

pub struct AmazonBrowser {
    // todayは含めない
    range: Range,
}

impl AmazonBrowser {
    pub fn new(range: Range) -> Self {
        Self { range: range }
    }
}

pub fn wakeup_browser(range: Range) -> Option<AmazonBrowser> {
    let today = Local::today().naive_local().format("%Y-%m-%d").to_string();
    // todayがendの日の場合はアウト
    let validity = range.end_after(&today);
    if validity {
        Some(AmazonBrowser::new(range))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    #[rstest(end_offset, result,
        case(-1, true),
        case(0, false),
        case(1, false),
    )]
    fn アマゾンに指定する期間が無効な場合エラーとなる(
        end_offset: i64,
        result: bool,
    ) {
        use crate::amazon_browser::wakeup_browser;
        use chrono::{Duration, Local};
        use range::Range;
        let end_date = (Local::today() + Duration::days(end_offset))
            .naive_local()
            .format("%Y-%m-%d")
            .to_string();
        let range = Range::new(&end_date, "2021-04-01");
        let browser = wakeup_browser(range);
        // 未来日、今日（Today）が含まれているのでエラー
        assert_eq!(browser.is_some(), result);
    }
    #[test]
    fn アマゾンの購入履歴を期間を指定して取得() {
        // use range::Range;
        // let range = Range::new("2021-10-01", "2021-04-01");
        // let browser = AmazonBrowser::new(range);
        // assert_eq!(browser.valid_with_date(), true);
    }
}
