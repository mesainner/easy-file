#[derive(Debug)]
pub enum ErrorType {
    OptSuccess,
    OpenFileFailure,
    FileNotExist,
    ReadFileFailure,
    WriteFileFailure,
    Unknuw,
}

#[derive(Debug)]
pub struct OptError{
    msg: String,
}

impl OptError {
    pub fn get_msg(err_type: ErrorType) -> String {
        match err_type {
            ErrorType::OptSuccess => "operation successfully!".to_string(),
            ErrorType::Unknuw => "unknow error!".to_string(),
            ErrorType::OpenFileFailure => "open file failure!".to_string(),
            ErrorType::FileNotExist => "file not exist!".to_string(),
            ErrorType::ReadFileFailure => "read file failure!".to_string(),
            ErrorType::WriteFileFailure => "write file failure!".to_string(),
        }
    }
}