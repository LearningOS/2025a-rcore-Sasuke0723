//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// sbrk syscall
const SYSCALL_SBRK: usize = 214;
/// munmap syscall
const SYSCALL_MUNMAP: usize = 215;
/// mmap syscall
const SYSCALL_MMAP: usize = 222;
/// trace syscall
const SYSCALL_TRACE: usize = 410;

mod fs;
mod process;

use fs::*;
use process::*;
use crate::mm::{translated_byte_buffer};
// use crate::task::TASK_MANAGER;

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
     crate::task::set_nr_syscall(syscall_id);
    // TASK_MANAGER.increase_current_syscall_count(syscall_id);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_SBRK => sys_sbrk(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}


pub fn translated_ref<T>(token: usize, ptr: *const T) -> Option<&'static T> {
    let ptr =ptr as * const u8;
    let len = core::mem::size_of::<T>();
    let buffers = translated_byte_buffer(token, ptr, len);

    if buffers.len() != 1 || buffers[0].len() != len {
        return None;
    }
    Some(unsafe { &*(buffers[0].as_ptr() as *const T) })
}

pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> Option<&'static mut T> {
    let ptr =ptr as * mut u8;
    let len = core::mem::size_of::<T>();
    let mut buffers = translated_byte_buffer(token, ptr, len);

    if buffers.len() != 1 || buffers[0].len() != len {
        return None;
    }
    Some(unsafe { &mut *(buffers[0].as_mut_ptr() as *mut T) })
}