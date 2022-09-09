// memory management and data access
use core::mem::MaybeUninit;
use heapless::pool::{singleton::arc::ArcInner, Node};
use heapless::{arc_pool, Arc, Vec};
use spin::{Lazy, Mutex};

// library specific
use crate::grain::Grain;
use crate::manager::MAX_GRAINS;

static MOCK_ARRAY: [f32; 48] = [1.0_f32; 48];

// ==========================
// INITIALZATION OF SINGLETON
// ==========================

#[derive(Debug)]
pub struct Grains<'a> {
    pub grains: Mutex<Vec<Grain<'a>, MAX_GRAINS>>,
}

// figure out memory size for MAX_GRAINS
static mut MEMORY_SIZE: MaybeUninit<[Node<ArcInner<Grains>>; 1]> = MaybeUninit::uninit();

// create pool type
arc_pool!(GrainPool: Grains<'static>);

// grow the pool
static GRAIN_POOL: Lazy<Arc<GrainPool>> = Lazy::new(|| {
    GrainPool::grow_exact(unsafe { &mut MEMORY_SIZE });

    GrainPool::alloc(Grains {
        grains: Mutex::new(Vec::<Grain, MAX_GRAINS>::new()),
    })
    .ok()
    .expect("Out of Memory!")
});

impl<'a> Grains<'a> {
    pub fn get_instance() -> Arc<GrainPool> {
        GRAIN_POOL.clone()
    }
}

// =============
// SINGLETON API
// =============

pub fn push_grain<'a>(id: usize) -> Result<(), Grain<'a>> {
    Grains::get_instance()
        .grains
        .lock()
        .push(Grain::new(id, &MOCK_ARRAY[..]))
}

pub fn remove_grain<'a>(id: usize) -> Result<Grain<'a>, usize> {
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

pub fn get_grain<'a>(id: usize) -> Result<Grain<'a>, usize> {
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

pub fn update_source_samples() {
    let singleton = Grains::get_instance();
    let mut grains = singleton.grains.lock();

    for position in 0..grains.len() {
        grains.get_mut(position).unwrap().update_source_sample();
    }
}

pub fn get_next_sample() -> f32 {
    let singleton = Grains::get_instance();
    let mut grains = singleton.grains.lock();

    let mut sample = 0.0_f32;
    let num_of_grains = grains.len();

    for position in 0..num_of_grains {
        sample += grains.get_mut(position).unwrap().get_next_sample();
    }

    sample
}
