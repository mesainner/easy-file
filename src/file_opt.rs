use std::io::Result;
use crate::cache::disk::CacheFlag;

#[derive(Debug)]
pub struct FileAttr {
    pub file_size: i64,
    pub last_modify: String,
}

pub trait FileOpt {
    fn open(path: &str, flag: Option<CacheFlag>) -> Self;
    fn get_attr(&self) -> Option<FileAttr>;
    fn seek(&mut self, offset: i64) -> Result<()>;
    fn read(&mut self, len: i64) -> Option<Vec<u8>>;
    fn read_to_file(&mut self, len: i64, path: &str) -> Result<()>;
    fn write_from_file(&self, path: &str, progress: &mut f32) -> Result<()>;
    fn write(&self, buffer: &[u8]) -> Result<()>;
    fn close(&self) ;
    fn enumerate(path: &str) -> Result<Vec<String>>;
    fn delete(path: &str) -> Result<()>;
    fn is_exist(path: &str) -> bool;
}