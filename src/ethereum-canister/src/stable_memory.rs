use std::cell::RefCell;
use std::thread::LocalKey;

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{Cell, DefaultMemoryImpl, Storable};

// IDs per variable in stable memory
pub(crate) const LAST_CONSENSUS_RPC_URL_ID: MemoryId = MemoryId::new(0);
pub(crate) const LAST_EXECUTION_RPC_URL_ID: MemoryId = MemoryId::new(1);
pub(crate) const LAST_CHECKPOINT_ID: MemoryId = MemoryId::new(2);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub(crate) type StableCell<T> = Cell<T, VirtualMemory<DefaultMemoryImpl>>;

pub(crate) fn memory_of(id: MemoryId) -> VirtualMemory<DefaultMemoryImpl> {
    MEMORY_MANAGER.with(|mngr| mngr.borrow().get(id))
}

/// If memory of `id` is already initialized then that content will be used, otherwise it is
/// initialized with `value`.
pub(crate) fn init_stable_cell<T>(id: MemoryId, value: T) -> StableCell<T>
where
    T: Storable,
{
    StableCell::init(memory_of(id), value).expect("failed to initialize StableCell")
}

/// If memory of `id` is already initialized then that content will be used, otherwise it is
/// initialized with `Default::default()` value.
pub(crate) fn init_stable_cell_default<T>(id: MemoryId) -> StableCell<T>
where
    T: Storable + Default,
{
    init_stable_cell(id, T::default())
}

pub(crate) fn save_static_string(
    cell: &'static LocalKey<RefCell<StableCell<String>>>,
    s: impl Into<Option<String>>,
) {
    let s = s.into();

    cell.with(|val| {
        val.borrow_mut()
            .set(s.unwrap_or_default())
            .expect("failed to save string")
    });
}

pub(crate) fn load_static_string(
    cell: &'static LocalKey<RefCell<StableCell<String>>>,
) -> Option<String> {
    let s = cell.with(|val| val.borrow().get().clone());

    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
