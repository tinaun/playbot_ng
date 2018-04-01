#![feature(slice_get_slice)]

// TODO: Come up with better type names
// TODO: Provide ARC version

use std::slice;
use std::str;
use std::fmt;
use std::ops::Deref;
use std::borrow::Borrow;
use std::slice::SliceIndex;
use std::hash::{Hash, Hasher};
use std::cmp::{PartialEq, Eq, PartialOrd, Ord, Ordering};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct SharedStr {
    _handle: Rc<String>,
    ptr: *const u8,
    len: usize,
}

unsafe impl Send for SharedStr {}
unsafe impl Sync for SharedStr {}

impl SharedStr {
    pub fn as_str(&self) -> &str {
        unsafe {
            let slice = slice::from_raw_parts(self.ptr, self.len);
            str::from_utf8_unchecked(slice)
        }
    }

    pub fn trim(&self) -> SharedStr {
        self.from_str(self.as_str().trim())
    }

    pub fn trim_left(&self) -> SharedStr {
        self.from_str(self.as_str().trim_left())
    }

    pub fn split_at(&self, mid: usize) -> (SharedStr, SharedStr) {
        let (left, right) = self.as_str().split_at(mid);
            
        let left = self.from_str(left);
        let right = self.from_str(right);

        (left, right)
    }

    pub fn split_whitespace<'a>(&'a self) -> impl Iterator<Item = SharedStr> + 'a {
        self.as_str()
            .split_whitespace()
            .map(move |s| self.from_str(s))
    }

    pub fn slice<I>(&self, range: I) -> SharedStr
    where
        I: SliceIndex<str, Output = str>,
    {
        self.from_str(range.index(self.as_str()))
    }
}

impl SharedStr {
    /// Create a `SharedStr` from a `&str`,
    /// reusing the backing buffer of `self`
    /// if possible.
    pub fn from_str(&self, s: &str) -> Self {
        // Return empty string with this identity
        if s.len() == 0 {
            return Self {
                len: 0,
                .. self.clone()
            }
        }

        if self.owns(s) {
            SharedStr {
                _handle: self._handle.clone(),
                ptr: s.as_ptr(),
                len: s.len(),
            }
        } else {
            Self::from(s)
        }
    }

    fn owns(&self, s: &str) -> bool {
        let start = self.as_str().as_ptr() as usize;
        let end = start + self.as_str().len();
        let addr = s.as_ptr() as usize;
        
        start <= addr && addr < end
    }
}

impl From<Rc<String>> for SharedStr {
    fn from(s: Rc<String>) -> SharedStr {
        SharedStr {
            ptr: s.as_str().as_ptr(),
            len: s.as_str().len(),
            _handle: s,
        }
    }
}

impl From<String> for SharedStr {
    fn from(s: String) -> SharedStr {
        Rc::new(s).into()
    }
}

impl<'a> From<&'a SharedStr> for SharedStr {
    fn from(s: &'a SharedStr) -> SharedStr {
        s.clone()
    }
}

impl<'a> From<&'a str> for SharedStr {
    fn from(s: &'a str) -> SharedStr {
        s.to_string().into()
    }
}

impl fmt::Display for SharedStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl PartialEq for SharedStr {
    fn eq(&self, other: &SharedStr) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for SharedStr {}

impl PartialOrd for SharedStr {
    fn partial_cmp(&self, other: &SharedStr) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for SharedStr {
    fn cmp(&self, other: &SharedStr) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for SharedStr {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher
    {
        self.as_str().hash(state);
    }
}

impl AsRef<str> for SharedStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for SharedStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for SharedStr {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}
