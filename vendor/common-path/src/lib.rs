//! Calculates the common prefix for a set of paths, if a common prefix exists
//!
//! # Example
//!
//! ```rust
//! # extern crate common_path;
//! use std::path::{PathBuf, Path};
//! use common_path::common_path;
//!
//! # fn main() {
//! let baz = Path::new("/foo/bar/baz");
//! let quux = Path::new("/foo/bar/quux");
//! let prefix = common_path(baz, quux).unwrap();
//! assert_eq!(prefix, Path::new("/foo/bar").to_path_buf());
//! # }
//! ```
//!
//! Or for more than 2 paths:
//!
//! ```rust
//! # extern crate common_path;
//! use std::path::{PathBuf, Path};
//! use common_path::common_path_all;
//!
//! # fn main() {
//! let baz = Path::new("/foo/bar/baz");
//! let quux = Path::new("/foo/bar/quux");
//! let foo = Path::new("/foo/bar/foo");
//! let prefix = common_path_all(vec![baz, quux, foo]).unwrap();
//! assert_eq!(prefix, Path::new("/foo/bar").to_path_buf());
//! # }
//! ```
#[cfg(test)]
extern crate rand;

use std::path::{Path, PathBuf};

/// Find the common prefix, if any, between any number of paths
///
/// # Example
///
/// ```rust
/// # extern crate common_path;
/// use std::path::{PathBuf, Path};
/// use common_path::common_path_all;
///
/// # fn main() {
/// let baz = Path::new("/foo/bar/baz");
/// let quux = Path::new("/foo/bar/quux");
/// let foo = Path::new("/foo/bar/foo");
/// let prefix = common_path_all(vec![baz, quux, foo]).unwrap();
/// assert_eq!(prefix, Path::new("/foo/bar").to_path_buf());
/// # }
/// ```
pub fn common_path_all<'a>(paths: impl IntoIterator<Item = &'a Path>) -> Option<PathBuf> {
    let mut path_iter = paths.into_iter();
    let mut result = path_iter.next()?.to_path_buf();
    for path in path_iter {
        if let Some(r) = common_path(&result, &path) {
            result = r;
        } else {
            return None;
        }
    }
    Some(result.to_path_buf())
}

/// Find the common prefix, if any, between 2 paths
///
/// # Example
///
/// ```rust
/// # extern crate common_path;
/// use std::path::{PathBuf, Path};
/// use common_path::common_path;
///
/// # fn main() {
/// let baz = Path::new("/foo/bar/baz");
/// let quux = Path::new("/foo/bar/quux");
/// let prefix = common_path(baz, quux).unwrap();
/// assert_eq!(prefix, Path::new("/foo/bar").to_path_buf());
/// # }
/// ```
pub fn common_path<P, Q>(one: P, two: Q) -> Option<PathBuf>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let one = one.as_ref();
    let two = two.as_ref();
    let one = one.components();
    let two = two.components();
    let mut final_path = PathBuf::new();
    let mut found = false;
    let paths = one.zip(two);
    for (l, r) in paths {
        if l == r {
            final_path.push(l.as_os_str());
            found = true;
        } else {
            break;
        }
    }
    if found {
        Some(final_path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{
        thread_rng,
        seq::SliceRandom,
    };

    #[test]
    fn compare_all_paths() {
        let mut rng = thread_rng();
        for _ in 0..6 {
            let one = Path::new("/foo/bar/baz/one.txt");
            let two = Path::new("/foo/bar/quux/quuux/two.txt");
            let three = Path::new("/foo/bar/baz/foo/bar");
            let result = Path::new("/foo/bar");
            let mut all = vec![one, two, three];
            all.shuffle(&mut rng);
            assert_eq!(common_path_all(all).unwrap(), result.to_path_buf())
        }
    }

    #[test]
    fn compare_paths() {
        let one = Path::new("/foo/bar/baz/one.txt");
        let two = Path::new("/foo/bar/quux/quuux/two.txt");
        let result = Path::new("/foo/bar");
        assert_eq!(common_path(&one, &two).unwrap(), result.to_path_buf())
    }

    #[test]
    fn no_common_path() {
        let one = Path::new("/foo/bar");
        let two = Path::new("./baz/quux");
        assert!(common_path(&one, &two).is_none());
    }
}
