pub trait StringUtils {
    fn flanked_by(&self, left: &str, right: &str) -> String;
}

impl StringUtils for &str {
    fn flanked_by(&self, left: &str, right: &str) -> String {
        format!("{}{}{}", left, self, right)
    }
}

impl StringUtils for String {
    fn flanked_by(&self, left: &str, right: &str) -> String {
        format!("{}{}{}", left, self, right)
    }
}
