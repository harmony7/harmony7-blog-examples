wit_bindgen::generate!({ path: "../wit", world: "tally-provider", generate_all });

use std::cell::Cell;

use exports::local::main::tally::{Guest, GuestTally};

/// The resource's representation, which lives in this component's memory.
struct Tally {
    name: String,
    total: Cell<u32>,
}

impl GuestTally for Tally {
    fn new(name: String) -> Self {
        eprintln!("tally: created {name}");
        Tally { name, total: Cell::new(0) }
    }

    fn add(&self, n: u32) -> u32 {
        self.total.set(self.total.get() + n);
        self.total.get()
    }
}

impl Drop for Tally {
    fn drop(&mut self) {
        eprintln!("tally: dropped {} at {}", self.name, self.total.get());
    }
}

struct Provider;

impl Guest for Provider {
    type Tally = Tally;
}

export!(Provider);
