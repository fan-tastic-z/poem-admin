use nutype::nutype;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageFilter {
    page_no: PageNo,
    page_size: PageSize,
}

impl PageFilter {
    pub fn new(page_no: PageNo, page_size: PageSize) -> Self {
        Self { page_no, page_size }
    }

    pub fn page_no(&self) -> &PageNo {
        &self.page_no
    }

    pub fn page_size(&self) -> &PageSize {
        &self.page_size
    }
}

#[nutype(
    validate(greater_or_equal = 1),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct PageNo(i32);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 200),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct PageSize(i32);
