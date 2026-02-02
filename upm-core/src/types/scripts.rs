pub struct Scripts {
    pub pre_install: Option<Script>,
    pub post_install: Option<Script>,
    pub pre_remove: Option<Script>,
    pub post_remove: Option<Script>,
}

impl Scripts {
    pub fn new() -> Self;
    pub fn has_any(&self) -> bool;
}

pub struct Script {
    pub content: String,
    pub interpreter: String,
    pub timeout: Option<Duration>,
}

impl Script {
    pub fn new(content: String) -> Self;
    pub fn bash(content: String) -> Self;
    pub fn with_timeout(mut self, timeout: Duration) -> Self;
}
