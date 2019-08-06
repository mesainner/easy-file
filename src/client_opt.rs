use std::io::Result;

pub trait ClientOpt {
/*    fn fopen(
        path: &str,
        flags: u8,
    );

    fn fread();
    fn fwrite();
    fn fclose();
    fn fseek();
    fn isexist();
    
*/
    fn new(
        access_key_id: &str,
        access_key_secret: &str,
        token_id: &str,
        end_point: &str,
        proxy: &str,
        user_agent: &str,
        sig_type: &str,
        is_bucket_vt: bool,
    ) -> Self;

    fn read_object_to_mem(
        &self, 
        bucket_name: &str, 
        object_name: &str,
        offset: i64,
        len: i64,
    ) -> Result<Vec<u8>>;

    fn read_object_to_file(
        &self, 
        bucket_name: &str, 
        object_name: &str,
        path: &str,
        progress: &mut f32    
    ) -> Result<String>;
    
    fn write_object_with_mem(
        &self, 
        bucket_name: &str, 
        object_name: &str,
        datas: &[u8],
    ) -> Result<()>;

    fn write_object_with_file(
        &self, 
        bucket_name: &str, 
        object_name: &str,
        path: &str,
        progress: &mut f32
    ) -> Result<String>;
    
    fn list_objects(
        &self, 
        bucket_name: &str, 
    ) -> Result<String>;

    fn delete_object(
        &self, 
        bucket_name: &str,
        object_name: &str,
    ) -> Result<()>;
    
    fn query_object_info(
        &self, 
        bucket_name: &str,
        object_name: &str,
    ) -> Result<String>;
}
