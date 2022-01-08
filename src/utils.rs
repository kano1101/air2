use diesel::MysqlConnection;

pub fn establish_connection() -> MysqlConnection {
    use diesel::prelude::*;
    use dotenv::dotenv;
    use std::env;

    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn widths_as_ascii(subject: &str) -> Vec<u32> {
    subject
        .chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .collect()
}

pub fn round_off(subject: &str, len: u32) -> String {
    if len == 0 {
        return "".to_string();
    }
    let mut result = vec![];
    let mut counts = vec![];
    let widths = widths_as_ascii(subject);
    for (i, ch) in subject.chars().enumerate() {
        let next_char_width = widths.get(i).unwrap();
        let sum = counts.iter().fold(0, |sum, i| sum + i);
        if len - sum == 0 {
            break;
        } else if len - sum == 1 && next_char_width == &2u32 {
            result.push(" ".to_string());
            break;
        } else {
            result.push(ch.to_string());
            counts.push(*next_char_width);
        }
    }
    result.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_off_test() {
        assert_eq!(round_off("aあ", 1), "a");
        assert_eq!(round_off("aあ", 2), "a ");
        assert_eq!(round_off("あa", 1), " ");
        assert_eq!(round_off("あa", 2), "あ");
        assert_eq!(widths_as_ascii("あaあa"), [2, 1, 2, 1]);
        assert_eq!(widths_as_ascii("aあaあ"), [1, 2, 1, 2]);
    }
}
