use getrandom::{register_custom_getrandom, Error};
use ic_cdk::api::management_canister::main::raw_rand;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use std::{cell::RefCell, marker::PhantomData};

register_custom_getrandom!(ic_getrandom);

thread_local! {
    static RNG: RefCell<Option<ChaCha20Rng>> = RefCell::new(None);
}

pub struct RngGuard(PhantomData<()>);

impl Drop for RngGuard {
    fn drop(&mut self) {
        RNG.with(|rng| {
            // Zeroize memory
            *rng.borrow_mut() = Some(ChaCha20Rng::from_seed([0u8; 32]));
            // Now make it None
            rng.borrow_mut().take();
        });
    }
}

#[allow(unused)]
pub async fn enter() -> RngGuard {
    let mut bytes = raw_rand().await.expect("failed to call raw_rand").0;
    let seed = bytes[..].try_into().expect("not 32 bytes");

    // Zeroise vec
    bytes.fill(0);

    // Init generator
    RNG.with(move |rng| {
        *rng.borrow_mut() = Some(ChaCha20Rng::from_seed(seed));
    });

    RngGuard(PhantomData)
}

fn ic_getrandom(dest: &mut [u8]) -> Result<(), Error> {
    RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        let rng = rng
            .as_mut()
            .expect("random not initialized. use `random::enter().await`.");
        rng.fill_bytes(dest);
    });
    Ok(())
}
