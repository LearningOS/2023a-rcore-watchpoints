//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use crate::task::TaskStatus;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    /// 从空闲任务中选择一个任务进行调度
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // self.ready_queue.pop_front()

        // 每次需要调度时，从当前 runnable 态的进程中选择 stride 最小的进程调度
        let mut min_task_stride = self.ready_queue[0].inner_exclusive_access().stride; //假设第一个最小
        let mut min_index = 0;

        //enumerate会让迭代器回传目前是第几次的迭代
        for (index, task) in self.ready_queue.iter().enumerate() {
            let inner = task.inner_exclusive_access();
            if inner.task_status == TaskStatus::Ready {
                if inner.stride < min_task_stride {
                    min_index = index;
                    min_task_stride = inner.stride;
                }
            }
        }

        let min_task = self.ready_queue.remove(min_index);
        // let mut inner = min_task.as_ref().inner_exclusive_access();
        // // 可以证明，如果令 P.pass = BigStride / P.priority   12是我随机写数值
        // inner.stride += 12;
        // drop(inner);
        min_task  
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
