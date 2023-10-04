use crossbeam::channel::{unbounded,TryRecvError};
use std::net::TcpStream as StdTcpStream;
use ssh2::Session;
use std::io::prelude::*;
use clap::Parser;
use std::thread;

// A tool to run a command on a remote host using the ssh-agent as the authorization
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The host to connect to
    #[arg(short = 'H', long,)]
    host: String,

    /// The port to connect to
    #[arg(short, long, default_value_t = String::from("22"))]
    port: String,

    /// The user to connect with
    #[arg(short, long,)]
    user: String,

    /// The commands to send
    #[arg(short, long,)]
    commands: Vec<String>,
}

fn connect(ip_address: String, user: String, port: String, commands: Vec<String>) {
    let tcp = StdTcpStream::connect(format!("{}:{}", ip_address, port)).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_agent(&user).unwrap();

    if commands.len() < 1 {
        let mut channel = sess.channel_session().unwrap();

        channel.request_pty("xterm", None, None).unwrap();

        channel.shell().unwrap();

        sess.set_blocking(false);

        let (trx, rev) = unbounded();

        thread::spawn(move || loop {
            let stdin = std::io::stdin();
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();
            trx.send(line).unwrap();
        });

        loop {
            let mut buf = vec![0; 4096];
            match channel.read(&mut buf) {
               Ok(_) => {
                   let s = String::from_utf8(buf).unwrap();
                   println!("{}", s);
               }
               Err(e) => {
                   if e.kind() != std::io::ErrorKind::WouldBlock {
                       println!("{}", e);
                   }
               }
            }

            if !rev.is_empty() {
                match rev.try_recv() {
                   Ok(line) => {
                       let cmd_string = line + "\n";
                       channel.write(cmd_string.as_bytes()).unwrap();
                       channel.flush().unwrap();
                    }
                   Err(TryRecvError::Empty) => {
                       println!("{}", "empty")
                    }
                   Err(TryRecvError::Disconnected) => {
                       println!("{}", "Disconnected");
                    }
                }
            }
        }
    }
    else {
        for i in 0..commands.len() {
            let mut channel = sess.channel_session().unwrap();
            channel.exec(&commands[i]).unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            println!("{}", s);
            let _ = channel.wait_close();
        }
    }
}

fn main() {
  let args = Cli::parse();

  connect(args.host,args.user, args.port, args.commands);
}



