pub struct TestCase {
    pub tmp: String,
}

pub trait TestCaseTrait {
    fn run(&self) {}
}

impl TestCaseTrait for TestCase {
    fn run(&self) {
        println!("COMMON V2");
    }
}
