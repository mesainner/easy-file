use std::io::Result;
use crate::file_opt::{FileOpt, FileAttr};
use crate::cache::disk::CacheFlag;

#[derive(Debug, Default)]
pub struct FtpClient {
    client: i32,
    read_offset: i64,
}

impl FileOpt for FtpClient {
    fn open(path: &str, flag: Option<CacheFlag>) -> Self {
        FtpClient {
            client:1,
            read_offset: 0,
        }
    }

    fn get_attr(&self) -> Option<FileAttr>{
        None
    }

    fn seek(&mut self, offset: i64) -> Result<()>{
        self.read_offset = offset;
        Ok(())
    }

    fn read(&mut self, len: i64) -> Option<Vec<u8>>{
        None
    }

    fn read_to_file(&mut self, len: i64, path: &str) -> Result<()>{
        Ok(())
    }

    fn is_exist(path: &str) -> bool{
        true
    }
    
    fn write_from_file(&self, path: &str, progress: &mut f32) -> Result<()>{
        Ok(())
    }

    fn write(&self, buffer: &[u8]) -> Result<()>{
        Ok(())
    }

    fn close(&self) {

    }

    fn enumerate(path: &str) -> Result<Vec<String>> {
        let ret: Vec<String> = Vec::new();
        Ok(ret)
    }

    fn delete(path: &str) -> Result<()>{
        Ok(())
    }

}