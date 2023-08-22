use std::cell::RefCell;
use std::thread::LocalKey;

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{Cell, DefaultMemoryImpl, Storable};

// IDs per variable in stable memory
pub(crate) const LAST_CONSENSUS_RPC_URL_ID: MemoryId = MemoryId::new(0);
pub(crate) const LAST_EXECUTION_RPC_URL_ID: MemoryId = MemoryId::new(1);
pub(crate) const LAST_CHECKPOINT_ID: MemoryId = MemoryId::new(2);
pub(crate) const LAST_NETWORK_ID: MemoryId = MemoryId::new(3);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub(crate) type StableCell<T> = RefCell<Cell<T, VirtualMemory<DefaultMemoryImpl>>>;

pub(crate) fn memory_of(id: MemoryId) -> VirtualMemory<DefaultMemoryImpl> {
    MEMORY_MANAGER.with(|mngr| mngr.borrow().get(id))
}

/// If memory of `id` is already initialized then that content will be used, otherwise it is
/// initialized with `value`.
pub(crate) fn init_stable_cell<T>(id: MemoryId, value: T) -> StableCell<T>
where
    T: Storable,
{
    let inner_cell = Cell::<T, VirtualMemory<DefaultMemoryImpl>>::init(memory_of(id), value)
        .expect("failed to initialize StableCell");
    RefCell::new(inner_cell)
}

/// If memory of `id` is already initialized then that content will be used, otherwise it is
/// initialized with `Default::default()` value.
pub(crate) fn init_stable_cell_default<T>(id: MemoryId) -> StableCell<T>
where
    T: Storable + Default,
{
    init_stable_cell(id, T::default())
}

pub trait StableStatic<T> {
    fn store(&'static self, val: impl Into<Option<T>>);
    fn load(&'static self) -> Option<T>;
}

impl<T> StableStatic<T> for LocalKey<StableCell<T>>
where
    T: Clone + Default + Storable + IsEmpty,
{
    fn store(&'static self, val: impl Into<Option<T>>) {
        let val = val.into();

        self.with(|cell| {
            cell.borrow_mut()
                .set(val.unwrap_or_default())
                .expect("failed to store value")
        });
    }

    fn load(&'static self) -> Option<T> {
        let val = self.with(|cell| cell.borrow().get().clone());

        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    }
}

trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl IsEmpty for String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }
}

impl IsEmpty for Vec<u8> {
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
}
