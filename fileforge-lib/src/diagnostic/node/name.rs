pub trait DiagnosticNodeName {
    fn show_ellipsis(&self) -> bool;
    fn as_str(&self) -> &str;
}