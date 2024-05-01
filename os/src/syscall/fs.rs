//! File and filesystem-related syscalls
use core::ptr::drop_in_place;
use core::slice;
use core::str;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use fat32_fs::sync_all;
use fat32_fs::ATTRIBUTE_ARCHIVE;

use crate::fat::*;
use crate::mm::*;
use crate::sbi::console_getchar;
use crate::task::current_task;
use crate::task::{current_user_token, suspend_current_and_run_next};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(
            current_user_token(),
            buf,
            len,
        ))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    /*
    如果 flags 为 0，则表示以只读模式 RDONLY 打开；
    如果 flags 第 0 位被设置（0x001），表示以只写模式 WRONLY 打开；
    如果 flags 第 1 位被设置（0x002），表示既可读又可写 RDWR ；
    如果 flags 第 9 位被设置（0x200），表示允许创建文件 CREATE ，在找不到该文件的时候应创建文件；如果该文件已经存在则应该将该文件的大小归零；
    如果 flags 第 10 位被设置（0x400），则在打开文件的时候应该清空文件的内容并将该文件的大小归零，也即 TRUNC 。 */
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        sync_all();
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        info!("Return FileHandle {}", fd);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { (pipe as *mut i32).add(1) }) = write_fd as i32;
    0
}

pub fn sys_getcwd(buf: *mut u8, len: usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let inner = task.inner_exclusive_access();

    if (buf != core::ptr::null_mut()) {
        let cwd_bytes = inner.cwd.as_bytes();
        let mut buffer = Vec::with_capacity(cwd_bytes.len());
        for &byte in cwd_bytes {
            buffer.push(byte);
        }
        let buffer_ptr = buffer.as_mut_ptr();
        for i in 0..cwd_bytes.len() {
            unsafe {
                *translated_refmut(token, buf.add(i)) = cwd_bytes[i];
            }
        }
    } else {
        return 0;
    }
    buf as isize
}

/*
cmod
第一位
RDONLY 0x000
WRONLY 0x001
RDWR 0x002

第二位
CREATE 0x40

第三位
DIRECTORY 0x0200000
DIR 0x040000
FILE 0x100000
*/
pub fn sys_mkdir(dirfd: isize, path: *const u8, cmod: usize) -> isize {
    info!("sysmkdir");
    let task = current_task().unwrap();
    let token = current_user_token();
    let inner = task.inner_exclusive_access();
    let cwd = inner.cwd.clone();
    // let current_file = &inner.fd_table[dirfd].unwrap();
    info!("Dir fd {}", dirfd);
    if (dirfd == -100) {
        let name = &translated_str(token, path);
        ROOT_INODE.create(name, ATTRIBUTE_ARCHIVE);
        sync_all();
        let osinode = ROOT_INODE.find_inode(&name);
        if let Some(osinode) = osinode {
            info!("Create Inode Suceess");
        }
    }
    0
}

pub fn sys_chdir(path: *const u8) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.inner_exclusive_access();

    let name = &translated_str(token, path);
    let osinode = find_osinode_by_path(&name);
    if let Some(osinode) = osinode {
        debug!("Sys_chdir find node {}", &name);
        inner.cwd = translated_str(token, path);
        debug!("Sys_chdir switch chdir to {}", &inner.cwd);
    } else {
        debug!("Sys_chdir fail to find node {}", &inner.cwd);
        return 1;
    }
    0
}
