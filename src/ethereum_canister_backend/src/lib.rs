use ic_cdk::{init, update};

mod random;

#[init]
async fn init() {
    ic_cdk::setup();
}

#[update]
async fn greet() -> String {
    let _guard = random::enter().await;

    let n: u32 = rand::random();
    format!("Hello, here is a random number: {}", n)
}
