use regex::Regex;
use std::sync::OnceLock;

/// HTML/XSS 过滤器
/// 用于过滤用户输入中的危险 HTML 标签和脚本
pub struct HtmlFilter;

impl HtmlFilter {
    /// 过滤 HTML 标签和脚本
    ///
    /// # Arguments
    /// * `input` - 输入字符串
    ///
    /// # Returns
    /// 过滤后的字符串
    pub fn sanitize(input: &str) -> String {
        static SCRIPT_REGEX: OnceLock<Regex> = OnceLock::new();
        static TAG_REGEX: OnceLock<Regex> = OnceLock::new();
        static EVENT_REGEX: OnceLock<Regex> = OnceLock::new();

        let script_regex =
            SCRIPT_REGEX.get_or_init(|| Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap());

        let tag_regex = TAG_REGEX.get_or_init(|| Regex::new(r"<[^>]*>").unwrap());

        let event_regex = EVENT_REGEX
            .get_or_init(|| Regex::new(r#"(?i)\s*(on\w+)\s*=\s*['"][^'"]*['"]"#).unwrap());

        // 1. 移除 script 标签及其内容
        let mut result = script_regex.replace_all(input, "").to_string();

        // 2. 移除所有 HTML 标签
        result = tag_regex.replace_all(&result, "").to_string();

        // 3. 移除事件处理器
        result = event_regex.replace_all(&result, "").to_string();

        // 4. 解码 HTML 实体
        result = decode_html_entities(&result);

        // 5. 再次移除可能通过实体解码产生的标签（二次过滤）
        result = tag_regex.replace_all(&result, "").to_string();

        // 6. 移除 javascript: 协议
        result = remove_javascript_protocol(&result);

        result
    }

    /// 检查输入是否包含 XSS 攻击特征
    pub fn contains_xss(input: &str) -> bool {
        let patterns = [
            r"(?i)<script",
            r"(?i)javascript:",
            r"(?i)on\w+\s*=",
            r"(?i)<iframe",
            r"(?i)<object",
            r"(?i)<embed",
            r"(?i)<svg.*onload",
            r"(?i)<img.*onerror",
            r"(?i)expression\s*\(",
            r"(?i)eval\s*\(",
            r"(?i)alert\s*\(",
            r"(?i)document\.cookie",
            r"(?i)document\.write",
        ];

        static REGEXES: OnceLock<Vec<Regex>> = OnceLock::new();
        let regexes =
            REGEXES.get_or_init(|| patterns.iter().map(|&p| Regex::new(p).unwrap()).collect());

        regexes.iter().any(|re| re.is_match(input))
    }

    /// 验证输入是否安全
    pub fn is_safe(input: &str) -> bool {
        !Self::contains_xss(input)
    }
}

/// 解码 HTML 实体
fn decode_html_entities(input: &str) -> String {
    let mut result = input.to_string();

    // 常见的 HTML 实体
    let entities = [
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&amp;", "&"),
        ("&quot;", "\""),
        ("&#39;", "'"),
        ("&apos;", "'"),
        ("&nbsp;", " "),
    ];

    for (entity, char) in entities.iter() {
        result = result.replace(entity, char);
    }

    // 处理数字实体
    static NUM_ENTITY_REGEX: OnceLock<Regex> = OnceLock::new();
    let num_entity_regex =
        NUM_ENTITY_REGEX.get_or_init(|| Regex::new(r"&#(x[0-9a-fA-F]+|\d+);").unwrap());

    result = num_entity_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let num_str = &caps[1];
            let code = if let Some(stripped) = num_str.strip_prefix('x') {
                u32::from_str_radix(stripped, 16).ok()
            } else {
                num_str.parse::<u32>().ok()
            };

            code.and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default()
        })
        .to_string();

    result
}

/// 移除 javascript: 协议
fn remove_javascript_protocol(input: &str) -> String {
    static JS_PROTOCOL_REGEX: OnceLock<Regex> = OnceLock::new();
    let js_protocol_regex =
        JS_PROTOCOL_REGEX.get_or_init(|| Regex::new(r"(?i)javascript\s*:").unwrap());

    js_protocol_regex.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_tag_removal() {
        let input = "<script>alert('XSS')</script>Hello";
        assert_eq!(HtmlFilter::sanitize(input), "Hello");
    }

    #[test]
    fn test_event_handler_removal() {
        let input = r#"<img src="x" onerror="alert('XSS')">"#;
        assert_eq!(HtmlFilter::sanitize(input), "");
    }

    #[test]
    fn test_javascript_protocol_removal() {
        let input = r#"<a href="javascript:alert('XSS')">Click</a>"#;
        assert_eq!(HtmlFilter::sanitize(input), "Click");
    }

    #[test]
    fn test_xss_detection() {
        assert!(HtmlFilter::contains_xss("<script>alert('XSS')</script>"));
        assert!(HtmlFilter::contains_xss("javascript:alert('XSS')"));
        assert!(HtmlFilter::contains_xss("<img onerror='alert()'>"));
        assert!(!HtmlFilter::contains_xss("Hello, World!"));
    }

    #[test]
    fn test_is_safe() {
        assert!(!HtmlFilter::is_safe("<script>alert('XSS')</script>"));
        assert!(HtmlFilter::is_safe("Normal text"));
    }

    #[test]
    fn test_html_entity_decoding() {
        // HTML 实体编码的 script 标签应该被完全移除
        let input = "&lt;script&gt;alert('XSS')&lt;/script&gt;";
        let result = HtmlFilter::sanitize(input);
        // script 标签被移除，只留下内容
        assert_eq!(result, "alert('XSS')");

        // 普通 HTML 实体应该被正确解码
        let input2 = "Hello &amp; World";
        assert_eq!(HtmlFilter::sanitize(input2), "Hello & World");
    }

    #[test]
    fn test_complex_xss() {
        let input = r#"<svg onload="alert('XSS')"><img src=x onerror=alert()></svg>"#;
        assert!(!HtmlFilter::is_safe(input));
    }
}
