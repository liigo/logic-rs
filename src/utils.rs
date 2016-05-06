/// 以参数sep为中心将source文本分割为左右两边，左右均不包含sep
/// 如果没找到sep，视左边为空文本，右边为source
pub fn split_lr<'a>(source: &'a str, sep: &str) -> (&'a str, &'a str) {
    if let Some(index1) = source.find(sep) {
        let index2 = index1 + sep.len();
        (&source[0..index1], &source[index2..])
    } else {
        ("", source)
    }
}

#[cfg(test)]
mod tests {
    use utils::split_lr;

    #[test]
    fn test_split_lr() {
        let (l,r) = split_lr("var:name", ":");
        assert!(l == "var" && r == "name");
        let (l,r) = split_lr(":name", ":");
        assert!(l == "" && r == "name");
        let (l,r) = split_lr("a===b", "===");
        assert!(l == "a" && r == "b");
        let (l,r) = split_lr("var---", "---");
        assert!(l == "var" && r == "");
        let (l,r) = split_lr("var:name", "??");
        assert!(l == "" && r == "var:name");
        let (l,r) = split_lr("str : text", ":");
        assert!(l == "str " && r == " text");
    }
}
