pub mod suite_common;
pub mod suite_katana;
pub mod suite_madara;

pub trait TestSuiteTrait {
    fn run(&self) {}
}
