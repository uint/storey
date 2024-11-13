use std::marker::PhantomData;

use crate::encoding::{DecodableWith, EncodableWith, Encoding};
use crate::storage::StorageBranch;
use crate::storage::{Storage, StorageMut};

use super::common::TryGetError;
use super::{Storable, Terminal};

/// A single item in the storage.
///
/// This simple container doesn't manage a namespace of keys, but simply stores a single
/// value under a single key.
///
/// # Example
/// ```
/// # use mocks::encoding::TestEncoding;
/// # use mocks::backend::TestStorage;
/// use storey::containers::Item;
///
/// let mut storage = TestStorage::new();
/// let item = Item::<u64, TestEncoding>::new(0);
///
/// item.access(&mut storage).set(&42).unwrap();
/// assert_eq!(item.access(&storage).get().unwrap(), Some(42));
/// ```
pub struct Item<T, E> {
    key: u8,
    phantom: PhantomData<(T, E)>,
}

impl<T, E> Item<T, E>
where
    E: Encoding,
    T: EncodableWith<E> + DecodableWith<E>,
{
    /// Create a new item with the given key.
    ///
    /// It is the responsibility of the caller to ensure that the key is unique.
    pub const fn new(key: u8) -> Self {
        Self {
            key,
            phantom: PhantomData,
        }
    }

    /// Acquire an accessor to the item.
    ///
    /// # Example
    /// ```
    /// # use mocks::encoding::TestEncoding;
    /// # use mocks::backend::TestStorage;
    /// use storey::containers::Item;
    ///
    /// // immutable accessor
    /// let storage = TestStorage::new();
    /// let item = Item::<u64, TestEncoding>::new(0);
    /// let access = item.access(&storage);
    ///
    /// // mutable accessor
    /// let mut storage = TestStorage::new();
    /// let item = Item::<u64, TestEncoding>::new(0);
    /// let mut access = item.access(&mut storage);
    pub fn access<S>(&self, storage: S) -> ItemAccess<E, T, StorageBranch<S>> {
        Self::access_impl(StorageBranch::new(storage, vec![self.key]))
    }
}

impl<T, E> Storable for Item<T, E>
where
    E: Encoding,
    T: EncodableWith<E> + DecodableWith<E>,
{
    type Kind = Terminal;
    type Accessor<S> = ItemAccess<E, T, S>;
    type Key = ();
    type KeyDecodeError = ItemKeyDecodeError;
    type Value = T;
    type ValueDecodeError = E::DecodeError;

    fn access_impl<S>(storage: S) -> ItemAccess<E, T, S> {
        ItemAccess {
            storage,
            phantom: PhantomData,
        }
    }

    fn decode_key(key: &[u8]) -> Result<(), ItemKeyDecodeError> {
        if key.is_empty() {
            Ok(())
        } else {
            Err(ItemKeyDecodeError)
        }
    }

    fn decode_value(value: &[u8]) -> Result<Self::Value, Self::ValueDecodeError> {
        T::decode(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, thiserror::Error)]
#[error("invalid key length, expected empty key")]
pub struct ItemKeyDecodeError;

/// An accessor for an `Item`.
///
/// This type provides methods to get and set the value of the item.
pub struct ItemAccess<E, T, S> {
    storage: S,
    phantom: PhantomData<(E, T)>,
}

impl<E, T, S> ItemAccess<E, T, S>
where
    E: Encoding,
    T: EncodableWith<E> + DecodableWith<E>,
    S: Storage,
{
    /// Get the value of the item.
    ///
    /// Returns `Ok(None)` if the item doesn't exist (has not been set yet).
    ///
    /// # Examples
    /// ```
    /// # use mocks::encoding::TestEncoding;
    /// # use mocks::backend::TestStorage;
    /// use storey::containers::Item;
    ///
    /// let storage = TestStorage::new();
    /// let item = Item::<u64, TestEncoding>::new(0);
    /// let access = item.access(&storage);
    ///
    /// assert_eq!(access.get().unwrap(), None);
    /// ```
    ///
    /// ```
    /// # use mocks::encoding::TestEncoding;
    /// # use mocks::backend::TestStorage;
    /// use storey::containers::Item;
    ///
    /// let mut storage = TestStorage::new();
    /// let item = Item::<u64, TestEncoding>::new(0);
    ///
    /// item.access(&mut storage).set(&42).unwrap();
    /// assert_eq!(item.access(&storage).get().unwrap(), Some(42));
    /// ```
    pub fn get(&self) -> Result<Option<T>, E::DecodeError> {
        self.storage
            .get(&[])
            .map(|bytes| T::decode(&bytes))
            .transpose()
    }

    /// Get the value of the item.
    ///
    /// Returns [`TryGetError::Empty`] if the item doesn't exist (has not been
    /// set yet).
    ///
    /// This is similar to [`get`](Self::get), but removes one level of nesting
    /// so that you can get to your data faster, without having to unpack the
    /// [`Option`].
    ///
    /// # Examples
    /// ```
    /// # use mocks::encoding::TestEncoding;
    /// # use mocks::backend::TestStorage;
    /// use storey::containers::Item;
    ///
    /// let mut storage = TestStorage::new();
    /// let item = Item::<u64, TestEncoding>::new(0);
    ///
    /// item.access(&mut storage).set(&42).unwrap();
    /// assert_eq!(item.access(&storage).try_get().unwrap(), 42);
    /// ```
    ///
    /// ```
    /// # use mocks::encoding::TestEncoding;
    /// # use mocks::backend::TestStorage;
    /// use storey::containers::Item;
    ///
    /// let storage = TestStorage::new();
    /// let item = Item::<u64, TestEncoding>::new(0);
    /// let access = item.access(&storage);
    ///
    /// assert!(access.try_get().is_err());
    /// ```
    pub fn try_get(&self) -> Result<T, TryGetError<E::DecodeError>> {
        self.get()?.ok_or_else(|| TryGetError::Empty)
    }

    /// Get the value of the item or a provided default.
    ///
    /// Returns the value of the item if it exists, otherwise returns the provided default.
    ///
    /// # Example
    /// ```
    /// # use mocks::encoding::TestEncoding;
    /// # use mocks::backend::TestStorage;
    /// use storey::containers::Item;
    ///
    /// let storage = TestStorage::new();
    /// let item = Item::<u64, TestEncoding>::new(0);
    ///
    /// assert_eq!(item.access(&storage).get_or(42).unwrap(), 42);
    /// ```
    pub fn get_or(&self, default: T) -> Result<T, E::DecodeError> {
        self.get().map(|opt| opt.unwrap_or(default))
    }
}

impl<E, T, S> ItemAccess<E, T, S>
where
    E: Encoding,
    T: EncodableWith<E> + DecodableWith<E>,
    S: Storage + StorageMut,
{
    /// Set the value of the item.
    ///
    /// # Example
    /// ```
    /// # use mocks::encoding::TestEncoding;
    /// # use mocks::backend::TestStorage;
    /// use storey::containers::Item;
    ///
    /// let mut storage = TestStorage::new();
    /// let item = Item::<u64, TestEncoding>::new(0);
    ///
    /// item.access(&mut storage).set(&42).unwrap();
    /// assert_eq!(item.access(&storage).get().unwrap(), Some(42));
    /// ```
    pub fn set(&mut self, value: &T) -> Result<(), E::EncodeError> {
        let bytes = value.encode()?;
        self.storage.set(&[], &bytes);
        Ok(())
    }

    pub fn update<F>(&mut self, f: F) -> Result<(), UpdateError<E>>
    where
        F: FnOnce(Option<T>) -> T,
    {
        let new_value = f(self.get().map_err(UpdateError::Decode)?);
        self.set(&new_value).map_err(UpdateError::Encode)
    }

    /// Remove the value of the item.
    ///
    /// # Example
    /// ```
    /// # use mocks::encoding::TestEncoding;
    /// # use mocks::backend::TestStorage;
    /// use storey::containers::Item;
    ///
    /// let mut storage = TestStorage::new();
    /// let item = Item::<u64, TestEncoding>::new(0);
    ///
    /// item.access(&mut storage).set(&42).unwrap();
    /// item.access(&mut storage).remove();
    /// assert_eq!(item.access(&storage).get().unwrap(), None);
    /// ```
    pub fn remove(&mut self) {
        self.storage.remove(&[]);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, thiserror::Error)]
pub enum UpdateError<E>
where
    E: Encoding,
    E::DecodeError: std::fmt::Display,
    E::EncodeError: std::fmt::Display,
{
    #[error("decode error: {0}")]
    Decode(E::DecodeError),
    #[error("encode error: {0}")]
    Encode(E::EncodeError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use mocks::backend::TestStorage;
    use mocks::encoding::TestEncoding;

    #[test]
    fn basic() {
        let mut storage = TestStorage::new();

        let item0 = Item::<u64, TestEncoding>::new(0);
        item0.access(&mut storage).set(&42).unwrap();

        let item1 = Item::<u64, TestEncoding>::new(1);
        let access1 = item1.access(&storage);

        assert_eq!(item0.access(&storage).get().unwrap(), Some(42));
        assert_eq!(storage.get(&[0]), Some(42u64.to_le_bytes().to_vec()));
        assert_eq!(access1.get().unwrap(), None);
        assert_eq!(storage.get(&[1]), None);
    }
}
