extern crate easy_file;
use easy_file::file_opt::FileOpt;
use easy_file::file::File;
use easy_file::cache::CacheFlag;

fn main() {

    let ssf = CacheFlag{node_size: 20, cache_maxsize: 20};
    let mut file = File::open("s3://ak@sk/cos.ap-chengdu.myqcloud.com/test-1259750376/123", Some(ssf));
    file.seek(3).unwrap();
    let read_bytes = file.read(4).unwrap();
    println!("{:?}", read_bytes);

    File::delete("s3://ak@sk/cos.ap-chengdu.myqcloud.com/test-1259750376/123").unwrap();
    let vec = File::enumerate("s3://ak@sk/cos.ap-chengdu.myqcloud.com/test-1259750376");

/*
    let tec = Awss3Client::new(
        "ak",
        "sk",
        "xx",
        "https://cos.ap-chengdu.myqcloud.com",
        "",
        "",
        "V4",
        true,
    );

    let sdf = tec.read_object_to_mem("test-1259750376", "123", 0, 40).unwrap();
    println!("\n\n{:#?}\n\n", str::from_utf8(&sdf).unwrap());

    tec.write_object_with_mem("test-1259750376", "1234", "datas".as_bytes()).unwrap();
    let listobjects = tec.list_objects("test-1259750376").unwrap();
    println!("{:?}", listobjects);
    let del = tec.delete_object("test-1259750376", "1234").unwrap();
    println!("{:?}", del);
    
    let ret_str = tec.query_object_info("test-1259750376", "123").unwrap();
    println!("{:?}", ret_str);

    let mut dwc: f32 = 0.0;
    let dex = tec.read_object_to_file("test-1259750376", "1234", "a.txt", &mut dwc).unwrap();
    println!("{:?}", dex);

    tec.write_object_with_file("test-1259750376", "1234", "a.txt", &mut dwc).unwrap();
*/
    println!("xxxxxxxxsdf={:?}", vec);
}
