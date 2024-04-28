use crate::drivers::BLOCK_DEVICE;
use crate::mm::UserBuffer;
use crate::println;
use crate::sync::UPSafeCell;
use alloc::string::String;
use alloc::sync::Arc;
use fat32_fs::*;
use lazy_static::*;
use log::info;
use xmas_elf::header;

mod file;
mod pipe;
mod stdio;
pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

// lazy_static! {
//     pub static ref ROOT_INODE: Arc<fat32_fs::VFile> = {
//         let efs = FAT32Manager::open(BLOCK_DEVICE.clone());
//         let root_inode = efs.get_root_vfile(&efs.clone());
//         Arc::new(root_inode)
//     };
// }

lazy_static! {
    pub static ref ROOT_INODE: Arc<OSInode> = {
        let efs = FAT32Manager::open(BLOCK_DEVICE.clone());
        let root_inode = efs.get_root_vfile(&efs.clone());
        Arc::new(OSInode::new(true, true, Arc::new(root_inode)))
    };
}

pub fn find_osinode_by_path(path: &str) -> Option<Arc<OSInode>> {
    let mut path_ = path;
    let mut targetinode = Some(Arc::new(OSInode::new(false, false, ROOT_INODE.get_vfile())));
    loop {
        {
            let mut parts = path_.splitn(2, '/');
            let first_part = parts.next().unwrap();
            let rest = parts.next().unwrap_or("");
            path_ = rest;
            info!("first part {}", first_part);
            targetinode = targetinode.unwrap().find_inode(first_part);
            info!("path {}", path_);
            if (rest == "") {
                break;
            }
        }
    }
    targetinode
}
pub use file::{open_file, OSInode, OpenFlags};
pub use pipe::*;
pub use stdio::*;

pub fn osinode_sort_test() {
    let txtinode = find_osinode_by_path("dir/a.txt");
    let mut buf = [0; 128];
    txtinode.unwrap().get_vfile().read_at(0, &mut buf);
    info!("{}", core::str::from_utf8(&buf).unwrap());
}
