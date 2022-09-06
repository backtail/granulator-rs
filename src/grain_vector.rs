// memory management and data access
use core::mem::MaybeUninit;
use heapless::pool::{singleton::arc::ArcInner, Node};
use heapless::{arc_pool, Arc, Vec};
use spin::{Lazy, Mutex};

// library specific
use crate::grain::Grain;

// ==========================
// INITIALZATION OF SINGLETON
// ==========================

// max amount of simultanious grains
pub const MAX_GRAINS: usize = 64;

// singleton
#[derive(Debug)]
pub struct Grains {
    pub grains: Mutex<Vec<Grain, MAX_GRAINS>>,
}

// figure out memory size for MAX_GRAINS
static mut MEMORY_SIZE: MaybeUninit<[Node<ArcInner<Grains>>; 1]> = MaybeUninit::uninit();

// create pool type
arc_pool!(GrainPool: Grains);

// grow the pool
static GRAIN_POOL: Lazy<Arc<GrainPool>> = Lazy::new(|| {
    GrainPool::grow_exact(unsafe { &mut MEMORY_SIZE });

    GrainPool::alloc(Grains {
        grains: Mutex::new(Vec::<Grain, MAX_GRAINS>::new()),
    })
    .ok()
    .expect("Out of Memory!")
});

impl Grains {
    pub fn get_instance() -> Arc<GrainPool> {
        GRAIN_POOL.clone()
    }
}

// =============
// SINGLETON API
// =============

pub fn push_grain(id: usize) -> Result<(), Grain> {
    Grains::get_instance().grains.lock().push(Grain::new(id))
}

pub fn remove_grain(id: usize) -> Result<Grain, ()> {
    let singleton = Grains::get_instance();
    let mut grains = singleton.grains.lock();

    for position in 0..grains.len() {
        if grains.get(position).unwrap().id == id {
            return Ok(grains.swap_remove(position));
        }
    }

    // when no element has been removed return an Err
    Err(())
}
