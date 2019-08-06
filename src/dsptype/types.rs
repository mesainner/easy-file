use std::io;
use std::io::Result;
use crate::protocols::s3::Awss3Client;
use crate::protocols::ftp::FtpClient;
use crate::protocols::smb::SmbClient;
use crate::file_opt::{FileOpt, FileAttr};
use crate::cache::disk::CacheFlag;

pub struct FileType {
    file_s3: Awss3Client,
    file_ftp: FtpClient,
    file_smb: SmbClient,
    path: String,
}

enum Type {
    Types3,
    TypeFtp,
    TypeSmb,
    Unknow,
}

fn dsptype(path: &str) -> Type{
    let pos = match path.find(":") {
        Some(t) => t,
        _ => {
            0
        },
    };

    let tag = &path[0..pos];
    if tag == "s3" {
        Type::Types3
    }
    else if tag == "ftp" {
        Type::TypeFtp
    }
    else if tag == "smb" {
        Type::TypeSmb
    }
    else {
        Type::Unknow
    }
}


impl FileOpt for FileType {
    fn open(path: &str, flag: Option<CacheFlag>) -> Self {
        let file_s3 = Awss3Client::open(path, None);
        let file_ftp = FtpClient::open(path, None);
        let file_smb = SmbClient::open(path, None);
        FileType{
            file_s3: file_s3,
            file_ftp: file_ftp,
            file_smb: file_smb,
            path: path.to_string(),
        }
    }

    fn seek(&mut self, offset: i64) -> Result<()>{
        let file_type = dsptype(&self.path);
        match file_type {
            Type::Types3 => self.file_s3.seek(offset),
            Type::TypeFtp => self.file_ftp.seek(offset),
            Type::TypeSmb => self.file_smb.seek(offset),
            Type::Unknow => Ok(()),
        }
    }

    fn get_attr(&self) -> Option<FileAttr>{
        let file_type = dsptype(&self.path);
        match file_type {
            Type::Types3 => self.file_s3.get_attr(),
            Type::TypeFtp => self.file_ftp.get_attr(),
            Type::TypeSmb => self.file_smb.get_attr(),
            Type::Unknow => None,
        }
    }

    fn read(&mut self, len: i64) -> Option<Vec<u8>> {
        let file_type = dsptype(&self.path);
        match file_type {
            Type::Types3 => self.file_s3.read(len),
            Type::TypeFtp => self.file_ftp.read(len),
            Type::TypeSmb => self.file_smb.read(len),
            Type::Unknow => None,
        }
    }

    fn read_to_file(&mut self, len: i64, path: &str) -> Result<()>{
        Ok(())
    }

    fn write_from_file(&self, path: &str, progress: &mut f32) -> Result<()>{
        let file_type = dsptype(&self.path);
        match file_type {
            Type::Types3 => self.file_s3.write_from_file(path, progress),
            Type::TypeFtp => self.file_ftp.write_from_file(path, progress),
            Type::TypeSmb => self.file_smb.write_from_file(path, progress),
            Type::Unknow => Ok(()),
        }
    }

    fn write(&self, buffer: &[u8]) -> Result<()> {
        let file_type = dsptype(&self.path);
        match file_type {
            Type::Types3 => self.file_s3.write(buffer),
            Type::TypeFtp => self.file_ftp.write(buffer),
            Type::TypeSmb => self.file_smb.write(buffer),
            Type::Unknow => Ok(()),
        }
    }

    fn is_exist(path: &str) -> bool  {
        let file_type = dsptype(path);
        match file_type {
            Type::Types3 => Awss3Client::is_exist(path),
            Type::TypeFtp => FtpClient::is_exist(path),
            Type::TypeSmb => SmbClient::is_exist(path),
            Type::Unknow => false,
        }
    }

    fn close(&self) {

    }

    fn enumerate(path: &str) -> Result<Vec<String>> {
        let file_type = dsptype(path);
        match file_type {
            Type::Types3 => Awss3Client::enumerate(path),
            Type::TypeFtp => FtpClient::enumerate(path),
            Type::TypeSmb => SmbClient::enumerate(path),
            Type::Unknow => Err(io::Error::last_os_error()),
        }
    }

    fn delete(path: &str) -> Result<()>{
        let file_type = dsptype(path);
        match file_type {
            Type::Types3 => Awss3Client::delete(path),
            Type::TypeFtp => FtpClient::delete(path),
            Type::TypeSmb => SmbClient::delete(path),
            Type::Unknow => Ok(()),
        }
    }
}