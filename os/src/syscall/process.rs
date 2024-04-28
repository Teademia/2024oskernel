use crate::fat;
use crate::mm::{translated_refmut, translated_str};
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next,
};
use crate::timer::get_time_ms;
use alloc::sync::Arc;
pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
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

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(file) = unsafe { fat::ROOT_INODE.find_inode(path.as_str()) } {
        let task = current_task().unwrap();
        task.exec(unsafe { file.read_as_elf() });
        0
    } else {
        -1
    }
}

//传入值的pid如果是-1则代表清理任意进程
//如果不是-1,则代表等待制定的进程退出

//返回值-1代表进程不存在,-2表示进程还在运行不能返回
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();

    //解释一下下面的这个段
    //当pid不等于-1
    //而且找不到制定要wait的进程,就会return -1告诉user你指定的进程不存在

    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
    }

    //现在是已经确定pid==-1或者找到了对应的进程,所以我们要考虑的就是如何回收这个进程
    //将child全都遍历一边,找到
    //是僵尸进程 而且 pid和指定的值一样的进程
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
    });

    if let Some((idx, _)) = pair {
        //先得到要清理的子进程的TCB
        let child = inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        let exit_code = child.inner_exclusive_access().exit_code;
        //将exit_code保存到对应的地址
        if (exit_code_ptr as usize != 0) {
            *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        }
        found_pid as isize
    } else {
        //进程还在运行不能退
        -2
    }
}
