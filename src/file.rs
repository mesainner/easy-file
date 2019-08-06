use std::io::{Result, Error};
use crate::file_opt::{FileOpt, FileAttr};
use crate::dsptype::types::FileType;
use crate::cache::disk::{DiskCache, CacheFlag};

pub struct File {
    client: FileType,
    cache: DiskCache,
    cur_offset: i64,
    is_init_ok: bool,
}

impl FileOpt for File {
    fn open(path: &str, flag: Option<CacheFlag>) -> Self {

        let file_client = FileType::open(path, None);
        let file_cache = DiskCache::new(path, flag.unwrap());

        File {
            client: file_client,
            cache: file_cache,
            cur_offset: 0,
            is_init_ok: true,
        }
    }

    fn get_attr(&self) -> Option<FileAttr>{
        self.client.get_attr()
    }

    fn seek(&mut self, offset: i64) -> Result<()> {
        if self.is_init_ok {
            self.cur_offset = offset;
            Ok(())
        }
        else {
            Err(Error::last_os_error())
        }
    }

    fn read(&mut self, len: i64) -> Option<Vec<u8>> {
        Some(self.cache.get_mem(self.cur_offset, len, &mut self.client))
    }

    fn read_to_file(&mut self, len: i64, path: &str) -> Result<()>{
        self.cache.get_mem_to_file(self.cur_offset, len, &mut self.client, path)
    }

    fn is_exist(path: &str) -> bool{
        FileType::is_exist(path)
    }

    fn write_from_file(&self, path: &str, progress: &mut f32) -> Result<()> {
        self.client.write_from_file(path, progress)
    }

    fn write(&self, buffer: &[u8]) -> Result<()> {
        self.client.write(buffer)
    }

    fn close(&self) {

    }

    fn enumerate(path: &str) -> Result<Vec<String>> {
        FileType::enumerate(path)
    }

    fn delete(path: &str) -> Result<()>{
        FileType::delete(path)
    }
}