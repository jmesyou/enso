use crate::*;



// ==============
// === Export ===
// ==============

#[cfg(target_arch = "wasm32")]
pub mod internal {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace=console)]
        fn error(msg: String);

        type Error;

        #[wasm_bindgen(constructor)]
        fn new() -> Error;

        #[wasm_bindgen(structural, method, getter)]
        fn stack(error: &Error) -> String;
    }

    /// Print the current backtrace.
    pub fn backtrace() -> String {
        Error::new().stack()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod internal {
    extern crate backtrace as bt;
    use bt::Backtrace;

    /// Print the current backtrace.
    pub fn backtrace() -> String {
        let bt = Backtrace::new();
        format!("{bt:?}")
    }
}

pub use internal::backtrace;



// ===================
// === TraceCopies ===
// ===================

/// A utility for tracing all copies of CloneRef-able entity.
///
/// This structure should be added as a field to structure implementing Clone or CloneRef. It will
/// mark each copy with unique id (the original copy has id of 0). Once enabled, it will print
/// backtrace of each clone, clone_ref or drop operation with assigned name (the same for all
/// copies) and copy id.
#[derive(Debug, Default)]
pub struct TraceCopies {
    clone_id: u64,
    handle:   Rc<RefCell<Option<ImString>>>,
}

thread_local! {
    static NEXT_CLONE_ID : Cell<u64> = Cell::new(1);
}

fn next_clone_id() -> u64 {
    NEXT_CLONE_ID.with(|id| {
        let next = id.get();
        id.set(next + 1);
        next
    })
}

impl TraceCopies {
    /// Create enabled structure with appointed entity name (shared between all copies).
    pub fn enabled(name: impl Into<ImString>) -> Self {
        let this: Self = default();
        this.enable(name);
        this
    }

    /// Assign a name to the entity (shared between all copies) and start printing logs.
    pub fn enable(&self, name: impl Into<ImString>) {
        let name = name.into();
        debug!("[{name}] TraceCopies enabled");
        *self.handle.borrow_mut() = Some(name);
    }
}

impl Clone for TraceCopies {
    fn clone(&self) -> Self {
        let borrow = self.handle.borrow();
        let clone_id = next_clone_id();
        let handle = self.handle.clone();
        if let Some(name) = &*borrow {
            let bt = backtrace();
            debug!("[{name}] Cloning {} -> {clone_id} {bt}", self.clone_id);
        }
        Self { clone_id, handle }
    }
}

impl CloneRef for TraceCopies {
    fn clone_ref(&self) -> Self {
        let borrow = self.handle.borrow();
        let clone_id = next_clone_id();
        let handle = self.handle.clone_ref();
        if let Some(name) = &*borrow {
            let bt = backtrace();
            debug!("[{name}] Cloning {} -> {clone_id} {bt}", self.clone_id);
        }
        Self { clone_id, handle }
    }
}

impl Drop for TraceCopies {
    fn drop(&mut self) {
        let borrow = self.handle.borrow();
        if let Some(name) = &*borrow {
            let bt = backtrace();
            let instances = Rc::strong_count(&self.handle) - 1;
            debug!("[{name}] Dropping {}; instances left: {instances} {bt}", self.clone_id);
        }
    }
}
