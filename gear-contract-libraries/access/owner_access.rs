use gstd::{msg, prelude::*, ActorId};

pub use storage_traits::{OwnableStorage};
pub use modifier::modifier;

pub trait OwnableStorage {
    fn get(&self) -> &OwnableData;
    fn get_mut(&mut self) -> &mut OwnableData;
}

#[derive(Debug, Default)]
pub struct OwnableData {
    pub owner: ActorId,
}

pub fn only_owner<T, F>(instance: &mut T, body: F) 
where
    T: OwnableStorage,
    F: FnOnce(&mut T),
{
    if instance.get().owner != msg::source() {
        panic!();
    }
    body(instance)
}

