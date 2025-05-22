use std::{io, net, sync::mpsc::TryIter, thread};

pub struct Client {
    simulated_ping: std::time::Duration,
    socket: net::UdpSocket,
    sending_thread: thread::JoinHandle<()>,
    receiver: std::sync::mpsc::Receiver<(usize, [u8; 2048])>,
}

impl Client {
    const HOST: net::Ipv4Addr = net::Ipv4Addr::new(127, 0, 0, 1);
    const PORT: u16 = 0;

    pub fn connect<A>(remote: A, simulated_ping: std::time::Duration) -> io::Result<Client>
    where
        A: net::ToSocketAddrs,
    {
        let socket = net::UdpSocket::bind((Self::HOST, Self::PORT))?;
        socket.connect(remote)?;

        let (tx, receiver) = std::sync::mpsc::channel();
        let socket_ref = socket.try_clone()?;

        let sending_thread = thread::spawn(move || {
            loop {
                let mut buf = [0; 2048];
                match socket_ref.recv(&mut buf) {
                    Ok(read) => {
                        let tx_ref = tx.clone();
                        thread::spawn(move || {
                            thread::sleep(simulated_ping / 2);
                            tx_ref.send((read, buf)).unwrap();
                        });
                    }
                    Err(e) => eprintln!("recv error: {e}"),
                }
            }
        });

        Ok(Self {
            simulated_ping,
            socket,
            sending_thread,
            receiver,
        })
    }

    pub fn try_iter(&self) -> TryIter<(usize, [u8; 2048])> {
        self.receiver.try_iter()
    }

    pub fn send(&self, msg: &impl serde::Serialize) -> io::Result<()> {
        let socket = self.socket.try_clone()?;
        let serialized = serde_json::to_vec(msg).unwrap();
        let delay = self.simulated_ping / 2;
        thread::spawn(move || {
            thread::sleep(delay);
            let _ = socket.send(&serialized);
        });
        Ok(())
    }
}
