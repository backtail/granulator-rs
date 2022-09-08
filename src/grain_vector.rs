// memory management and data access
use core::mem::MaybeUninit;
use heapless::pool::{singleton::arc::ArcInner, Node};
use heapless::{arc_pool, Arc, Vec};
use spin::{Lazy, Mutex};

// library specific
use crate::grain::Grain;
use crate::manager::MAX_GRAINS;
use crate::source::Source;
use crate::window_function::WindowFunction;

// ==========================
// INITIALZATION OF SINGLETON
// ==========================

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

pub fn remove_grain(id: usize) -> Result<Grain, usize> {
    let singleton = Grains::get_instance();
    let mut grains = singleton.grains.lock();

    for position in 0..grains.len() {
        let real_grain = grains.get(position);
        if real_grain.is_some() {
            if real_grain.unwrap().id == id {
                return Ok(grains.remove(position));
            }
        }
    }

    // when no element has been removed return an Err
    Err(id)
}

pub fn get_grain(id: usize) -> Result<Grain, usize> {
    let singleton = Grains::get_instance();
    let grains = singleton.grains.lock();

    for position in 0..grains.len() {
        let current_grain = grains.get(position).unwrap();
        if current_grain.id == id {
            return Ok(*current_grain);
        }
    }

    // when no element has been found
    Err(id)
}

pub fn flush_grains() {
    Grains::get_instance().grains.lock().clear();
}

pub fn is_finished(id: usize) -> Result<bool, usize> {
    let singleton = Grains::get_instance();
    let grains = singleton.grains.lock();

    for position in 0..grains.len() {
        let current_grain = grains.get(position).unwrap();
        if current_grain.id == id {
            return Ok(current_grain.finished);
        }
    }

    // when no element has been found
    Err(id)
}

pub fn update_envolopes() {
    let singleton = Grains::get_instance();
    let mut grains = singleton.grains.lock();

    for position in 0..grains.len() {
        grains.get_mut(position).unwrap().update_envelope();
    }
}

pub fn set_grain_parameters(
    id: usize,
    size_in_ms: f32,
    window: WindowFunction,
    window_paramater: Option<f32>,
    source: Source,
    source_length: Option<usize>,
    source_offset: Option<usize>,
) -> Result<(), usize> {
    let singleton = Grains::get_instance();
    let mut grains = singleton.grains.lock();

    for position in 0..grains.len() {
        let current_grain = grains.get_mut(position).unwrap();
        if current_grain.id == id {
            current_grain.set_grain_size(size_in_ms);
            current_grain.window = window;
            current_grain.window_parameter = window_paramater;
            current_grain.source = Some(source);
            current_grain.source_length = source_length;
            current_grain.source_offset = source_offset;
            return Ok(());
        }
    }

    // when no element has been found
    Err(id)
}
