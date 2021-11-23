use std::net::{TcpListener, IpAddr, SocketAddrV4};
use std::process;
use local_ip_address::local_ip;
use fs::{send_file, handle_client};

const PROGRAM_DESC: &str = "fs v0.0.1\nfs can be used to send and recieve files on a local network.\n
Usage:\nfs send [ADDRESS] (FILEPATH)\tsend a file to the specified server
fs rec\t\t\t\tstart listening for files
fs rec -o PATH/TO/DIR\t\toptionally specify a directory";

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let _program = args[0].clone();

    let config = Config::new(args);
    
    if config.server_mode {
        let listener = match TcpListener::bind(&config.ip) {
            Ok(listener) => {
                println!("Server listening on {}", &config.ip);
                listener
            },
            Err(e) => {
                eprintln!("Error creating server: {}", e);
                process::exit(1)
            }
        };
        
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("Connected to: {:?}", addr);
                handle_client(stream, &config.rec_dir);
                drop(listener)
            },
            Err(e) => {
                eprintln!("Error connecting: {}", e);
                process::exit(1)
            }
        }
    
        Ok(())
    } else {
        send_file(&config.ip, &config.send_path)
    }
}

struct Config {
    pub server_mode: bool,
    pub rec_dir: String,
    pub send_path: String,
    pub ip: SocketAddrV4
}

impl Config {
    fn new(args: Vec<String>) -> Config {
        let command = match args.get(1) {
            Some(arg) => arg.clone(), 
            None => {
                println!("{}", PROGRAM_DESC);
                process::exit(0);
            }
        };

        match command.as_str() {
            "send" => {
                let ip = match args.get(2) {
                    Some(ip) => ip,
                    None => {
                        eprintln!("Error: no file provided. Use fs send [ADDRESS] (FILEPATH)");
                        process::exit(1)
                    }
                };
                let ip: SocketAddrV4 = match ip.parse() {
                    Ok(ip) => ip,
                    Err(_) => {
                        eprintln!("Error: ip address not recognized");
                        process::exit(1)
                    }
                };
                let file = match args.get(3) {
                    Some(file) => file,
                    None => {
                        eprintln!("Error: no ip provided. Use fs send [ADDRESS] (FILEPATH)");
                        process::exit(1)
                    }
                };
                Config {
                    server_mode: false,
                    rec_dir: String::from("/rec"),
                    send_path: file.clone(),
                    ip
                }
            },
            "rec" => {
                let ip = match local_ip() {
                    Ok(IpAddr::V4(ip)) => ip,
                    Ok(IpAddr::V6(_)) => {
                        eprintln!("Could not get local ip");
                        process::exit(1)
                    },
                    Err(_) => {
                        eprintln!("Could not get local ip");
                        process::exit(1)
                    }
                };
                let mut rec_dir = String::from(std::env::current_dir().unwrap().display().to_string());
                if args.get(2) == Some(&String::from("-o")) {
                    rec_dir = match args.get(3) {
                        Some(dir) => String::from(format!("{}/{}", std::env::current_dir().unwrap().display(), dir)),
                        None => {
                            eprintln!("Error, no output directory provided. Use fs rec -o PATH/TO/DIR");
                            process::exit(1)
                        }
                    }; 
                }
                let socket: SocketAddrV4 = format!("{}:3333", &ip.to_string()).parse().unwrap();
                Config {
                    server_mode: true,
                    rec_dir, 
                    send_path: String::new(),
                    ip: socket
                }
            },
            "--help" => {
                println!("{}", PROGRAM_DESC);
                process::exit(1)
            },
            _ => {
                eprintln!("Error: unrecognized command. Use fs send [ADDRESS] (FILEPATH) or fs rec");
                process::exit(1);
            }
        }
    }
}



