//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, current_user_token,TASK_MANAGER};
use crate::timer::get_time_us;
use crate::mm::{VirtAddr, MapPermission,PTEFlags};
use crate::mm::translated_byte_buffer;
use crate::config::PAGE_SIZE;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

// ///
// fn port_to_map_perm(port: usize) -> MapPermission {
//     let mut perm = MapPermission::empty();
//     if port & 0x1 != 0 {
//         perm |= MapPermission::R;
//     }
//     if port & 0x2 != 0 {
//         perm |= MapPermission::W;
//     }
//     if port & 0x4 != 0 {
//         perm |= MapPermission::X;
//     }
//     perm |= MapPermission::U;
//     perm
// }

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");   
    let us = get_time_us();
    let sec = us / 1_000_000;
    let usec = us % 1_000_000;

    let mut timeval_ptr = translated_byte_buffer(current_user_token(), ts as *const u8, core::mem::size_of::<TimeVal>());
    let timeval = unsafe { &mut *(timeval_ptr[0].as_mut_ptr() as *mut TimeVal) };
    timeval.sec = sec;
    timeval.usec = usec;

    0
    // let start_va = ts as usize;
    // let end_va = start_va + core::mem::size_of::<TimeVal>();
    // let start_vpn = VirtAddr::from(start_va).floor();
    // let end_vpn = VirtAddr::from(end_va-1).floor();

    // if start_vpn == end_vpn {
    //     if let Some(time_val) = translated_refmut::<TimeVal>(token, ts) {
    //     time_val.sec = sec;
    //     time_val.usec = usec;
    //     0
    //     } else {
    //         -1
    //     }
    // } else {
    //     let sec_ptr = unsafe{ &mut (*ts).sec as *mut usize };
    //     if let Some(sec_ref) = translated_refmut::<usize>(token, sec_ptr) {
    //         let usec_ptr = unsafe{ &mut (*ts).usec as *mut usize };
    //         if let Some(usec_ref) = translated_refmut::<usize>(token, usec_ptr) {
    //             *sec_ref = sec;
    //             *usec_ref = usec;
    //             0
    //         } else {
    //             -1
    //         }
    //     } else {
    //         -1
    //     }
    // }
    
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");

    TASK_MANAGER.set_nr_syscall(id);
    match trace_request {
        0 => {
            // read
            // check id
            println!("kernel: sys_trace read id = {:#x}", id);
            let token = current_user_token();
            let page_table = crate::mm::PageTable::from_token(token);
            let pe = page_table.translate(VirtAddr::from(id).floor());
            println!("kernel: sys_trace read id = {:#x}", id);
            match pe {
                Some(pte) => {
                    println!("kernel: sys_trace pte = {:?}", pte.flags());
                    if pte.flags().contains(PTEFlags::U) {
                        println!("kernel: PTEFlags::U");
                        let kernel_id = translated_byte_buffer(current_user_token(), id as *const u8, 1);
                        println!("kernel: sys_trace read id = {:#x}", id);
                        (unsafe { *(kernel_id[0].as_ptr()) }).into()
                    } else {
                        println!("kernel: NO PTEFlags::U");
                        return -1;
                    }
                },
                None => {
                    //println!("kernel: sys_trace read id = {:#x}", id);
                    return -1
                },
            }
        },
        1 => {
            // write
            //println!("kernel: sys_trace read id = {:#x}", id);
            let token = current_user_token();
            let page_table = crate::mm::PageTable::from_token(token);
            let pe = page_table.translate(VirtAddr::from(id).floor());
            //println!("kernel: sys_trace read id = {:#x}", id);
            match pe {
                Some(pte) => {
                    //println!("kernel: sys_trace pte = {:?}", pte.flags());
                    if !pte.flags().contains(PTEFlags::U) || !pte.flags().contains(PTEFlags::W) {
                        //println!("kernel: NO PTEFlags::U or PTEFlags::W");
                        -1
                    } else {
                        //println!("kernel: PTEFlags::U and PTEFlags::W");
                        let mut kernel_id = translated_byte_buffer(current_user_token(), id as *mut u8, 1);
                        (unsafe { *(kernel_id[0].as_mut_ptr()) = data as u8 });
                        0 
                    }
                },
                None => {
                    -1
                },
            }
        },
        2 => {
            // get the number of times syscall with _id has been called
            // YOUR JOB
            TASK_MANAGER.get_nr_syscall(id) as isize
        },
        _ => -1,
    }
    // use crate::mm::{PTEFlags,VirtAddr};
    // use crate::task::{current_memory_set};
    
    // match _trace_request {
    //     0 => {
    //         let va = VirtAddr::from(id);
    //         let vpn = va.floor();            
    //         let token = current_user_token();
    //         let user_ptr = id as *mut u8;
            
    //         let has_permission = {
    //             let memory_set = current_memory_set();
    //             let ms = memory_set.exclusive_access();
    //             if let Some(pte) = (*ms).translate(vpn) {
    //                 let flags = pte.flags();
    //                 if va.page_offset() == 0 {
    //                     flags.contains(PTEFlags::R)
    //                 } else {
    //                     flags.contains(PTEFlags::R) && (va.page_offset() + 1) <= PAGE_SIZE
    //                 }
    //             } else {
    //                 false
    //             }
    //         };
    //         if has_permission {
    //             if let Some(byte_ref) = translated_ref(token, user_ptr) {
    //                 *byte_ref as isize
    //             } else {
    //                 -1
    //             }
    //         } else {
    //             -1
    //         }
    //     }
    //     1 => {
    //         let va = VirtAddr::from(id);
    //         let vpn = va.floor();
    //         let token = current_user_token();
    //         let user_ptr = id as *mut u8;
    //         let has_permission = {
    //             let memory_set = current_memory_set();
    //             let ms = memory_set.exclusive_access();
    //             if let Some(pte) = (*ms).translate(vpn) {
    //                 let flags = pte.flags();
    //                 if va.page_offset() == 0 {
    //                     flags.contains(PTEFlags::W)
    //                 } else {
    //                     flags.contains(PTEFlags::W) && (va.page_offset() + 1) <= PAGE_SIZE
    //                 }
    //             } else {
    //                 false
    //             }
    //         };
    //         if has_permission {
    //             if let Some(byte_mut) = translated_refmut(token, user_ptr) {
    //                 *byte_mut = data as u8;
    //                 0
    //             } else {
    //                 -1
    //             }
    //         } else {
    //             -1
    //         }
    //     }
    //     2 => {
    //         TASK_MANAGER.get_current_syscall_count(id) as isize
    //     }
    //     _ => -1,
    // }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
let mut perm = crate::mm::MapPermission::empty();
    // check start alignment
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    // prot validity: only low 3 bits allowed and must not be zero
    if prot & !0x7 != 0 || prot & 0x7 == 0 {
        return -1;
    } else {
        // build permissions from prot bits: bit0 read, bit1 write, bit2 exec
        if prot & 0x1 != 0 { perm |= MapPermission::R; }
        if prot & 0x2 != 0 { perm |= MapPermission::W; }
        if prot & 0x4 != 0 { perm |= MapPermission::X; }
        // user bit usually needed for user mappings
        perm |= crate::mm::MapPermission::U;
    }

    // nothing to do
    if len == 0 {
        return 0;
    }
    // round up
    let length = (len + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;

    // --- Phase 1: detect overlap with already mapped pages ---
    let overlap = TASK_MANAGER.check_mmap_area(start.into(), (start + length).into());
    if overlap {
        return -1;
    }

    // --- Phase 2: allocate physical frames and map them ---
    TASK_MANAGER.add_mmap_area(
        VirtAddr::from(start),
        VirtAddr::from(start + len),
        perm,
    );
    // if _start % PAGE_SIZE != 0{
    //     return -1;
    // }
    // if _len == 0 {
    //     return 0;
    // }

    // let start_va = VirtAddr::from(_start);
    // let end_va = VirtAddr::from(_start + _len);
    // let start_vpn = start_va.floor();
    // let end_vpn = end_va.ceil();
    // let map_perm = port_to_map_perm(_port);
    
    // use crate::task::current_memory_set;
    // let memory_set = current_memory_set();
    // let mut ms = memory_set.exclusive_access();

    // for vpn in VPNRange::new(start_vpn, end_vpn) {
    //     if (*ms).translate(vpn).is_some(){
    //         return -1;
    //     }
    // }

    // let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
    // (*ms).push(map_area,None);

    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);
    TASK_MANAGER.remove_mmap_area(start_va, end_va)

    // if _start % PAGE_SIZE != 0 {
    //     return -1;
    // }
    // if _len == 0{
    //     return 0;
    // }

    // let start_va = VirtAddr::from(_start);
    // let end_va = VirtAddr::from(_start +_len);
    // let start_vpn = start_va.floor();
    // let end_vpn = end_va.ceil();

    // use crate::task::current_memory_set;
    // let memory_set = current_memory_set();
    // let mut ms  = memory_set.exclusive_access();

    // for vpn in VPNRange::new(start_vpn, end_vpn) {
    //     if (*ms).translate(vpn).is_none(){
    //         return -1;
    //     }
    // }

    // (*ms).unmap_range(start_vpn, end_vpn);
    // 0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
