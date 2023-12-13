use std::marker::PhantomData;

use crate::backend::StorageBackend;
use crate::encoding::{DecodableWith, EncodableWith, Encoding};
use crate::init::StorageInit;

struct Item<'k, E, T> {
    prefix: &'k [u8],
    phantom: PhantomData<(T, E)>,
}

impl<'k, E, T> Item<'k, E, T>
where
    E: Encoding,
    T: DecodableWith<E> + EncodableWith<E>,
{
    pub fn new(prefix: &'k [u8]) -> Self {
        Self {
            prefix,
            phantom: PhantomData,
        }
    }

    pub fn get(&self, storage: &mut impl StorageBackend, key: &[u8]) -> Option<T> {
        let data = storage.get(key)?;
        let item = T::decode(&data).ok()?;
        Some(item)
    }
}

impl<T, E> StorageInit<E> for Item<'_, T, E>
where
    E: Encoding,
{
    fn init(&self, storage: &mut impl StorageBackend) {}
}
