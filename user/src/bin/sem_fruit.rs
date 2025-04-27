#![no_std]
#![no_main]

use alloc::vec::Vec;
use user_lib::{exit, semaphore_create, semaphore_down, semaphore_up, thread_create, waittid};

#[macro_use]
extern crate user_lib;
extern crate alloc;

const SEM_MUTEX: usize = 0;
const SEM_EMPTY: usize = 1;
const SEM_APPLE: usize = 2;
const SEM_ORANGE: usize = 3;
const BUFFER_SIZE: usize = 3;
static mut BUFFER: [usize; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut FRONT: usize = 0;
static mut TAIL: usize = 0;
const PRODUCER_COUNT: usize = 2;
const NUMBER_PER_PRODUCER: usize = 10;

fn producer(id: *const usize) -> ! {
    unsafe {
        let id = *id;
        for _ in 0..NUMBER_PER_PRODUCER {
            semaphore_down(SEM_EMPTY);
            semaphore_down(SEM_MUTEX);
            BUFFER[TAIL] = id;
            TAIL = (TAIL + 1) % BUFFER_SIZE;
            match id {
                0 => println!("父亲放入了 苹果"),
                1 => println!("父亲放入了 橘子"),
                _ => println!("错误! id={}", id),
            }
            semaphore_up(SEM_MUTEX);
            semaphore_up(id + SEM_APPLE);
        }
        exit(0)
    }
}

fn consumer(id: usize) -> ! {
    unsafe {
        for _ in 0..NUMBER_PER_PRODUCER {
            semaphore_down(id + SEM_APPLE);
            semaphore_down(SEM_MUTEX);
            match BUFFER[FRONT] {
                0 => println!("儿子取出了 苹果 "),
                1 => println!("女儿取出了 橘子 "),
                _ => println!("错误! id={}", BUFFER[FRONT]),
            }
            FRONT = (FRONT + 1) % BUFFER_SIZE;
            semaphore_up(SEM_MUTEX);
            semaphore_up(SEM_EMPTY);
        }
        exit(0)
    }
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    // create semaphores
    assert_eq!(semaphore_create(1) as usize, SEM_MUTEX);
    assert_eq!(semaphore_create(BUFFER_SIZE) as usize, SEM_EMPTY);
    assert_eq!(semaphore_create(0) as usize, SEM_APPLE);
    assert_eq!(semaphore_create(0) as usize, SEM_ORANGE);
    // create threads
    let ids: Vec<_> = (0..PRODUCER_COUNT).collect();
    let mut threads = Vec::new();
    for i in 0..PRODUCER_COUNT {
        threads.push(thread_create(
            producer as usize,
            &ids.as_slice()[i] as *const _ as usize,
        ));
    }
    threads.push(thread_create(consumer as usize, 0));
    threads.push(thread_create(consumer as usize, 1));
    // wait for all threads to complete
    for thread in threads.iter() {
        waittid(*thread as usize);
    }
    println!("sem_fruit passed!");
    0
}
