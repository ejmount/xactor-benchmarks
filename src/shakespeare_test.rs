use std::sync::Arc;

use shakespeare::{actor, ActorHandles};

use super::{Result as BenchResult, Spec};

#[derive(Clone)]
pub struct Data(std::sync::Arc<String>);

#[derive(Clone)]
struct CloseRing {
    first: Arc<RingActor>,
}

//static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

// The ring actor
#[actor]
mod RingActor {
    use shakespeare::Context;

    struct RingState {
        // Next actor in the ring - must allow None as the actor only knows
        // about the next actor once it has been created.
        next: Option<Arc<RingActor>>,
        // The actor id (place in ring)
        id: u32,
        // Number of messages to pass
        msgs: u32,
        parallel: u32,
    }

    #[performance(canonical)]
    impl DataRep for RingState {
        async fn close(&mut self, msg: CloseRing) {
            match self.next {
                // If we have a next we pass this message on to the next actor
                Some(ref mut next) => {
                    next.close(msg);
                }
                // If not weŕe the last actor and set the first node as our next node
                None => {
                    self.next = Some(msg.first);
                }
            };
        }

        async fn handle(&mut self, ctx: &'_ mut Context<Self>, msg: Data) -> () {
            match self.next {
                Some(ref mut next) => {
                    // If we do we check if we are the first process
                    if self.id == 0 {
                        // If so we know we can end our system if we have send our
                        // messages
                        if self.msgs == 0 {
                            ctx.stop();
                        } else {
                            // Otherwise we decrement the message count and send a new
                            // `Data` message to the next actor in the ring.
                            self.msgs -= 1;

                            next.handle(msg);
                        }
                    } else {
                        // If we are not the first process we just keep passing on
                        // `Data` messages
                        next.handle(msg);
                    }
                }
                // so if it does we panic!
                None => panic!(
                    "[{}] Next was null! This is not a ring it's a string :( parallel={}",
                    self.id, self.parallel
                ),
            };
        }
    }
}

// Actor implementation
pub async fn run(spec: &Spec) -> BenchResult {
    //println!("ENTRY");
    // Pre-generate the payload so weŕe not measuring string
    // creation.
    let data = Arc::new((0..spec.size).map(|_| "x").collect::<String>());

    let ActorHandles {
        mut message_handle,
        join_handle,
        ..
    } = RingActor::start(RingState {
        next: None,
        id: 0,
        msgs: spec.messages,
        parallel: spec.parallel,
    });

    let first_handle = message_handle.clone();

    for n in 0..spec.procs {
        message_handle = RingActor::start(RingState {
            next: Some(message_handle),
            id: n + 1,
            msgs: spec.messages,
            parallel: spec.parallel,
        })
        .message_handle;
    }

    first_handle.close(CloseRing {
        first: message_handle.clone(),
    });
    //.await
    //.unwrap();

    // Next we put Data messages on the ring limited by the number
    // of parallel messages we want.
    for _ in 0..spec.parallel {
        message_handle.handle(Data(data.clone()));
    }

    join_handle.await;

    // The ring will run until our first actor decides it's time to shut down.
    BenchResult {
        name: String::from("rust_shakespeare"),
        spec: spec.clone(),
    }
}
