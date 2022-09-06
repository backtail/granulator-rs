use spin::Mutex;

static INDEX: Mutex<usize> = Mutex::new(0);

pub fn get_new_index() -> usize {
    let mut locked_index = INDEX.lock();
    let return_index = locked_index.clone();
    *locked_index += 1;

    return_index
}
