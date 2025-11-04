#![allow(warnings)]
use shakespeare::actor;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;

pub struct TestData {
    actors: Arc<RwLock<Vec<Arc<SquareActor>>>>,
    semaphore: Arc<Semaphore>,
}

#[actor]
mod SquareActor {

    struct State {
        id: usize,
        others: Arc<RwLock<Vec<Arc<SquareActor>>>>,
        received: usize,
        expected: usize,
        permit: Option<OwnedSemaphorePermit>,
    }

    #[performance(canonical)]
    impl Msg for State {
        async fn ping(&mut self) {
            let list = self.others.read().await;
            assert!(!list.is_empty());
            for a in list.iter() {
                a.pong();
            }
        }
        fn pong(&mut self) {
            self.received += 1;

            if self.received >= self.expected {
                let _s = { self.permit.take().unwrap().semaphore().clone() };
            }
        }
        fn reset(&mut self) {
            self.received = 0;
        }
    }
}

pub fn shakespeare_setup(n: usize) -> TestData {
    let semaphore = Arc::new(Semaphore::new(n));
    let actors = Arc::new(RwLock::new(vec![]));
    for id in 0..n {
        let a = SquareActor::start(State {
            id,
            others: actors.clone(),
            received: 0,
            expected: n,
            permit: Some(semaphore.clone().try_acquire_owned().unwrap()),
        })
        .message_handle;
        actors.try_write().unwrap().push(a);
    }

    TestData { actors, semaphore }
}

pub async fn shakespeare_run(t: TestData) {
    assert_eq!(t.semaphore.available_permits(), 0);
    let total = t.actors.read().await.len();
    for a in t.actors.read().await.iter() {
        a.ping();
    }
    let _p = t
        .semaphore
        .acquire_many(total as _)
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Timeout expired, {} permits available",
                t.semaphore.available_permits()
            )
        });
}

#[test]
fn square_test() {
    let rt = Runtime::new().unwrap();

    let _run = rt.block_on(async {
        let setup = shakespeare_setup(3);
        shakespeare_run(setup).await
    });
}
