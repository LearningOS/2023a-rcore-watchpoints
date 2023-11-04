进程创建

大家一定好奇过为啥进程创建要用 fork + exec 这么一个奇怪的系统调用，就不能直接搞一个新进程吗？ 思而不学则殆，我们就来试一试！

这章的编程练习请大家实现一个完全 DIY 的系统调用 spawn，用以创建一个新进程。

https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter5/index.html


# 第五章：进程及进程管理

实验要求
实现分支：ch5。

实验目录请参考 ch3。注意在reports中放入lab1-3的所有报告。

通过所有测例。

在 os 目录下 make run BASE=2 加载所有测例， ch5_usertest 打包了所有你需要通过的测例， 你也可以通过修改这个文件调整本地测试的内容, 或者单独运行某测例来纠正特定的错误。 ch5_stride 检查 stride 调度算法是否满足公平性要求，六个子程序运行的次数应该大致与其优先级呈正比，测试通过标准是 
 
 
.

CI 的原理是用 ch5_usertest 替代 ch5b_initproc ，使内核在所有测例执行完后直接退出。




## 阅读文档：
- https://learningos.cn/rCore-Tutorial-Guide-2023A/chapter5/index.html



任务和进程的关系与区别

第三章提到的 任务 是这里提到的 进程 的初级阶段，与任务相比，进程能在运行中创建 子进程 、 用新的 程序 内容覆盖已有的 程序 内容、可管理更多物理或虚拟 资源 。


/bin/sh: 1: rustup: not found
/bin/sh: 1: rustup: not found

- 重新 拉去工程


![](https://files.mdnice.com/user/5197/3976dc89-def5-46e3-8b2b-907bfa5ddb9a.png)




进程管理的核心数据结构
本节导读
为了更好实现进程管理，我们需要设计和调整内核中的一些数据结构，包括：

基于应用名的应用链接/加载器

进程标识符 PidHandle 以及内核栈 KernelStack

任务控制块 TaskControlBlock

任务管理器 TaskManager

处理器管理结构 Processo

进程管理机制的设计实现
本节导读
本节将介绍如何基于上一节设计的内核数据结构来实现进程管理：

初始进程 initproc 的创建；

进程调度机制：当进程主动调用 sys_yield 交出 CPU 使用权，或者内核本轮分配的时间片用尽之后如何切换到下一个进程；

进程生成机制：介绍进程相关的两个重要系统调用 sys_fork/sys_exec 的实现；

字符输入机制：介绍 sys_read 系统调用的实现；

进程资源回收机制：当进程调用 sys_exit 正常退出或者出错被内核终止后，如何保存其退出码，其父进程又是如何通过 sys_waitpid 收集该进程的信息并回收其资源。


### 


### Code
- [Soure Code of labs for 2023A](https://github.com/LearningOS/rCore-Tutorial-Code-2023A)
### Documents

- Concise Manual: [rCore-Tutorial-Guide-2023A](https://LearningOS.github.io/rCore-Tutorial-Guide-2023A/)

- Detail Book [rCore-Tutorial-Book-v3](https://rcore-os.github.io/rCore-Tutorial-Book-v3/)


### OS API docs of rCore Tutorial Code 2023A
- [OS API docs of ch1](https://learningos.github.io/rCore-Tutorial-Code-2023A/ch1/os/index.html)
  AND [OS API docs of ch2](https://learningos.github.io/rCore-Tutorial-Code-2023A/ch2/os/index.html)
- [OS API docs of ch3](https://learningos.github.io/rCore-Tutorial-Code-2023A/ch3/os/index.html)
  AND [OS API docs of ch4](https://learningos.github.io/rCore-Tutorial-Code-2023A/ch4/os/index.html)
- [OS API docs of ch5](https://learningos.github.io/rCore-Tutorial-Code-2023A/ch5/os/index.html)
  AND [OS API docs of ch6](https://learningos.github.io/rCore-Tutorial-Code-2023A/ch6/os/index.html)
- [OS API docs of ch7](https://learningos.github.io/rCore-Tutorial-Code-2023A/ch7/os/index.html)
  AND [OS API docs of ch8](https://learningos.github.io/rCore-Tutorial-Code-2023A/ch8/os/index.html)
- [OS API docs of ch9](https://learningos.github.io/rCore-Tutorial-Code-2023A/ch9/os/index.html)

### Related Resources
- [Learning Resource](https://github.com/LearningOS/rust-based-os-comp2022/blob/main/relatedinfo.md)






https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter5/index.html


https://github.com/LearningOS/2023a-rcore-Suzukaze7/blob/ch4/os/src/syscall/process.rs

## 作业任务


~~~
/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!(
        "kernel:pid[{}] sys_task_info NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
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
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}
~~~



函数原型：

pub fn sys_munmap(_start: usize, _len: usize) -> isize 



函数原型：

current_task_unmap(_start, _len)


思路：



### let cpid = spawn("ch5_exit0\0");



流程图：


用户： spawn("ch5_exit0\0");

syscall:   SYSCALL_SPAWN => sys_spawn(args[0] as *const u8),

//参考：创建第一个用户态进程 initproc 怎么创建的
//https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter5/3implement-process-mechanism.html  
~~~

lazy_static! {
    /// Creation of initial process
    ///
    /// the name "initproc" may be changed to any other app name like "usertests",
    /// but we have user_shell, so we don't need to change it.
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("ch5b_initproc").unwrap()
    ));
}
  
~~~

# setup build&run environment first
$ git clone https://github.com/LearningOS/rCore-Tutorial-Code-2023A.git
$ cd rCore-Tutorial-Code-2023A
$ git clone https://github.com/LearningOS/rCore-Tutorial-Checker-2023A.git ci-user
$ git clone https://github.com/LearningOS/rCore-Tutorial-Test-2023A.git ci-user/user
$ cd ci-user && make test CHAPTER=$ID

实验要求
完成分支: ch3-lab

实验目录要求不变

通过所有测例

lab3 有 3 类测例，在 os 目录下执行 make run TEST=1 检查基本 sys_write 安全检查的实现， make run TEST=2 检查 set_priority 语义的正确性， make run TEST=3 检查 stride 调度算法是否满足公平性要求， 六个子程序运行的次数应该大致与其优先级呈正比，测试通过标准是


### 第三次测试 报错

~~~
/*
理想结果：对于错误的 mmap 返回 -1，最终输出 Test 04_4 test OK!
*/

#[no_mangle]
fn main() -> i32 {
    let start: usize = 0x10000000;
    let len: usize = 4096;
    let prot: usize = 3; //?????
    assert_eq!(0, mmap(start, len, prot));
    assert_eq!(mmap(start - len, len + 1, prot), -1);
    assert_eq!(mmap(start + len + 1, len, prot), -1);
    assert_eq!(mmap(start + len, len, 0), -1);
    assert_eq!(mmap(start + len, len, prot | 8), -1);
    println!("Test 04_4 test OK!");
    0
}
~~~

`
