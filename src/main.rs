use std::net::TcpStream;
use ssh2::Session;
use std::io::prelude::*;
use clap::Parser;

// A tool to run a command on a remote host using the ssh-agent as the authorization
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The host to connect to
    #[arg(short = 'H', long,)]
    host: String,

    // The port to connect to
    #[arg(short, long,)]
    port: String,

    // The user to connect with
    #[arg(short, long,)]
    user: String,

    // The commands to send
    #[arg(short, long,)]
    commands: Vec<String>,
}

fn connect(ip_address: String, user: String, port: String, commands: Vec<String>) {
    let tcp = TcpStream::connect(format!("{}:{}", ip_address, port)).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_agent( &user).unwrap();

    for i in 0..commands.len() {
        println!("running command: \n{:?}", &commands[i]);
        let mut channel = sess.channel_session().unwrap();
        channel.exec(&commands[i]).unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        println!("{}", s);
        let _ = channel.wait_close();
    }

}

fn main() {
  let args = Cli::parse();

  connect(args.host,args.user, args.port, args.commands);
}
