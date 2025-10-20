//! Process management syscalls
use crate::{
    task::{TASK_MANAGER,exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// TODO: implement the syscall
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request{
	0 => trace_read_byte(_id) ,
	1 => trace_write_byte(_id,_data),
	2 => trace_get_syscall_count(_id),
	_ => -1,
    }
}

fn trace_read_byte(addr:usize) -> isize {
    let ptr = addr as *const u8;
    unsafe {*ptr as isize}
}

fn trace_write_byte(addr: usize, data: usize) -> isize{
    let ptr = addr as *mut u8;
    let value = (data & 0xFF) as u8;
    unsafe{
	*ptr = value;
    }
    0
}

fn trace_get_syscall_count(syscall_id: usize) -> isize {
    TASK_MANAGER.get_current_syscall_count(syscall_id) as isize
}
