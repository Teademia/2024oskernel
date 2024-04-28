use super::*;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use fat32_fs::*;
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>,
}

pub struct OSInodeInner {
    offset: usize,
    inode: Arc<VFile>,
}

impl OSInode {
    /// Construct an OS inode from a inode
    pub fn new(readable: bool, writable: bool, inode: Arc<VFile>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPSafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    pub fn find_inode(&self, file_name: &str) -> Option<Arc<OSInode>> {
        if let Some(vfile) = self.get_vfile().find_vfile_name(file_name) {
            Some(Arc::new(OSInode::new(true, false, Arc::new(vfile))))
        } else {
            None
        }
    }
    pub fn clear(&self) {
        let current_inode = self.inner.exclusive_access().inode.clone();
        current_inode.clear();
    }
    pub fn create(&self, name: &str, attribute: u8) -> Option<Arc<OSInode>> {
        let current_inode = self.inner.exclusive_access().inode.clone();
        if let Some(vfile) = current_inode.create(name, attribute) {
            Some(Arc::new(OSInode::new(true, false, vfile)))
        } else {
            None
        }
    }
    pub unsafe fn read_as_elf(&self) -> &'static [u8] {
        let current_inode = self.inner.exclusive_access().inode.clone();
        current_inode.read_as_elf()
    }
    pub fn get_vfile(&self) -> Arc<VFile> {
        self.inner.exclusive_access().inode.clone()
    }
}
bitflags! {
    ///Open file flags
    pub struct OpenFlags: u32 {
        ///Read only
        const RDONLY = 0;
        ///Write only
        const WRONLY = 1 << 0;
        ///Read & Write
        const RDWR = 1 << 1;
        ///Allow create
        const CREATE = 1 << 9;
        ///Clear file and return an empty one
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = ROOT_INODE.find_inode(name) {
            info!("Find {}", name);
            // clear size
            inode.clear();
            Some(inode)
        } else {
            // create file
            info!("Creating {}", name);
            ROOT_INODE.create(name, ATTRIBUTE_ARCHIVE)
        }
    } else {
        ROOT_INODE.find_inode(name)
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
}
