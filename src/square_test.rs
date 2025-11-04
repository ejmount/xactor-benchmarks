#![allow(warnings)]
use shakespeare::actor;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::OwnedSemaphorePermit;
use tokio::sync::Semaphore;
use xactor::timeout;

pub struct TestData {
    actors: Arc<Vec<Arc<SquareActor>>>,
    semaphore: Arc<Semaphore>,
}

#[actor]
mod SquareActor {

    struct State {
        id: usize,
        others: Arc<Vec<Arc<SquareActor>>>,
        received: usize,
        expected: usize,
        permit: Option<OwnedSemaphorePermit>,
    }

    #[performance(canonical)]
    impl Msg for State {
        fn ping(&self) {
            for a in self.others.iter() {
                a.pong();
            }
        }
        fn pong(&mut self) {
            self.received += 1;
            if cfg!(test) {
                println!(
                    "ID {} received {} waiting on {}",
                    self.id, self.received, self.expected
                );
            }
            if self.received >= self.expected {
                let s = { self.permit.take().unwrap().semaphore().clone() };
                assert!(self.permit.is_none());
                println!(
                    "ID {} is dropping, {} permits dropped",
                    self.id,
                    s.available_permits()
                );
            }
        }
        fn reset(&mut self) {
            self.received = 0;
        }
        fn update_actor_set(&mut self, actors: Arc<Vec<Arc<SquareActor>>>) {
            self.others = actors;
        }
        fn send_permit(&mut self, p: OwnedSemaphorePermit) {
            self.permit = Some(p);
        }
    }
}

pub fn shakespeare_setup(n: usize) -> TestData {
    let semaphore = Arc::new(Semaphore::new(n));
    let mut actors: Vec<_> = vec![];
    for id in 0..n {
        let a = SquareActor::start(State {
            id,
            others: Arc::new(vec![]),
            received: 0,
            expected: n,
            permit: None,
        })
        .message_handle;
        actors.push(a);
    }
    let actors = Arc::new(actors);

    for a in actors.iter() {
        a.update_actor_set(actors.clone());
        a.send_permit(semaphore.clone().try_acquire_owned().unwrap());
    }

    TestData { actors, semaphore }
}

pub async fn shakespeare_run(t: TestData) {
    assert_eq!(t.semaphore.available_permits(), 0);
    for a in t.actors.iter() {
        a.ping();
    }
    //println!("Wating for pings");
    let p = //timeout(
        //Duration::from_secs(1),
        t.semaphore.acquire_many(t.actors.len() as _)
    //)
    .await
    .unwrap_or_else(|_| {
        panic!(
            "Timeout expired, {} permits available",
            t.semaphore.available_permits()
        )
    });
    //.unwrap();
    dbg!(p.num_permits());
}

#[test]
fn square_test() {
    let rt = Runtime::new().unwrap();

    let _run = rt.block_on(async {
        let setup = shakespeare_setup(3);
        shakespeare_run(setup).await
    });
}
