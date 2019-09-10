use alloc::{string::String, vec::Vec};
use core::ops::{self, Add, AddAssign, Deref, DerefMut, Index, IndexMut};
use core::str::{self, FromStr};
use core::{fmt, ptr};

pub use core::convert::TryFrom;

use crate::Result;

pub type Tryte = u8;

/// A char array holding all acceptable characters in the tryte
/// alphabet. Used because strings can't be cheaply indexed in rust.
pub const TRYTE_ALPHABET: [u8; 27] = [
    57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87,
    88, 89, 90,
];

#[derive(Clone, Debug, PartialEq)]
pub struct Trytes {
    vec: Vec<Tryte>,
}

impl Trytes {
    /// Creates a new empty `Trytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let t = Trytes::new();
    /// ```
    #[inline]
    pub fn new() -> Trytes {
        Trytes { vec: Vec::new() }
    }

    /// Creates a new empty `Trytes` with a particular capacity.
    ///
    /// If the given capacity is `0`, no allocation will occur, and this method
    /// is identical to the [`new`] method.
    ///
    /// [`new`]: #method.new
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::with_capacity(10);
    ///
    /// // The Trytes contains no chars, even though it has capacity for more
    /// assert_eq!(t.len(), 0);
    ///
    /// // These are all done without reallocating...
    /// let cap = t.capacity();
    /// for _ in 0..10 {
    ///     t.push('a');
    /// }
    ///
    /// assert_eq!(t.capacity(), cap);
    ///
    /// // ...but this may make the vector reallocate
    /// t.push('a');
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Trytes {
        Trytes {
            vec: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn from_utf8(vec: Vec<u8>) -> Result<Trytes> {
        Self::all_tryte_alphabete(vec.iter().copied())?;
        Ok(Trytes { vec })
    }

    pub unsafe fn from_utf8_unchecked(bytes: Vec<u8>) -> Trytes {
        Trytes { vec: bytes }
    }

    #[inline]
    pub unsafe fn from_raw_parts(buf: *mut u8, length: usize, capacity: usize) -> Trytes {
        Trytes {
            vec: Vec::from_raw_parts(buf, length, capacity),
        }
    }

    /// Converts a `Trytes` into a byte vector.
    ///
    /// This consumes the `Trytes`, so we do not need to copy its contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let t = Trytes::try_from("ABCD").unwrap();
    /// let bytes = t.into_bytes();
    ///
    /// assert_eq!(&[65, 66, 67, 68][..], &bytes[..]);
    /// ```
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.vec
    }

    /// Returns a byte slice of this `Trytes`'s contents.
    ///
    /// The inverse of this method is [`from_utf8`].
    ///
    /// [`from_utf8`]: #method.from_utf8
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let t = String::from("ABCD");
    ///
    /// assert_eq!(&[65, 66, 67, 68], t.as_bytes());
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.vec
    }

    /// Extracts a string slice containing the entire `Trytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let t = Trytes::try_from("FOO").unwrap();
    ///
    /// assert_eq!("FOO", t.as_str());
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.vec).unwrap()
    }

    /// Converts a `Trytes` into a mutable string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut s = Trytes::try_from("FOO").unwrap();
    /// let s_mut_str = s.as_mut_str();
    ///
    /// s_mut_str.make_ascii_lowercase();
    ///
    /// assert_eq!("foo", s_mut_str);
    /// ```
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        str::from_utf8_mut(&mut self.vec).unwrap()
    }

    /// Appends a given string slice onto the end of this `Trytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let t = Trytes::try_from("IOTA").unwrap();
    /// let mut t2 = Trytes::try_from("IO").unwrap();
    ///
    /// t2.push_str("TA").unwrap();
    ///
    /// assert_eq!(t, t2);
    /// ```
    #[inline]
    pub fn push_str(&mut self, string: &str) -> Result<()> {
        let bytes = string.as_bytes();
        Self::all_tryte_alphabete(bytes.iter().copied())?;
        self.vec.extend_from_slice(bytes);
        Ok(())
    }

    /// Returns this `Trytes`' capacity, in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let t = Trytes::with_capacity(10);
    ///
    /// assert!(t.capacity() >= 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.vec.reserve_exact(additional)
    }

    /// Shrinks the capacity of this `Trytes` to match its length.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("FOO").unwrap();
    ///
    /// t.reserve(100);
    /// assert!(t.capacity() >= 100);
    ///
    /// t.shrink_to_fit();
    /// assert_eq!(3, t.capacity());
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.vec.shrink_to_fit()
    }

    /// Appends the given [`char`] to the end of this `Trytes`.
    ///
    /// [`char`]: ../../std/primitive.char.html
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("I").unwrap();
    ///
    /// t.push('O');
    /// t.push('T');
    /// t.push('A');
    ///
    /// assert_eq!("IOTA", t.as_str());
    /// ```
    #[inline]
    pub fn push(&mut self, ch: char) -> Result<()> {
        match Self::is_tryte_alphabete(ch as u8) {
            true => self.vec.push(ch as u8),
            false => return Err(format_err!("Invalid tryte alphabete")),
        }
        Ok(())
    }

    /// Shorten this `Trytes` to the specified length.
    ///
    /// If `new_len` is greater than the trytes's current length, this has no
    /// effect.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the trytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("IOTA").unwrap();
    ///
    /// t.truncate(2);
    ///
    /// assert_eq!("IO", t.as_str());
    /// ```
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            self.vec.truncate(new_len)
        }
    }

    /// Removes the last character from the string buffer and returns it.
    ///
    /// Returns [`None`] if this `String` is empty.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("ABC").unwrap();
    ///
    /// assert_eq!(t.pop(), Some('C'));
    /// assert_eq!(t.pop(), Some('B'));
    /// assert_eq!(t.pop(), Some('A'));
    /// assert_eq!(t.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.vec.pop()?;
        Some(ch as char)
    }

    /// Removes a [`char`] from this `Trytes` at a byte position and returns it.
    ///
    /// This is an `O(n)` operation, as it requires copying every element in the
    /// buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than or equal to the `Trytes`'s length.
    ///
    /// [`char`]: ../../std/primitive.char.html
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("ABC").unwrap();
    ///
    /// assert_eq!(t.remove(0), 'A');
    /// assert_eq!(t.remove(1), 'C');
    /// assert_eq!(t.remove(0), 'B');
    /// ```
    pub fn remove(&mut self, idx: usize) -> char {
        let ch = self[idx];

        let next = idx + 1;
        let len = self.len();
        unsafe {
            ptr::copy(
                self.vec.as_ptr().add(next),
                self.vec.as_mut_ptr().add(idx),
                len - next,
            );
            self.vec.set_len(len - (next - idx));
        }
        ch as char
    }

    /// Inserts a character into this `Trytes` at a byte position.
    ///
    /// This is an `O(n)` operation as it requires copying every element in the
    /// buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the `String`'s length.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::with_capacity(3);
    ///
    /// t.insert(0, 'B');
    /// t.insert(1, 'E');
    /// t.insert(2, 'E');
    ///
    /// assert_eq!("BEE", t.as_str());
    /// ```
    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) -> Result<()> {
        match Self::is_tryte_alphabete(ch as u8) {
            true => unsafe {
                self.insert_bytes(idx, &[ch as u8]);
            },
            false => return Err(format_err!("Invalid tryte alphabete")),
        }
        Ok(())
    }

    /// Inserts a string slice into this `Trytes` at a byte position.
    ///
    /// This is an `O(n)` operation as it requires copying every element in the
    /// buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than the `String`'s length.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("BAR").unwrap();
    ///
    /// t.insert_str(0, "FOO").unwrap();
    ///
    /// assert_eq!("FOOBAR", t.as_str());
    /// ```
    #[inline]
    pub fn insert_str(&mut self, idx: usize, string: &str) -> Result<()> {
        let bytes = string.as_bytes();
        Self::all_tryte_alphabete(bytes.iter().copied())?;
        unsafe {
            self.insert_bytes(idx, bytes);
        }
        Ok(())
    }

    /// Returns a mutable reference to the contents of this `Trytes`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid tryte alphabete in UTF=8. If this constraint is
    /// violated, it may cause memory unsafety issues with future users of the
    /// `Trytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("ABC").unwrap();
    ///
    /// unsafe {
    ///     let vec = t.as_mut_vec();
    ///     assert_eq!(&[65, 66, 67][..], &vec[..]);
    ///
    ///     vec.reverse();
    /// }
    /// assert_eq!("CBA", t.as_str());
    /// ```
    pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.vec
    }

    /// Returns the length of this `Trytes`, in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("ABC").unwrap();
    ///
    /// assert_eq!(t.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns `true` if this `Trytes` has a length of zero, and `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::new();
    /// assert!(t.is_empty());
    ///
    /// t.push('9');
    /// assert!(!t.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Splits the trytes into two at the given index.
    ///
    /// Returns a newly allocated `Trytes`. `self` contains bytes `[0, at)`, and
    /// the returned `Trytes` contains bytes `[at, len)`.
    ///
    /// Note that the capacity of `self` does not change.
    ///
    /// # Panics
    ///
    /// Panics if `at` is beyond the last code point of the trytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("IOTA").unwrap();
    /// let t2 = t.split_off(2);
    /// assert_eq!(t.as_str(), "IO");
    /// assert_eq!(t2.as_str(), "TA");
    /// ```
    #[inline]
    pub fn split_off(&mut self, at: usize) -> Trytes {
        let other = self.vec.split_off(at);
        unsafe { Self::from_utf8_unchecked(other) }
    }

    /// Truncates this `Trytes`, removing all contents.
    ///
    /// While this means the `Trytes` will have a length of zero, it does not
    /// touch its capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::try_from("FOO").unwrap();
    ///
    /// t.clear();
    ///
    /// assert!(t.is_empty());
    /// assert_eq!(0, t.len());
    /// assert_eq!(3, t.capacity());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.vec.clear()
    }

    /// Encode a string literal as bytes to trytes.
    ///
    /// ```
    /// use iota_trinary::*;
    ///
    /// let mut t = Trytes::new();
    ///
    /// t.encode("Lorem");
    ///
    /// assert_eq!("VBCDFDTCAD", t.as_str());
    /// ```
    pub fn encode(&mut self, plain: &str) {
        for byte in plain.bytes() {
            let first = byte % 27;
            let second = (byte - first) / 27;
            self.vec.push(TRYTE_ALPHABET[first as usize]);
            self.vec.push(TRYTE_ALPHABET[second as usize]);
        }
    }

    /// ```
    /// use iota_trinary::*;
    ///
    /// let t = Trytes::try_from("VBCDFDTCAD").unwrap();
    ///
    /// assert_eq!("Lorem", t.decode());
    /// ```
    pub fn decode(&self) -> String {
        let mut s = String::with_capacity(self.len() / 2);
        for slice in self.chunks(2) {
            let first: u8 = TRYTE_ALPHABET.iter().position(|&x| x == slice[0]).unwrap() as u8;
            let second: u8 = TRYTE_ALPHABET.iter().position(|&x| x == slice[1]).unwrap() as u8;
            let decimal = first + second * 27;

            s.push(decimal as char);
        }
        s
    }

    // TODO: retain, drain, replace_range, into_boxed_str

    unsafe fn insert_bytes(&mut self, idx: usize, bytes: &[u8]) {
        let len = self.len();
        let amt = bytes.len();
        self.vec.reserve(amt);

        ptr::copy(
            self.vec.as_ptr().add(idx),
            self.vec.as_mut_ptr().add(idx + amt),
            len - idx,
        );
        ptr::copy(bytes.as_ptr(), self.vec.as_mut_ptr().add(idx), amt);
        self.vec.set_len(len + amt);
    }

    fn all_tryte_alphabete<I>(vals: I) -> Result<()>
    where
        I: Iterator<Item = u8>,
    {
        let mut v = vals;
        ensure!(v.all(Self::is_tryte_alphabete), "Invalid trytes alphabete.");
        Ok(())
    }

    fn is_tryte_alphabete(t: u8) -> bool {
        match t {
            57 | 65..=90 => true,
            _ => false,
        }
    }
}

impl Default for Trytes {
    /// Creates an empty `Trytes`.
    #[inline]
    fn default() -> Trytes {
        Trytes::new()
    }
}

impl fmt::Display for Trytes {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Add<&Trytes> for Trytes {
    type Output = Trytes;

    #[inline]
    fn add(mut self, rhs: &Trytes) -> Self::Output {
        self.vec.extend_from_slice(rhs.as_bytes());
        self
    }
}

impl AddAssign<&Trytes> for Trytes {
    #[inline]
    fn add_assign(&mut self, rhs: &Trytes) {
        self.vec.extend_from_slice(rhs.as_bytes());
    }
}

impl ops::Index<usize> for Trytes {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self[..][index]
    }
}

impl ops::Index<ops::Range<usize>> for Trytes {
    type Output = [u8];

    #[inline]
    fn index(&self, index: ops::Range<usize>) -> &Self::Output {
        &self[..][index]
    }
}

impl ops::Index<ops::RangeTo<usize>> for Trytes {
    type Output = [u8];

    #[inline]
    fn index(&self, index: ops::RangeTo<usize>) -> &Self::Output {
        &self[..][index]
    }
}

impl ops::Index<ops::RangeFrom<usize>> for Trytes {
    type Output = [u8];

    #[inline]
    fn index(&self, index: ops::RangeFrom<usize>) -> &Self::Output {
        &self[..][index]
    }
}

impl ops::Index<ops::RangeFull> for Trytes {
    type Output = [u8];

    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &Self::Output {
        &self.vec[..]
    }
}

impl ops::Index<ops::RangeInclusive<usize>> for Trytes {
    type Output = [u8];

    #[inline]
    fn index(&self, index: ops::RangeInclusive<usize>) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl ops::Index<ops::RangeToInclusive<usize>> for Trytes {
    type Output = [u8];

    #[inline]
    fn index(&self, index: ops::RangeToInclusive<usize>) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl ops::IndexMut<usize> for Trytes {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self[..][index]
    }
}

impl ops::IndexMut<ops::Range<usize>> for Trytes {
    #[inline]
    fn index_mut(&mut self, index: ops::Range<usize>) -> &mut [u8] {
        &mut self[..][index]
    }
}

impl ops::IndexMut<ops::RangeTo<usize>> for Trytes {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeTo<usize>) -> &mut [u8] {
        &mut self[..][index]
    }
}

impl ops::IndexMut<ops::RangeFrom<usize>> for Trytes {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeFrom<usize>) -> &mut [u8] {
        &mut self[..][index]
    }
}

impl ops::IndexMut<ops::RangeFull> for Trytes {
    #[inline]
    fn index_mut(&mut self, _index: ops::RangeFull) -> &mut [u8] {
        &mut self.vec[..]
    }
}

impl ops::IndexMut<ops::RangeInclusive<usize>> for Trytes {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeInclusive<usize>) -> &mut [u8] {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl ops::IndexMut<ops::RangeToInclusive<usize>> for Trytes {
    #[inline]
    fn index_mut(&mut self, index: ops::RangeToInclusive<usize>) -> &mut [u8] {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl Deref for Trytes {
    type Target = [Tryte];

    #[inline]
    fn deref(&self) -> &[Tryte] {
        &self.vec[..]
    }
}

impl DerefMut for Trytes {
    #[inline]
    fn deref_mut(&mut self) -> &mut [Tryte] {
        &mut self.vec[..]
    }
}

impl FromStr for Trytes {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Trytes> {
        Ok(Trytes::try_from(s)?)
    }
}

pub trait Trinary {
    fn to_trytes(&self) -> Result<Trytes>;
}

impl Trinary for str {
    #[inline]
    fn to_trytes(&self) -> Result<Trytes> {
        Trytes::try_from(self)
    }
}

impl Trinary for String {
    #[inline]
    fn to_trytes(&self) -> Result<Trytes> {
        Trytes::try_from(self)
    }
}

impl Trinary for Trytes {
    #[inline]
    fn to_trytes(&self) -> Result<Trytes> {
        Ok(Trytes::from(self))
    }
}

impl AsRef<str> for Trytes {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for Trytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl TryFrom<&str> for Trytes {
    type Error = failure::Error;

    #[inline]
    fn try_from(s: &str) -> Result<Trytes> {
        let bytes = s.as_bytes();
        Self::all_tryte_alphabete(bytes.iter().copied())?;
        Ok(Trytes {
            vec: bytes.to_vec(),
        })
    }
}

impl TryFrom<&String> for Trytes {
    type Error = failure::Error;

    #[inline]
    fn try_from(s: &String) -> Result<Trytes> {
        let bytes = s.as_bytes();
        Self::all_tryte_alphabete(bytes.iter().copied())?;
        Ok(Trytes {
            vec: bytes.to_vec(),
        })
    }
}

impl From<&Trytes> for Trytes {
    fn from(trytes: &Trytes) -> Trytes {
        trytes.clone()
    }
}

impl From<Trytes> for Vec<u8> {
    fn from(trytes: Trytes) -> Vec<u8> {
        trytes.into_bytes()
    }
}

impl fmt::Write for Trytes {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Err(_) = self.push_str(s) {
            return Err(fmt::Error);
        }
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        if let Err(_) = self.push(c) {
            return Err(fmt::Error);
        }
        Ok(())
    }
}
