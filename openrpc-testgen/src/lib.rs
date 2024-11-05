pub mod suite_common;
pub mod suite_katana;
pub mod suite_madara;
pub mod utils;

pub trait RunnableTrait {
    fn run(&self) {}
}
