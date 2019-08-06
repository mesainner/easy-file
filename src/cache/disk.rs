extern crate crypto_hash;
use crypto_hash::{Algorithm, hex_digest};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{SeekFrom, Seek};
use std::io::Result;
use crate::dsptype::types::FileType;
use std::collections::HashMap;
use crate::file_opt::FileOpt;

// node_size 固定，文件名hash携带的又分片编号，否则匹配不上
pub struct CacheFlag {  
    pub node_size: i64,
    pub cache_maxsize: i64,
}

pub struct DiskCache {
    path: String,
    current_cache_size: i64,
    flag: CacheFlag,

    // file_name = hash256(last_modify+file_size+path+cache_name)
    // file_name : hash256
    cache_node_list: HashMap<String, String>,
}

#[derive(Debug, Default)]
struct CacheNodeHeader {
    total_size: String,
    ver: String,
    image: String,
    file_size: String,
    node_size: String,
    path: String,
    last_modify: String,
}

impl DiskCache {
    pub fn new(path: &str, flag: CacheFlag) -> Self {

        let metadata = fs::metadata("./cache");
        if metadata.is_err() {
            fs::create_dir("./cache").unwrap();
        }

        DiskCache {
            path: path.to_string(),
            current_cache_size: 0,
            flag: flag,
            cache_node_list: HashMap::new(),
        }
    }

    fn get_cache_node_from_net(&self, offset: i64, len: i64, file_handle: &mut FileType, cache_path: &str) -> Result<()> {
        file_handle.seek(offset).unwrap();
        let read_byte = file_handle.read(len).unwrap();

        fs::write(cache_path, read_byte).unwrap();
        Ok(())
    }

    pub fn get_mem_to_file(&mut self, offset: i64, len: i64, file_handle: &mut FileType, save_path: &str) -> Result<()>{
        let node_offset = offset % (20 * 1024 * 1024);
        let node_next = 20 * 1024 * 1024 - node_offset;
        let mut offset = offset;
        let mut len = len; 
        if node_next >= len {
            let buffer = self.get_mem(offset, len, file_handle);
            if buffer.len() == len as usize {
                fs::write(save_path, buffer).unwrap();
                return Ok(());
            }
        }
        else {
            let buffer = self.get_mem(offset, node_next, file_handle);
            if buffer.len() == len as usize {
                fs::write(save_path, buffer).unwrap();
                offset += node_next;
                len -= node_next;
            }
        }

        loop {
            let read_len = if len >= 20 * 1024 * 1024 {20 * 1024 * 1024}else{len};
            let buffer = self.get_mem(offset, read_len, file_handle);
            if buffer.len() == len as usize {
                fs::write(save_path, buffer).unwrap();
                len -= read_len;
                if len == 0 {
                    return Ok(());
                }

                offset += read_len;
            }     
        }
    }

    pub fn get_mem(&mut self, offset: i64, len: i64, file_handle: &mut FileType) -> Vec<u8>{
        
        let attr = file_handle.get_attr().unwrap();

        let mut offset: i64 = offset;
        let mut ret_buffer: Vec<u8> = Vec::new();

        loop {
            let num = offset / (20 * 1024 * 1024);
            let node_offset = offset % (20 * 1024 * 1024);
            let mut hash_factors: String = attr.last_modify.to_string();
            hash_factors += &self.path;
            hash_factors += &(num.to_string());
            hash_factors += &(attr.file_size.to_string());
            let digest = match self.cache_node_list.contains_key(&hash_factors) {
                true => self.cache_node_list.get(&hash_factors).unwrap().to_string(),
                false => {
                    let hash256 = hex_digest(Algorithm::SHA256, hash_factors.as_bytes()).to_string();
                    self.cache_node_list.insert(hash_factors, hash256.to_string());
                    hash256
                },
            };

            let mut cache_node_path = "./cache/".to_string();
            cache_node_path += &digest;
            let metadata = fs::metadata(&cache_node_path);
            if metadata.is_ok() {
                // 有文件
                let metadata = metadata.unwrap();
                let sss: i64 = metadata.len() as i64 - node_offset;
                let f = File::open(&cache_node_path);
                if f.is_ok() {
                    let mut file = f.unwrap();
                    file.seek(SeekFrom::Start(node_offset as u64)).unwrap();
        
                    if sss >= len {
                        let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
                        let read_size = file.take(len as u64).read_to_end(&mut buffer).unwrap();
                        if read_size == len as usize {
                            ret_buffer.append(&mut buffer);
                            break;
                        }
                    }
                    else {
                        let mut buffer: Vec<u8> = Vec::with_capacity(sss as usize);
                        let read_size = file.take(sss as u64).read_to_end(&mut buffer).unwrap();
                        if read_size == sss as usize {
                            ret_buffer.append(&mut buffer);
                            offset += sss;
                            continue;
                        }
                    }
                }
            }

            fs::remove_file(&cache_node_path);
            self.get_cache_node_from_net(num * 20 * 1024 * 1024, 20 * 1024 * 1024, file_handle, &cache_node_path).unwrap();
        }

        ret_buffer
    }

    fn erase(num: i64) -> bool{
        true
    }

    fn is_empty(&self) -> bool{
        if self.current_cache_size == 0 {
            true
        }
        else {
            false
        }
    }
}