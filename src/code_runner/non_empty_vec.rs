

#[derive(Debug)]
pub struct NonEmptyVec<T> {
    head: T,
    tail: Vec<T>,
}


impl<T> NonEmptyVec<T> {
    pub fn head(&self) -> T where T: Clone {
        self.head.clone()
    }

    pub fn tail(&self) -> Vec<T> where T: Clone {
        self.tail.clone()
    }
}

pub fn from_vec<T>(mut vec: Vec<T>) -> Option<NonEmptyVec<T>> {
    if vec.is_empty() {
        None
    } else {
        let head = vec.remove(0);

        Some(NonEmptyVec{
            head: head,
            tail: vec,
        })
    }
}
