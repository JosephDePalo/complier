use std::io::Read;
use std::net::TcpStream;

use mlua::{UserData, UserDataMethods};
use ssh2::Session;

pub struct SSHSession {
    sess: Session,
}

impl SSHSession {
    pub fn new(addr: &str, username: &str, password: &str) -> Self {
        let tcp = TcpStream::connect(addr).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        sess.userauth_password(username, password).unwrap();
        assert!(sess.authenticated());

        SSHSession { sess }
    }

    pub fn run_cmd(self: &Self, cmd: &str) -> String {
        let mut channel = self.sess.channel_session().unwrap();
        channel.exec(cmd).unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        channel.wait_close().unwrap();
        String::from(s.trim())
    }
}

impl UserData for SSHSession {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Expose a 'run_command' method to Lua
        methods.add_method_mut("run_cmd", |_, ssh_session, command: String| {
            Ok(ssh_session.run_cmd(command.as_str()))
        });
    }
}
