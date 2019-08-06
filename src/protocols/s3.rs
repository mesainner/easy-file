#![allow(unused_imports)]
#![allow(unused_variables)]  // Mainly got tired of looking at warnings so added this :)

extern crate aws_sdk_rust;
//#[macro_use]
extern crate lsio;
extern crate url;
//extern crate hyper;
extern crate rustc_serialize;
extern crate term;
extern crate md5;

use std::io;
use std::io::{Read, Write, Seek, SeekFrom, BufReader};
use std::path::Path;
use std::fs::File;
use std::str;
use std::str::FromStr;
// NOTE: Have to add `use std::iter;` if using repeat macros
use std::iter;

use rustc_serialize::json;
use rustc_serialize::base64::{ToBase64, STANDARD};

use lsio::commands::run_cli;

use hyper::client::Client;
use aws_sdk_rust::aws::common::credentials::*;
// NOTE: The bucket and obect use is using * but you may want to use specific items instead of everything
use aws_sdk_rust::aws::s3::bucket::*;
use aws_sdk_rust::aws::s3::object::*;
use aws_sdk_rust::aws::s3::acl::*;

use aws_sdk_rust::aws::common::region::Region;
use aws_sdk_rust::aws::s3::endpoint::{Endpoint, Signature};
use aws_sdk_rust::aws::s3::s3client::S3Client;

use crate::cache::disk::{DiskCache, CacheFlag};
use crate::client_opt::ClientOpt;
use crate::file_opt::{FileOpt, FileAttr};
use crate::error::OptError;
use std::io::Result;

pub struct Awss3Client {
    field: S3Client<AutoRefreshingProvider<ChainProvider>, Client>,
    bucket_name: String,
    object_name: String,
    read_offset: i64,
}

impl FileOpt for Awss3Client {
    fn open(path: &str, flag: Option<CacheFlag>) -> Self {
        let pos = path.find("//").unwrap();
        let path = &path[(pos+"//".len())..path.len()];
        let v: Vec<&str> = path.split('/').collect();
        let vas: Vec<&str> = v[0].split('@').collect();

        let access_key_id: &str = vas[0];
        let access_key_secret: &str = vas[1];
        let mut end_point = String::from("https://");
        end_point += v[1];
        let bucket_name: &str = if v.len() > 2 {v[2]}else{""};
        let object_name = if v.len() > 3 {v[3]}else{""};

        let param_provider: Option<ParametersProvider> = Some(
            ParametersProvider::with_parameters(
                access_key_id,
                access_key_secret,
                None).unwrap()
        );
    
        let provider = DefaultCredentialsProvider::new(param_provider).unwrap();

        // 这里区分腾讯云 v2
        let endpoint = Endpoint::new(
            Region::UsEast1, 
            Signature::V4, 
            Some(url::Url::parse(&end_point).unwrap()), 
            None, 
            None, 
            Some(false),
        );
        let client = S3Client::new(provider, endpoint);

        Awss3Client{
            field: client,
            bucket_name: bucket_name.to_string(),
            object_name: object_name.to_string(),
            read_offset: -1,
        }
    }

    fn get_attr(&self) -> Option<FileAttr>{
        let mut head_object = HeadObjectRequest::default();
        head_object.bucket = self.bucket_name.to_string();
        head_object.key = self.object_name.to_string();

        match self.field.head_object(&head_object) {
            Ok(output) => {
                Some(FileAttr{
                    file_size: output.content_length,
                    last_modify: output.last_modified,
                })
            },
            Err(e) => {
                None
            },
        }
    }

    fn seek(&mut self, offset: i64) -> Result<()>{
        self.read_offset = offset;
        Ok(())
    }

    fn read(&mut self, len: i64) -> Option<Vec<u8>>{
        let mut get_object = GetObjectRequest::default();
        get_object.bucket = self.bucket_name.to_string();
        get_object.key = self.object_name.to_string();

        if self.read_offset != -1 {
            if len == -1 {
                get_object.range = Some(format!("bytes={}-", self.read_offset));
            }
            else{
                get_object.range = Some(format!("bytes={}-{}", self.read_offset, self.read_offset + len -1));
            }
        }

        let ret: Vec<u8> = match self.field.get_object(&get_object, None) {
            Ok(output) => {
                if output.is_body {
                    output.body
                }
                else {
                    output.body_buffer   
                }
            },
            Err(e) => {
                println!("{:#?}", e);
                Vec::new()
            },
        };

        Some(ret)
    }

    fn read_to_file(&mut self, len: i64, path: &str) -> Result<()>{
        Ok(())
    }

    fn is_exist(path: &str) -> bool {
        let client = Awss3Client::open(path, None);
        match client.get_attr() {
            Some(_) => true,
            None => false,
        }
    }

    fn write_from_file(&self, path: &str, progress: &mut f32) -> Result<()>{
        
        let mut ret = "write object success!".to_string();

        let mut create_multipart_upload_output: Option<MultipartUploadCreateOutput> = None;
        let mut create_multipart_upload = MultipartUploadCreateRequest::default();
        create_multipart_upload.bucket = self.bucket_name.to_string();
        create_multipart_upload.key = self.object_name.to_string();

        match self.field.multipart_upload_create(&create_multipart_upload) {
            Ok(output) => {
                println!("{:#?}", output);
                create_multipart_upload_output = Some(output);
            },
            Err(e) => ret = format!("{:#?}", e),
        }

        if create_multipart_upload_output.is_some() {
            let create_multipart_upload = create_multipart_upload_output.unwrap();
            let upload_id: &str = &create_multipart_upload.upload_id;
            let mut parts_list: Vec<String> = Vec::new();
            let dd = File::open(path).unwrap();
            let mut f = &dd;

            let metadata = f.metadata().unwrap();
            let len = metadata.len();
            let min_size: u64 = 1024*1024*50;
            let mut part_num: u64 = 0;
            loop {
        
                let offset: u64 = min_size * part_num;
                let seekoffset = if offset == 0 { 0 } else {offset + 1};
                f.seek(SeekFrom::Start(seekoffset)).unwrap();

                let read_bytes: u64 = if len > offset + min_size {
                    min_size
                } 
                else {
                    len - offset
                };

                let mut part_buffer: Vec<u8> = Vec::with_capacity(min_size as usize);
                match f.take(min_size).read_to_end(&mut part_buffer) {
                    Ok(_) => println!("Read in buffer 1 - {}", part_buffer.len()),
                    Err(e) =>  {
                        ret = format!("Error reading file {}", e);
                        break;
                    },
                }

                let mut upload_part = MultipartUploadPartRequest::default();
                upload_part.bucket = self.bucket_name.to_string();
                upload_part.upload_id = upload_id.to_string();
                upload_part.key = self.object_name.to_string();
                upload_part.body = Some(&part_buffer);
                upload_part.part_number = (part_num + 1) as i32;
                // Compute hash - Hash is slow
                //let hash = md5::compute(upload_part.body.unwrap()).to_base64(STANDARD);
                //upload_part.content_md5 = Some(hash);

                match self.field.multipart_upload_part(&upload_part) {
                    Ok(output) => {
                        // Collecting the partid in a list.
                        let new_out = output.clone();
                        parts_list.push(output);
                        println!("Part {} - {:#?}", part_num, new_out);
                    },
                    Err(e) => {
                        ret = format!("{:#?}", e);
                        break;
                    },
                }

                if offset + min_size >= len {
                    break;
                }
                part_num += 1;
                *progress = (offset / len) as f32;
            }

            let item_list : Vec<u8>;
            let mut complete_multipart_upload = MultipartUploadCompleteRequest::default();
            complete_multipart_upload.bucket = self.bucket_name.to_string();
            complete_multipart_upload.upload_id = upload_id.to_string();
            complete_multipart_upload.key = self.object_name.to_string();

            // parts_list gets converted to XML and sets the item_list.
            match multipart_upload_finish_xml(&parts_list) {
                Ok(parts_in_xml) => item_list = parts_in_xml,
                Err(e) => {
                    item_list = Vec::new(); // Created the list here so it will fail in the call below
                    ret = format!("{:#?}", e);
                },
            }

            complete_multipart_upload.multipart_upload = Some(&item_list);

            match self.field.multipart_upload_complete(&complete_multipart_upload) {
                Ok(output) => println!("{:#?}", output),
                Err(e) => ret = format!("{:#?}", e),
            }
        }

        Ok(())
    }

    fn write(&self, buffer: &[u8])  -> Result<()>{
        
        let mut put_object = PutObjectRequest::default();
        put_object.bucket = self.bucket_name.to_string();
        put_object.key = self.object_name.to_string();
        put_object.body = Some(buffer);

        match self.field.put_object(&put_object, None) {
            Ok(output) => {
                println!("{:#?}", output);
                Ok(())
            },
            Err(e) => {
                println!("{:#?}", e);
                Err(io::Error::last_os_error())
            }
        }
    }

    fn close(&self) {

    }

    fn enumerate(path: &str) -> Result<Vec<String>> {
        let mut ret: Vec<String> = Vec::new();
        let client = Awss3Client::open(path, None);
        let mut list_objects = ListObjectsRequest::default();
        list_objects.bucket = client.bucket_name.to_string();

        match client.field.list_objects(&list_objects) {
            Ok(objects) => {
                for var in &objects.contents{
                    ret.push(var.key.to_string());
                }
                println!("{:#?}", objects);
            },
            Err(e) => {
                println!("{:#?}", e);
            }
        }

        Ok(ret)
    }

    fn delete(path: &str) -> Result<()>{
        let client = Awss3Client::open(path, None);

        let mut del_object = DeleteObjectRequest::default();
        del_object.bucket = client.bucket_name.to_string();
        del_object.key = client.object_name.to_string();
        match client.field.delete_object(&del_object, None) {
            Ok(output) => {
                println!("{:#?}", output);
                Ok(())
            }
            Err(e) => {
                println!("{:#?}", e);
                Err(io::Error::last_os_error())
            }
        }
    }
}

/*
impl ClientOpt for Awss3Client {
    fn new(       
        access_key_id: &str,
        access_key_secret: &str,
        token_id: &str,
        end_point: &str,
        proxy: &str,
        user_agent: &str,
        sig_type: &str,
        is_bucket_vt: bool,
    ) -> Self {

        let token_id: Option<String> = match token_id.len(){
            0 => None,
            len => Some(token_id.to_string()),    
        };
        let param_provider: Option<ParametersProvider> = Some(
            ParametersProvider::with_parameters(
                access_key_id,
                access_key_secret,
                token_id).unwrap()
        );
    
        let provider = DefaultCredentialsProvider::new(param_provider).unwrap();
        let proxy: Option<url::Url> = match proxy.len(){
            0 => None,
            len => Some(url::Url::parse(&proxy).unwrap()),    
        };
        let user_agent: Option<String> = match user_agent.len(){
            0 => None,
            len => Some(user_agent.to_string()),    
        };

        let sig_type: Signature = if sig_type == "V2" {Signature::V2} else {Signature::V4};
        let endpoint = Endpoint::new(
            Region::UsEast1, 
            Signature::V4, 
            Some(url::Url::parse(&end_point).unwrap()), 
            proxy, 
            user_agent, 
            Some(is_bucket_vt),
        );
        let client = S3Client::new(provider, endpoint);

        Awss3Client{field: client}
    }

    fn read_object_to_mem(
        &self, 
        bucket_name: &str, 
        object_name: &str,
        offset: i64,
        len: i64,
    ) -> Result<Vec<u8>> {
        
        let mut get_object = GetObjectRequest::default();
        get_object.bucket = bucket_name.to_string();
        get_object.key = object_name.to_string();

        if offset != -1 {
            if len == -1 {
                get_object.range = Some(format!("bytes={}-", offset));
            }
            else{
                get_object.range = Some(format!("bytes={}-{}", offset, offset + len -1));
            }
        }

        let ret: Vec<u8> = match self.field.get_object(&get_object, None) {
            Ok(output) => {
                if output.is_body {
                    output.body
                }
                else {
                    output.body_buffer   
                }
            },
            Err(e) => {
                println!("{:#?}", e);
                Vec::new()
            },
        };
        
        Ok(ret)
    }

    fn read_object_to_file(
        &self, 
        bucket_name: &str, 
        object_name: &str,
        path: &str,
        progress: &mut f32    
    ) -> Result<String>{
        
        let mut ret = "read object to file success!".to_string();

        let mut head_object = HeadObjectRequest::default();
        head_object.bucket = bucket_name.to_string();
        head_object.key = object_name.to_string();

        let mut object_len: i64 = 0;
        match self.field.head_object(&head_object) {
            Ok(output) => {
                object_len = output.content_length as i64;
                println!("{}", object_len);
            },
            Err(e) => ret = format!("{:#?}", e),
        }

        let dd = File::create(path).unwrap();
        let mut f = &dd;
        let mut offset: i64 = 0;
        loop{
            let buf = self.read_object_to_mem(bucket_name, object_name, offset, 1024*1024).unwrap();
            f.write_all(&buf).unwrap();
            f.flush()?;

            offset += buf.len() as i64;
            if offset >= object_len {
                break;
            }

            *progress = (offset / object_len) as f32;
        }

        Ok(ret)
    }

    fn write_object_with_mem(
        &self, 
        bucket_name: &str, 
        object_name: &str,
        datas: &[u8],
    ) -> Result<()> {
        
        let mut put_object = PutObjectRequest::default();
        put_object.bucket = bucket_name.to_string();
        put_object.key = object_name.to_string();
        put_object.body = Some(datas);

        match self.field.put_object(&put_object, None) {
            Ok(output) => {
                println!("{:#?}", output);
                Ok(())
            },
            Err(e) => {
                println!("{:#?}", e);
                Err(io::Error::last_os_error())
            }
        }
    }

    fn write_object_with_file(
        &self, 
        bucket_name: &str, 
        object_name: &str,
        path: &str,
        progress: &mut f32
    ) -> Result<String>{

        let mut ret = "write object success!".to_string();

        let mut create_multipart_upload_output: Option<MultipartUploadCreateOutput> = None;
        let mut create_multipart_upload = MultipartUploadCreateRequest::default();
        create_multipart_upload.bucket = bucket_name.to_string();
        create_multipart_upload.key = object_name.to_string();

        match self.field.multipart_upload_create(&create_multipart_upload) {
            Ok(output) => {
                println!("{:#?}", output);
                create_multipart_upload_output = Some(output);
            },
            Err(e) => ret = format!("{:#?}", e),
        }

        if create_multipart_upload_output.is_some() {
            let create_multipart_upload = create_multipart_upload_output.unwrap();
            let upload_id: &str = &create_multipart_upload.upload_id;
            let mut parts_list: Vec<String> = Vec::new();
            let dd = File::open(path).unwrap();
            let mut f = &dd;

            let metadata = f.metadata().unwrap();
            let len = metadata.len();
            let min_size: u64 = 1024*1024*50;
            let mut part_num: u64 = 0;
            loop {
        
                let offset: u64 = min_size * part_num;
                let seekoffset = if offset == 0 { 0 } else {offset + 1};
                f.seek(SeekFrom::Start(seekoffset)).unwrap();

                let read_bytes: u64 = if len > offset + min_size {
                    min_size
                } 
                else {
                    len - offset
                };

                let mut part_buffer: Vec<u8> = Vec::with_capacity(min_size as usize);
                match f.take(min_size).read_to_end(&mut part_buffer) {
                    Ok(_) => println!("Read in buffer 1 - {}", part_buffer.len()),
                    Err(e) =>  {
                        ret = format!("Error reading file {}", e);
                        break;
                    },
                }

                let mut upload_part = MultipartUploadPartRequest::default();
                upload_part.bucket = bucket_name.to_string();
                upload_part.upload_id = upload_id.to_string();
                upload_part.key = object_name.to_string();
                upload_part.body = Some(&part_buffer);
                upload_part.part_number = (part_num + 1) as i32;
                // Compute hash - Hash is slow
                //let hash = md5::compute(upload_part.body.unwrap()).to_base64(STANDARD);
                //upload_part.content_md5 = Some(hash);

                match self.field.multipart_upload_part(&upload_part) {
                    Ok(output) => {
                        // Collecting the partid in a list.
                        let new_out = output.clone();
                        parts_list.push(output);
                        println!("Part {} - {:#?}", part_num, new_out);
                    },
                    Err(e) => {
                        ret = format!("{:#?}", e);
                        break;
                    },
                }

                if offset + min_size >= len {
                    break;
                }
                part_num += 1;
                *progress = (offset / len) as f32;
            }

            let item_list : Vec<u8>;
            let mut complete_multipart_upload = MultipartUploadCompleteRequest::default();
            complete_multipart_upload.bucket = bucket_name.to_string();
            complete_multipart_upload.upload_id = upload_id.to_string();
            complete_multipart_upload.key = object_name.to_string();

            // parts_list gets converted to XML and sets the item_list.
            match multipart_upload_finish_xml(&parts_list) {
                Ok(parts_in_xml) => item_list = parts_in_xml,
                Err(e) => {
                    item_list = Vec::new(); // Created the list here so it will fail in the call below
                    ret = format!("{:#?}", e);
                },
            }

            complete_multipart_upload.multipart_upload = Some(&item_list);

            match self.field.multipart_upload_complete(&complete_multipart_upload) {
                Ok(output) => println!("{:#?}", output),
                Err(e) => ret = format!("{:#?}", e),
            }
        }

        Ok(ret)
    }

    fn list_objects(
        &self, 
        bucket_name: &str, 
    ) -> Result<String> {

        let mut list_objects = ListObjectsRequest::default();
        list_objects.bucket = bucket_name.to_string();
        // NOTE: The default is version 1 listing. You must set it to version 2 if you want version 2.
        //list_objects.version = Some(2);

        match self.field.list_objects(&list_objects) {
            Ok(objects) => {
                //println!("{:#?}", objects);
                //println!("----------JSON (serial)--");
                //let encoded = json::encode(&objects).unwrap();
                //println!("{:#?}", encoded);
                //println!("----------JSON-----------");
                //println!("{}", json::as_pretty_json(&objects));
                let ret = json::as_pretty_json(&objects);
                Ok(format!("{:#?}", objects).to_string())
            },
            Err(e) => {
                println!("{:#?}", e);
                Err(io::Error::last_os_error())
            }
        }
    }

    fn delete_object(
        &self, 
        bucket_name: &str,
        object_name: &str,
    ) -> Result<()> {
   
        let mut del_object = DeleteObjectRequest::default();
        del_object.bucket = bucket_name.to_string();
        del_object.key = object_name.to_string();

        match self.field.delete_object(&del_object, None) {
            Ok(output) => {
                println!("{:#?}", output);
                Ok(())
            }
            Err(e) => {
                println!("{:#?}", e);
                Err(io::Error::last_os_error())
            }
        }
    }

    fn query_object_info(
        &self, 
        bucket_name: &str,
        object_name: &str,
    ) -> Result<String> {

        let mut head_object = HeadObjectRequest::default();
        head_object.bucket = bucket_name.to_string();
        head_object.key = object_name.to_string();

        match self.field.head_object(&head_object) {
            Ok(output) => {
                //println!("{:#?}", output);
                Ok(format!("{:#?}", output).to_string())
            },
            Err(e) => {
                println!("{:#?}", e);
                Err(io::Error::last_os_error())
            },
        }
    }
}
*/
