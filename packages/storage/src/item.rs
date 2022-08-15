use std::{any::type_name};

use std::marker::PhantomData;

use cosmwasm_std::{Storage, ReadonlyStorage, StdResult, StdError};
use secret_toolkit_serialization::{Serde, Bincode2};
use serde::{Serialize, de::DeserializeOwned};

/// This storage struct is based on Item from cosmwasm-storage-plus
pub struct Item<'a, T, Ser = Bincode2>
    where
        T: Serialize + DeserializeOwned,
        Ser: Serde,
{
    storage_key: &'a [u8],
    item_type: PhantomData<T>,
    serialization_type: PhantomData<Ser>,
}

impl<'a, T: Serialize + DeserializeOwned, Ser: Serde> Item<'a, T, Ser> {
    pub const fn new(key: &'a [u8]) -> Self {
        Self {
            storage_key: key,
            item_type: PhantomData,
            serialization_type: PhantomData,
        }
    }
}

impl<'a, T, Ser> Item<'a, T, Ser>
where
    T: Serialize + DeserializeOwned,
    Ser: Serde
{
    /// save will serialize the model and store, returns an error on serialization issues
    pub fn save<S: Storage>(&self, storage: &mut S, data: &T) -> StdResult<()> {
        self.save_impl(storage, data)
    }

    /// userfacing remove function
    pub fn remove<S: Storage>(&self, storage: &mut S) {
        self.remove_impl(storage);
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load<S: ReadonlyStorage>(&self, storage: &S) -> StdResult<T> {
        self.load_impl(storage)
    }

    /// may_load will parse the data stored at the key if present, returns `Ok(None)` if no data there.
    /// returns an error on issues parsing
    pub fn may_load<S: ReadonlyStorage>(&self, storage: &S) -> StdResult<Option<T>> {
        self.may_load_impl(storage)
    }
    
    /// efficient way to see if any object is currently saved.
    pub fn is_empty<S: ReadonlyStorage>(&self, storage: &S) -> bool {
        match storage.get(self.as_slice()) {
            Some(_) => true,
            None => false,
        }
    }

    /// Loads the data, perform the specified action, and store the result
    /// in the database. This is shorthand for some common sequences, which may be useful.
    ///
    /// It assumes, that data was initialized before, and if it doesn't exist, `Err(StdError::NotFound)`
    /// is returned.
    pub fn update<S, A>(&self, storage: &mut S, action: A) -> StdResult<T>
    where
        S: Storage,
        A: FnOnce(T) -> StdResult<T>
    {
        let input = self.load_impl(storage)?;
        let output = action(input)?;
        self.save_impl(storage, &output)?;
        Ok(output)
    }

    /// Returns StdResult<T> from retrieving the item with the specified key.  Returns a
    /// StdError::NotFound if there is no item with that key
    ///
    /// # Arguments
    ///
    /// * `storage` - a reference to the storage this item is in
    fn load_impl<S: ReadonlyStorage>(&self, storage: &S) -> StdResult<T> {
        Ser::deserialize(
            &storage.get(self.as_slice())
                .ok_or(StdError::not_found(type_name::<T>()))?,
        )
    }

    /// Returns StdResult<Option<T>> from retrieving the item with the specified key.  Returns a
    /// None if there is no item with that key
    ///
    /// # Arguments
    ///
    /// * `storage` - a reference to the storage this item is in
    fn may_load_impl<S: ReadonlyStorage>(&self, storage: &S) -> StdResult<Option<T>> {
        match storage.get(self.as_slice()) {
            Some(value) => Ser::deserialize(&value).map(Some),
            None => Ok(None),
        }
    }

    /// Returns StdResult<()> resulting from saving an item to storage
    ///
    /// # Arguments
    ///
    /// * `storage` - a mutable reference to the storage this item should go to
    /// * `value` - a reference to the item to store
    fn save_impl<S: Storage>(&self, storage: &mut S, value: &T) -> StdResult<()> {
        storage.set(self.as_slice(), &Ser::serialize(value)?);
        Ok(())
    }

    /// Removes an item from storage
    ///
    /// # Arguments
    ///
    /// * `storage` - a mutable reference to the storage this item is in
    fn remove_impl<S: Storage>(&self, storage: &mut S) {
        storage.remove(self.as_slice());
    }

    fn as_slice(&self) -> &[u8] {
        self.storage_key
    }
}