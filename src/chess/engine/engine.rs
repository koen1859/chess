use std::collections::HashMap;

#[derive(Clone)]
pub struct Engine {
    pub tt: HashMap<u64, (i32, u8)>,
}
impl Engine {
    pub fn new() -> Self {
        Engine {
            tt: HashMap::with_capacity(100_000),
        }
    }
}
