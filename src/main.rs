use std::net::{TcpListener, TcpStream};
use std::io::{IoSlice, IoSliceMut, prelude::*};
use std::fs::{File};
use std::os::unix::prelude::FileExt;
use std::process;

fn main() -> std::io::Result<()> {
    let server_mode = true;
    if server_mode {
        let listener = match TcpListener::bind("0.0.0.0:3333") {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Error creating server: {}", e);
                process::exit(1)
            }
        };
        
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("Connected to: {:?}", addr);
                handle_client(stream);
                drop(listener)
            },
            Err(e) => {
                eprintln!("Error connecting: {}", e);
                process::exit(1)
            }
        }
    
        Ok(())
    } else {
        send_file()
    }
}

fn send_file() -> std::io::Result<()> {
    let filename = "photo.jpg";
    let filepath = format!("send/{}", filename);
    let file = File::open(filepath)?;
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
    let mut stream = match TcpStream::connect("0.0.0.0:3333") {
        Ok(stream) => stream,
        Err(e) => panic!("Error creating stream: {}", e)
    };

    stream.set_nodelay(true)?;
    stream.write_vectored(&[filename_len_slice, file_len_slice, filename_slice])?;

    //  Send data in 4kB buffers

    let sections_needed = (file_metadata.len() as f64 / 4000.0).ceil();
    let sections_needed = sections_needed as u64;
    for i in 0..sections_needed {
        let mut buffer = vec!(0 as u8; 4000);
        let bytes_read = file.read_at(&mut buffer, i * 4000)?;

        stream.write(&buffer[0..bytes_read])?;
    }

    Ok(())
}

fn handle_client (mut stream: TcpStream) {
    let mut filename_len_data = [0 as u8; 8];
    let mut file_len_data = [0 as u8; 8];

    let filename_len_buf = IoSliceMut::new(&mut filename_len_data);
    let file_len_buf = IoSliceMut::new(&mut file_len_data);

    match stream.read_vectored(&mut [filename_len_buf, file_len_buf]) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error reading stream: {}", e);
            process::exit(1)
        }
    }
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

    let filepath = format!("rec/{}", &filename);

    let file = match File::create(&filepath) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error creating file {}: {}", filename, e);
            process::exit(1)
        }
    };
    let mut total_written = 0;
    for i in 0..sections_needed {
        let mut buffer = vec!(0 as u8; 4000);
        let buf_size = stream.peek(&mut buffer).unwrap();
    
        stream.read(&mut buffer[0..buf_size]).unwrap();
        let bytes_written = file.write_at(&buffer[0..buf_size], i * 4000 ).unwrap();
        total_written += bytes_written;
    }
    println!("{} / {} bytes written", total_written, content_len)
}