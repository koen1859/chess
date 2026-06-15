use instant::Instant;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    Alpha,
    Beta,
}

#[derive(Clone)]
pub struct Engine {
    pub tt: HashMap<u64, (i32, u8, TTFlag)>,
    pub deadline: Option<Instant>,
    pub time_up: bool,
    pub nodes: u64,
}
impl Engine {
    pub fn new() -> Self {
        Engine {
            tt: HashMap::with_capacity(100_000),
            deadline: None,
            time_up: false,
            nodes: 0,
        }
    }
}
