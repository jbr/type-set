use crate::{unwrap, Key, Value};
use std::{
    any::{type_name, Any, TypeId},
    collections::btree_map,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// A view into a single type in the `TypeSet`, which may be either vacant or occupied.
///
/// This type is constructed by [`TypeSet::entry`][crate::TypeSet::entry]
///
/// ## Examples
///
/// This is a somewhat contrived example that demonstrates matching on the [`Entry`]. Often,
/// [`Entry::or_insert`], [`Entry::or_insert_with`], and [`Entry::and_modify`] can achieve
/// comparable results. See those functions for further usage examples.
///
/// ```rust
/// use type_set::{TypeSet, entry::Entry};
/// let mut set = TypeSet::new().with("hello");
/// let (previous, current) = match set.entry::<&'static str>() {
///     Entry::Vacant(vacant_entry) => {
///         let current = vacant_entry.insert("entry was vacant");
///         (None, current)
///     }
///
///     Entry::Occupied(mut occupied_entry) => {
///         let previous = occupied_entry.insert("entry was occupied");
///         (Some(previous), occupied_entry.into_mut())
///     }
/// };
/// assert_eq!(previous, Some("hello"));
/// assert_eq!(*current, "entry was occupied");
/// ```
pub enum Entry<'a, T> {
    /// A view into the location a T would be stored in the `TypeSet`. See [`VacantEntry`]
    Vacant(VacantEntry<'a, T>),

    /// A view into the location a T is currently stored in the `TypeSet`. See [`OccupiedEntry`]
    Occupied(OccupiedEntry<'a, T>),
}

impl<'a, T: Debug + Any + Send + Sync + 'static> Debug for Entry<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vacant(vacant_entry) => f.debug_tuple("Vacant").field(vacant_entry).finish(),
            Self::Occupied(occupied_entry) => {
                f.debug_tuple("Occupied").field(occupied_entry).finish()
            }
        }
    }
}

/// A view into a vacant entry in a `TypeSet`.
///
/// It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, T>(
    pub(super) btree_map::VacantEntry<'a, Key, Value>,
    PhantomData<T>,
);

impl<'a, T: Debug + Any + Send + Sync + 'static> Debug for VacantEntry<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "VacantEntry<{}>", type_name::<T>())
    }
}
/// A view into the location a T is stored
pub struct OccupiedEntry<'a, T>(
    pub(super) btree_map::OccupiedEntry<'a, Key, Value>,
    PhantomData<T>,
);

impl<'a, T: Debug + Any + Send + Sync + 'static> Debug for OccupiedEntry<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple(&format!("OccupiedEntry<{}>", type_name::<T>()))
            .field(unwrap!(self.0.get().downcast_ref::<T>()))
            .finish()
    }
}

impl<'a, T: Send + Sync + 'static> Entry<'a, T> {
    /// Ensures a value is in the `Entry` by inserting the provided `default` value if the Entry was
    /// previously vacant. Returns a mutable reference to the value.
    ///
    /// Prefer [`Entry::or_insert_with`] if constructing a T is expensive.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let mut set = type_set::TypeSet::new();
    /// assert_eq!(*set.entry().or_insert("hello"), "hello");
    /// assert_eq!(set.get::<&'static str>(), Some(&"hello"));
    /// assert_eq!(*set.entry().or_insert("world"), "hello");
    /// assert_eq!(set.get::<&'static str>(), Some(&"hello"));
    /// ```
    pub fn or_insert(self, default: T) -> &'a mut T {
        match self {
            Entry::Vacant(vacant) => vacant.insert(default),
            Entry::Occupied(occupied) => occupied.into_mut(),
        }
    }

    /// Ensures a value is in the `Entry` by inserting the provided value returned by the `default`
    /// function if the `Entry` was previously vacant. Returns a mutable reference to the value.
    ///
    /// Prefer this to [`Entry::or_insert`] if constructing a T is expensive.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let mut set = type_set::TypeSet::new();
    /// assert_eq!(*set.entry().or_insert_with(|| String::from("hello")), "hello");
    /// assert_eq!(set.get::<String>(), Some(&String::from("hello")));
    /// assert_eq!(*set.entry::<String>().or_insert_with(|| panic!("never called")), "hello");
    /// assert_eq!(set.get::<String>(), Some(&String::from("hello")));
    /// ```
    pub fn or_insert_with(self, default: impl FnOnce() -> T) -> &'a mut T {
        match self {
            Entry::Vacant(vacant) => vacant.insert(default()),
            Entry::Occupied(occupied) => occupied.into_mut(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any potential inserts into the
    /// set using [`Entry::or_insert`] or [`Entry::or_insert_with`].
    ///
    /// ## Example
    ///
    /// ```rust
    /// let mut set = type_set::TypeSet::new().with(String::from("hello"));
    /// let value = set.entry::<String>()
    ///     .and_modify(|s| s.push_str(" world"))
    ///     .or_insert_with(|| String::from("greetings"));
    /// assert_eq!(value, "hello world");
    ///
    /// set.take::<String>();
    /// let value = set.entry::<String>()
    ///     .and_modify(|s| s.push_str(" world"))
    ///     .or_insert_with(|| String::from("greetings"));
    /// assert_eq!(value, "greetings");
    /// ```
    #[must_use]
    pub fn and_modify(self, f: impl FnOnce(&mut T)) -> Self {
        match self {
            Entry::Vacant(vacant) => Entry::Vacant(vacant),
            Entry::Occupied(mut occupied) => {
                f(occupied.get_mut());
                Entry::Occupied(occupied)
            }
        }
    }

    /// Remove and return a value from this entry, if occupied.
    #[must_use]
    pub fn take(self) -> Option<T> {
        self.into_occupied().map(OccupiedEntry::remove)
    }

    /// Returns an `OccupiedEntry` or panic
    ///
    /// # Panics
    ///
    /// This function will panic if the entry is vacant
    #[must_use]
    pub fn unwrap_occupied(self) -> OccupiedEntry<'a, T> {
        self.into_occupied().unwrap_or_else(|| {
            panic!(
                "expected an occupied type-set entry for {}, but was vacant",
                type_name::<T>()
            )
        })
    }

    /// Returns a `VacantEntry` or panic
    ///
    /// # Panics
    ///
    /// This function will panic if the entry is occupied
    #[must_use]
    pub fn unwrap_vacant(self) -> VacantEntry<'a, T> {
        self.into_vacant().unwrap_or_else(|| {
            panic!(
                "expected a vacant type-set entry for {}, but was occupied",
                type_name::<T>()
            )
        })
    }

    /// Returns a mutable reference to the contained type, if this entry is occupied
    #[must_use]
    pub fn into_mut(self) -> Option<&'a mut T> {
        self.into_occupied().map(OccupiedEntry::into_mut)
    }

    /// Returns an [`OccupiedEntry`] or `None` if this entry is vacant.
    #[must_use]
    pub fn into_occupied(self) -> Option<OccupiedEntry<'a, T>> {
        match self {
            Entry::Occupied(occupied_entry) => Some(occupied_entry),
            Entry::Vacant(_) => None,
        }
    }

    /// Returns a [`VacantEntry`] or `None` if this entry is occupied.
    #[must_use]
    pub fn into_vacant(self) -> Option<VacantEntry<'a, T>> {
        match self {
            Entry::Occupied(_) => None,
            Entry::Vacant(vacant_entry) => Some(vacant_entry),
        }
    }

    /// Returns whether this `Entry` is a [`VacantEntry`]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Entry::Vacant(_))
    }

    /// Insert a value into this [`Entry`].
    ///
    /// If the Entry is already an [`OccupiedEntry`], the previously contained value will be
    /// returned
    pub fn insert(self, value: T) -> Option<T> {
        match self {
            Entry::Vacant(v) => {
                #[cfg(feature = "log")]
                log::trace!("inserting {}", type_name::<T>());
                v.insert(value);
                None
            }

            Entry::Occupied(mut o) => {
                #[cfg(feature = "log")]
                log::trace!("replacing {}", type_name::<T>());
                Some(o.insert(value))
            }
        }
    }

    pub(super) fn new(entry: btree_map::Entry<'a, TypeId, Value>) -> Self {
        match entry {
            btree_map::Entry::Vacant(vacant) => Self::Vacant(VacantEntry(vacant, PhantomData)),
            btree_map::Entry::Occupied(occupied) => {
                Self::Occupied(OccupiedEntry(occupied, PhantomData))
            }
        }
    }
}

impl<'a, T: Default + Send + Sync + 'static> Entry<'a, T> {
    /// Ensures a value is in the Entry by inserting the default value if vacant, and returns a
    /// mutable reference to the value.
    ///
    /// Equivalent to `.or_insert_with(Default::default)`
    ///
    /// ## Example
    ///
    /// ```rust
    /// let mut set = type_set::TypeSet::new();
    /// assert_eq!(*set.entry::<&'static str>().or_default(), "");
    /// set.insert("hello");
    /// assert_eq!(*set.entry::<&'static str>().or_default(), "hello");
    /// ```
    pub fn or_default(self) -> &'a mut T {
        #[allow(clippy::unwrap_or_default)]
        // this is the implementation of or_default so it can't call or_default
        self.or_insert_with(T::default)
    }
}

impl<'a, T: Send + Sync + 'static> VacantEntry<'a, T> {
    /// Sets the value of this entry to the provided `value`
    pub fn insert(self, value: T) -> &'a mut T {
        unwrap!(self.0.insert(Value::new(value)).downcast_mut())
    }
}

impl<'a, T: Send + Sync + 'static> OccupiedEntry<'a, T> {
    /// Gets a reference to the value in this entry
    #[must_use]
    pub fn get(&self) -> &T {
        unwrap!(self.0.get().downcast_ref())
    }

    /// Gets a mutable reference to the value in the entry
    ///
    /// If you need a reference to the `OccupiedEntry` that may outlive the
    /// destruction of the `Entry` value, see [`OccupiedEntry::into_mut`].
    #[must_use]
    pub fn get_mut(&mut self) -> &mut T {
        unwrap!(self.0.get_mut().downcast_mut())
    }

    /// Sets the value of the entry to `value`, returning the entry's previous value.
    pub fn insert(&mut self, value: T) -> T {
        unwrap!(self.0.insert(Value::new(value)).downcast())
    }

    /// Take ownership of the value from this Entry
    #[allow(clippy::must_use_candidate)] // sometimes we just want to take the value out and drop it
    pub fn remove(self) -> T {
        unwrap!(self.0.remove().downcast())
    }

    /// Converts the entry into a mutable reference to its value.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`OccupiedEntry::get_mut`].
    #[must_use]
    pub fn into_mut(self) -> &'a mut T {
        unwrap!(self.0.into_mut().downcast_mut())
    }
}

impl<'a, T: Send + Sync + 'static> Deref for OccupiedEntry<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a, T: Send + Sync + 'static> DerefMut for OccupiedEntry<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<'a, T: Send + Sync + 'static> From<OccupiedEntry<'a, T>> for Entry<'a, T> {
    fn from(occupied_entry: OccupiedEntry<'a, T>) -> Self {
        Self::Occupied(occupied_entry)
    }
}

impl<'a, T: Send + Sync + 'static> From<VacantEntry<'a, T>> for Entry<'a, T> {
    fn from(vacant_entry: VacantEntry<'a, T>) -> Self {
        Self::Vacant(vacant_entry)
    }
}
