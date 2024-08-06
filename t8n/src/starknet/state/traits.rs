/// This trait should be implemented by structures that internally have collections and each element
/// could be found by a hash
pub trait HashIdentified {
    type Element;
    type Hash;

    fn get_by_hash(&self, hash: Self::Hash) -> Option<&Self::Element>;
}

pub trait HashIdentifiedMut {
    type Element;
    type Hash;

    fn get_by_hash_mut(&mut self, hash: &Self::Hash) -> Option<&mut Self::Element>;
}
