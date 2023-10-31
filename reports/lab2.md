
### 实验要求
实现分支：ch4。

实现 mmap 和 munmap 两个系统调用，通过所有测例。

实验目录请参考 ch3，报告命名 lab2.md/pdf

TIPS：注意 port 参数的语义，它与内核定义的 MapPermission 有明显不同
~~~
/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    -1
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
~~~
### 查询资料学习

- Concise Manual: [rCore-Tutorial-Guide-2023A](https://LearningOS.github.io/rCore-Tutorial-Guide-2023A/)

- Detail Book [rCore-Tutorial-Book-v3](https://rcore-os.github.io/rCore-Tutorial-Book-v3/)

-https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/2address-space.html
 
 - http://learningos.cn/uCore-Tutorial-Guide-2023S/chapter4/index.html
 
 
 地址空间 (Address Space) 
  虚拟地址 (Virtual Address)

![](https://files.mdnice.com/user/5197/429e84aa-f787-4220-b699-9ce9a5fc83ad.png)

我们知道应用的数据终归还是存在物理内存中的，那么虚拟地址如何形成地址空间，虚拟地址空间如何转换为物理内存呢？操作系统可以设计巧妙的数据结构来表示地址空间。但如果完全由操作系统来完成转换每次处理器地址访问所需的虚实地址转换，那开销就太大了。这就需要扩展硬件功能来加速地址转换过程
- 分段内存管理

![](https://files.mdnice.com/user/5197/964e6ac9-69bd-4114-b9af-b3603f542468.png)
- 分页内存管理

![](https://files.mdnice.com/user/5197/4f930fcb-75f9-44e2-9fce-f6c8837880af.png)

- 实现 SV39 多级页表机制（上）

http://learningos.cn/rCore-Tutorial-Guide-2023A/chapter4/3sv39-implementation-1.html


![](https://files.mdnice.com/user/5197/a020b05d-4385-4c8b-a67a-b3c485f507af.png)

基于上述智能指针，可形成更强大的 集合 (Collection) 或称 容器 (Container) 类型，它们负责管理一组数目可变的元素，这些元素的类型相同或是有着一些同样的特征。在 C++/Python/Java 等高级语言中我们已经对它们的使用方法非常熟悉了，对于 Rust 而言，我们可以直接使用以下容器：

向量 Vec<T> 类似于 C++ 中的 std::vector ；

键值对容器 BTreeMap<K, V> 类似于 C++ 中的 std::map ；

有序集合 BTreeSet<T> 类似于 C++ 中的 std::set ；

链表 LinkedList<T> 类似于 C++ 中的 std::list ；

双端队列 VecDeque<T> 类似于 C++ 中的 std::deque 。

变长字符串 String 类似于 C++ 中的 std::string 
  
  
  ~~~
  /// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel: sys_write");
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

  ~~~

  ~~~
  / The task control block (TCB) of a task.
pub struct TaskControlBlock {
  ~~~
  
#####  提问4 地址空间疑问
 - what 
  单元测试：ch3_sleep1.rs  执行get_time()函数
  参数是个地址,这就是吧用户空间地址，传递给内核
  
  分页模式开启之后
  CPU先拿到虚存地址，需要通过 MMU 的地址转换变成物理地址，再交给 CPU 的访存单元去访问物理内存。？
  
  
    https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/5kernel-app-spaces.html
~~~
  pub fn get_time() -> isize {
    let time = TimeVal::new();
    match sys_get_time(&time, 0) {
        0 => ((time.sec & 0xffff) * 1000 + time.usec / 1000) as isize,
        _ => -1,
    }
}
~~~
  
    内核地址空间
在本章之前，内核和应用代码的访存地址都被视为一个物理地址，并直接访问物理内存，而在分页模式开启之后，CPU先拿到虚存地址，需要通过 MMU 的地址转换变成物理地址，再交给 CPU 的访存单元去访问物理内存。
  
  地址空间抽象的重要意义在于 隔离 (Isolation) ，当内核让应用执行前，内核需要控制 MMU 使用这个应用的多级页表进行地址转换。由于每个应用地址空间在创建的时候也顺带设置好了多级页表，使得只有那些存放了它的代码和数据的物理页帧能够通过该多级页表被映射到，这样它就只能访问自己的代码和数据而无法触及其他应用或内核的内容。
  
  内核如何访问应用的数据？

应用应该不能直接访问内核的数据，但内核可以访问应用的数据，这是如何做的？由于内核要管理应用，所以它负责构建自身和其他应用的多级页表。如果内核获得了一个应用数据的虚地址，内核就可以通过查询应用的页表来把应用的虚地址转换为物理地址，内核直接访问这个地址（注：内核自身的虚实映射是恒等映射），就可以获得应用数据的内容了

  
  
  

### 单元测试

cd ci-user && make test CHAPTER=4

-程序直接卡主了 why？


  ~~~
  Panicked at src/bin/ch3_taskinfo.rs:19, assertion `left == right` failed

  ~~~

ch4_mmap0.rs
  
  

### 遇到的问题


### q4 使用国内镜像加速 Rust 更新与下载
- what 
~~~
info: syncing channel updates for 'nightly-x86_64-unknown-linux-gnu'
error: could not download file from 'https://static.rust-lang.org/dist/channel-rust-nightly.toml.sha256' to '/home/wangchuanyi/.rustup/tmp/mqrirb62
~~~

- how
https://rsproxy.cn/#home


~~~
配置说明
步骤一：设置 Rustup 镜像， 修改配置 ~/.zshrc or ~/.bashrc
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

步骤二：安装 Rust（请先完成步骤一的环境变量导入并 source rc 文件或重启终端生效）
curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh

步骤三：设置 crates.io 镜像， 修改配置 ~/.cargo/config，已支持git协议和sparse协议，>=1.68 版本建议使用 sparse-index，速度更快。
sparse
rsproxy
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
[net]
git-fetch-with-cli = true
~~~

  
  
- 看不懂 看文档 https://wyfcyx.github.io/rCore_tutorial_doc/chapter5/part1.html
  
  
  页表：从虚拟内存到物理内存
  小王同学:
一个页表项 (PTE, Page Table Entry)是用来描述一个虚拟页号如何映射到物理页号的。如果一个虚拟页号通过某种手段找到了一个页表项，并通过读取上面的物理页号完成映射，我们称这个虚拟页号通过该页表项完成映射。 

小王同学:
这个理解不了，里面只存储了 物理页号和flag 根本没存虚拟页号，怎么关联呢？

  
  一个虚拟页号就对应找到一个页表项
  
  可以把整个页表看成一个数组，物理地址就是里面的值，虚拟地址就是索引
  
  对 索引这块没看懂
  
![](https://files.mdnice.com/user/5197/b768f252-5504-4ad8-9a66-eb2c8fed39f1.png)

  也可以说虚拟地址就是这个数组的下标
  
  
  看这个图应该清晰一点
  
  计算机组成原理：简单页表和多级页表（虚拟内存的映射）
  
  
![](https://files.mdnice.com/user/5197/edb196a8-3600-4cc9-95fc-a0781a3352b5.png)
  
计算机组成原理：简单页表和多级页表（虚拟内存的映射）
总结一下，对于一个内存地址转换，其实就是这样三个步骤：

把虚拟内存地址，切分成页号和偏移量的组合；
从页表里面，查询出虚拟页号，对应的物理页号；
直接拿物理页号，加上前面的偏移量，就得到了物理内存地址
  
  
![](https://files.mdnice.com/user/5197/66f2d611-9075-4648-b0fc-ed64fcd8870a.png)

  
  
![](https://files.mdnice.com/user/5197/6fb15a65-cb62-4eb0-bc7f-4e4985d3bb4e.png)


![](https://files.mdnice.com/user/5197/0a7a80d8-b2ad-4533-b092-a26879ce9e87.png)

  
  ~~~
  /// physical address
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);
/// virtual address
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);
/// physical page number
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);
/// virtual page number
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);
  ~~~
  
  
  
  ~~~
  bitflags! {
    /// map permission corresponding to that in pte: `R W X U`
    pub struct MapPermission: u8 {
        ///Readable
        const R = 1 << 1;
        ///Writable
        const W = 1 << 2;
        ///Excutable
        const X = 1 << 3;
        ///Accessible in U mode
        const U = 1 << 4;
    }
}

  
  bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7; //通常用于表示页面是否已被修改，以支持页面置换策略
    }
}
  ~~~
### Q3： 下载资源失败


echo nameserver 8.8.8.8 > /etc/resolv.conf
 
echo nameserver 114.114.114.114 > /etc/resolv.conf

### Q2  make install 失败  🆗


export PATH=$PATH:/home/wangchuanyi/code/qemu-7.0.0/build

#### Q1 处理could not resolve host: github.com问题 ？ 🆗


ping github.com
 /etc/hosts

