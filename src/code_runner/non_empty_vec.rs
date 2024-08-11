#[derive(Debug)]
pub struct NonEmptyVec<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    pub fn parts(self) -> (T, Vec<T>) {
        (self.head, self.tail)
    }
}

pub fn from_vec<T>(mut vec: Vec<T>) -> Option<NonEmptyVec<T>> {
    if vec.is_empty() {
        None
    } else {
        let head = vec.remove(0);

        Some(NonEmptyVec { head, tail: vec })
    }
}
