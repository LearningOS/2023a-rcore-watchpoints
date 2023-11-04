//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::MAX_SYSCALL_NUM,
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus, set_current_task_priority,
    },
};
use crate::mm::get_physical;
use crate::timer::get_time_us;
use crate::task::get_task_info;
use crate::task::current_task_map;
use crate::task::current_task_unmap;
use crate::task::TaskControlBlock;

#[repr(C)]
#[derive(Debug)]
/// Time value
pub struct TimeVal {
    ///秒
    pub sec: usize,
    ///微秒
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
   pub status: TaskStatus,
    /// The numbers of syscall called by task
   pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
   pub time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}
/// sys_getpid
pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

/// sys_fork
pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}
/// sys_exec
pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    // trace!(
    //     "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
    //     current_task().unwrap().pid.0
    // );
    // -1
    trace!(
        "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let us = get_time_us();
    let physical_address = get_physical(current_user_token(), _ts as usize);
    let ptr = physical_address as *mut TimeVal;
    unsafe {
        *ptr = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
    
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!(
        "kernel:pid[{}] sys_task_info",
        current_task().unwrap().pid.0
    );

    let physical_address = get_physical(current_user_token(), _ti as usize);
    let ptr = physical_address as *mut TaskInfo;
    unsafe{
        *ptr = get_task_info();
    }
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );

    if (_port & !0x7) != 0 || (_port & 0x7) == 0 {
        return -1
    }
    current_task_map(_start, _len, _port)
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );

    current_task_unmap(_start, _len)
    
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8) -> isize {
    //思路：创建第一个用户态进程 initproc 怎么创建的
    //https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter5/3implement-process-mechanism.html
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    
    let token = current_user_token();
    let path = translated_str(token, _path); //用户态的字符串变成内核态的字符串
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        // reuse TaskControlBlock::new()
        let task = Arc::new(TaskControlBlock::new(data)); 


        let parent = current_task().unwrap();  //cpu 正在执行的进程状态，1号进程
        // /// A vector containing TCBs of all child processes of the current process
        // TaskControlBlockInner pub children: Vec<Arc<TaskControlBlock>>,
        parent.inner_exclusive_access().children.push(task.clone());

        task.inner_exclusive_access().parent = Some(Arc::downgrade(&parent));
        // C++ 智能指针详解（二）——shared_ptr与weak_ptr

        ///// Parent process of the current process.
        // /// Weak will not affect the reference count of the parent
        // pub parent: Option<Weak<TaskControlBlock>>, //c++ Weak

        let pid = task.pid.0 as isize;
        // add task
        add_task(task);
        pid 
        // 父进程创建子进程 并且返回子进程id
    } else {
        -1
    }
}
// /// sys_fork
// pub fn sys_fork() -> isize {
//     trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
//     let current_task = current_task().unwrap();
//     let new_task = current_task.fork();
//     let new_pid = new_task.pid.0;
//     // modify trap context of new_task, because it returns immediately after switching
//     let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
//     // we do not have to move to next instruction since we have done it before
//     // for child process, fork returns 0
//     trap_cx.x[10] = 0;
//     // add new task to scheduler
//     add_task(new_task);
//     new_pid as isize
// }
// /// sys_exec
// pub fn sys_exec(path: *const u8) -> isize {
//     trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
//     let token = current_user_token();
//     let path = translated_str(token, path);
//     if let Some(data) = get_app_data_by_name(path.as_str()) {
//         let task = current_task().unwrap();
//         task.exec(data);
//         0
//     } else {
//         -1
//     }
// }

/// YOUR JOB: Set task priority.
/// 返回优先级 那个进程的
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    if _prio <= 1 {
        -1
    } else {
        set_current_task_priority(_prio as usize);
        _prio
    }

}

// syscall ID：140
// 设置当前进程优先级为 prio
// 参数：prio 进程优先级，要求 prio >= 2
// 返回值：如果输入合法则返回 prio，否则返回 -1
