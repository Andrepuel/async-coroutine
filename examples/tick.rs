use async_coroutine::coroutine_start;
use std::{cell::Cell, time::Duration};
use tokio::{runtime, task::LocalSet};

fn main() {
    let runtime = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let localset = LocalSet::new();
    localset.spawn_local(coroutine_start(|awaiter| {
        println!("[coroutine] coroutine started");
        loop {
            let back = awaiter.await_on(async move {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                "tick bg".to_string()
            });
            println!("{back}");
        }
    }));

    let borrow_example = Cell::new(0);

    localset.block_on(
        &runtime,
        coroutine_start(|awaiter| {
            let ticks = &borrow_example;
            println!("[coroutine] coroutine started");
            loop {
                ticks.set(ticks.take() + 1);
                let back = awaiter.await_on(async move {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    "tick coroutine".to_string()
                });
                println!("{back}");
            }
        }),
    );
}
