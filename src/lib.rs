use std::net::{TcpStream, SocketAddrV4};
use std::io::{prelude::*, IoSlice};
use std::fs::File;
//use std::os::unix::prelude::FileExt;
use std::process;
use std::time::Instant;


pub fn send_file(ip: &SocketAddrV4, path: &String) -> std::io::Result<()> {
    let fullpath = format!("{}/{}", std::env::current_dir().unwrap().display(), &path);
    let filename = path_to_name(&fullpath);
    let mut file = File::open(path)?;
    let file_metadata = file.metadata()?;

    //  Create byte slices holding the filename, name length, and content length
    let filename_data = filename.as_bytes();
    let filename_len_data = filename_data.len().to_be_bytes();
    let file_len_data = file_metadata.len().to_be_bytes();
 
    //  Create IoSlices
    let filename_len_slice = IoSlice::new(&filename_len_data);
    let file_len_slice = IoSlice::new(&file_len_data);
    let filename_slice = IoSlice::new(&filename_data);
    
    //  Change address back to 0.0.0.0 when you can't figure out why localhost won't work
    let mut stream = match TcpStream::connect(ip) {
        Ok(stream) => stream,
        Err(e) => panic!("Error creating stream: {}", e)
    };

    stream.set_nodelay(true)?;
    match stream.write_vectored(&[filename_len_slice, file_len_slice, filename_slice]) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error sending metadata: {}", e);
            process::exit(1)
        }
    };

    //  Send data in 4kB buffers

    let sections_needed = (file_metadata.len() as f64 / 4000.0).ceil();
    let sections_needed = sections_needed as u64;
    for _i in 0..sections_needed {
        let mut buffer = vec!(0 as u8; 4000);
        let bytes_read = file.read(&mut buffer)?;

        stream.write(&buffer[0..bytes_read])?;
    }
    println!("{} sent successfully", filename);

    Ok(())
}

pub fn handle_client (mut stream: TcpStream, path: &String) {
    let mut filename_len_data = [0 as u8; 8];
    let mut file_len_data = [0 as u8; 8];

    match stream.read(&mut filename_len_data) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error reading stream (filename length): {}", e);
            process::exit(1)
        }
    };
    std::thread::sleep(std::time::Duration::from_millis(15));
    match stream.read(&mut file_len_data) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error reading stream (file length): {}", e);
            process::exit(1)
        }
    };
    std::thread::sleep(std::time::Duration::from_millis(15));


    let filename_len = u64::from_be_bytes(filename_len_data);
    let content_len = u64::from_be_bytes(file_len_data);

    let mut filename_data = vec!(0 as u8; filename_len.try_into().unwrap());
    
    match stream.read(&mut filename_data) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error reading stream: {}", e);
            process::exit(1)
        }
    }

    let filename = std::str::from_utf8_mut(&mut filename_data).unwrap();
    println!("\tDownloading: {}\n\tSize: {} bytes", filename, content_len);

    let sections_needed = (content_len as f64 / 4000.0).ceil();
    let sections_needed = sections_needed as u64;
    let fullpath = format!("{}/{}", &path, &filename);
    let mut file = match File::create(&fullpath) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error creating file {}: {}", fullpath, e);
            process::exit(1)
        }
    };
    let mut total_written = 0;
    let now = Instant::now();
    for _i in 0..sections_needed {
        let mut buffer = vec!(0 as u8; 4000);
        let buf_size = stream.peek(&mut buffer).unwrap();
    
        stream.read(&mut buffer[0..buf_size]).unwrap();
        let bytes_written = file.write(&buffer[0..buf_size]).unwrap();
        total_written += bytes_written;
    }
    println!("{} / {} bytes written to {} in {} ms.", total_written, content_len, fullpath, now.elapsed().as_millis())
}

fn path_to_name(path: &String) -> String {
    let loc =  match path.rfind('/') {
        Some(loc) => loc,
        None => return path.clone()
    };
    path[loc+1..].to_string()
}
