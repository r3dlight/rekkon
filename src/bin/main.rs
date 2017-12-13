#[macro_use] extern crate log;
extern crate rekkon;
extern crate config;
extern crate simplelog;
use rekkon::ThreadPool;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;
use std::thread;
use std::time::Duration;
use simplelog::*;


fn main() {

    CombinedLogger::init(
        vec![
            TermLogger::new(LogLevelFilter::Warn, Config::default()).unwrap(),
            WriteLogger::new(LogLevelFilter::Info, Config::default(), File::create("rekkon.log").unwrap()),
        ]
    ).unwrap();

    let mut settings = config::Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name("config")).unwrap()
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(config::Environment::with_prefix("APP")).unwrap();

    //println!("{:?}", settings.get_str("debug").unwrap());
    info!("Parsing config.toml.");
    //println!("{:?}", settings.get_str("ListenOn").unwrap());
    //println!("{:?}", settings.get_str("NumberOfThreads").unwrap());
    let listen_on = settings.get_str("ListenOn").unwrap();
    info!("Rekkon will bind to : {:?}", settings.get_str("ListenOn").unwrap());
    let _debug = settings.get_str("Debug").unwrap();
    info!("Debug is set to : {:?}", settings.get_str("Debug").unwrap());
    let number_of_threads = settings.get_str("NumberOfThreads").unwrap();
    info!("Worker pool is set to : {:?}", settings.get_str("NumberOfThreads").unwrap());
    //println!("{:?}", listen_on);
    let listener = TcpListener::bind(listen_on).unwrap();
    let pool = ThreadPool::new(number_of_threads.parse().unwrap());

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

     let mut file = File::open(filename).unwrap();
     let mut contents = String::new();

     file.read_to_string(&mut contents).unwrap();

     let response = format!("{}{}", status_line, contents);

     stream.write(response.as_bytes()).unwrap();
     stream.flush().unwrap();
}
