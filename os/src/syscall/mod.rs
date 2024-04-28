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

//进程管理相关
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_GETPPID: usize = 173;

//文件相关
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_GETCWD: usize = 17;

mod fs;
mod process;

pub use fs::*;
use process::*;
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    //info!("Reach sys_call");
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_OPEN => sys_open(args[0] as *const u8, args[1] as u32),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE => sys_pipe(args[0] as *mut usize),
        SYSCALL_GETCWD => sys_getcwd(args[0] as *mut u8, args[1]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
