#![allow(warnings)]
use crate::gen::gen_tests;

pub mod actix_test;
mod gen;
pub mod shakespeare_test;
mod square_test;
pub mod xactor_test;

#[derive(Debug, Clone)]
pub struct Spec {
    pub procs: u32,
    pub messages: u32,
    pub parallel: u32,
    pub size: u32,
}

impl std::fmt::Display for Spec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} procs with {} messages {} in parallel of a size of {}",
            self.procs, self.messages, self.parallel, self.size
        )
    }
}

#[derive(Debug, Clone)]
pub struct Result {
    pub name: String,
    pub spec: Spec,
}

impl std::fmt::Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{},{}",
            self.name, self.spec.procs, self.spec.messages, self.spec.parallel, self.spec.size,
        )
    }
}

fn main() {}

#[test]
fn test_shakespeare() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let tests = gen_tests(Some(2));
    println!("Num tests {}", tests.len());
    let mut results = vec![];
    for spec in tests.into_iter() {
        results.push(rt.block_on((shakespeare_test::run(&spec))));
    }
    assert!(results.len() > 0);
}
