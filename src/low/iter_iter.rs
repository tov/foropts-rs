use std::borrow::Borrow;
use std::ops::{Index, Range};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArgItemType {
    Short, Long, Positional,
}

pub struct ArgItem<S> {
    the_type:       ArgItemType,
    original:       S,
    extra_original: Option<S>,
    range:          Range<usize>,
    extra_range:    Range<usize>,
}

impl<S> ArgItem<S> where S: Borrow<str> {
    pub fn is_positional(&self) -> bool {
        self.the_type == ArgItemType::Positional
    }

    pub fn is_short(&self) -> bool {
        self.the_type == ArgItemType::Short
    }

    pub fn is_long(&self) -> bool {
        self.the_type == ArgItemType::Long
    }

    pub fn has_param(&self) -> bool {
        self.extra_range.start >= self.extra_range.end
    }

    pub fn get_positional(&self) -> Option<&str> {
        if self.is_positional() {
            Some(self.original.borrow().index(self.range.clone()))
        } else {
            None
        }
    }

    pub fn get_flag(&self) -> Option<&str> {
        if self.is_positional() {
            None
        } else {
            Some(self.original.borrow().index(self.range.clone()))
        }
    }

    pub fn get_param(&self) -> Option<&str> {
        if self.is_positional() || self.extra_range.start >= self.extra_range.end {
            None
        } else {
            let original = self.extra_original.as_ref().unwrap_or(&self.original);
            Some(original.borrow().index(self.extra_range.clone()))
        }
    }
}
