pub struct SliceArena<T> {
    contents: Vec<T>
}

impl<T> SliceArena<T> {
    fn new() -> Self {
        SliceArena {
            contents: Vec::new()
        }
    }

    fn allocate_slice<I>(&self, iter: I) -> ASlice
        where
            I: IntoIterator<Item=T> {

        let base = self.contents.len();
        
        self.contents.extend(iter);

        let len = self.contents.len() - base;

        ASlice { base, len }
    }

    fn borrow_slice(&self, slice: ASlice) -> &[T] {
        &self.contents[slice.base..slice.base+slice.len]
    }
}

#[derive(Clone, Copy)]
pub struct ASlice {
    base: usize,
    len: usize
}
