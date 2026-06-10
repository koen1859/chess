// Either empty or occupied
#[derive(Debug)]
pub enum Square {
    Empty,
    Occupied(usize),
}
