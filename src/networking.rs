use std::{io, net, sync::mpsc, thread, time::Duration};

fn spawn_sender(
    socket: net::UdpSocket,
    tx: mpsc::Sender<Box<[u8]>>,
    delay: Duration,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0; u16::MAX as _];
        loop {
            match socket.recv(&mut buf) {
                Ok(read) => {
                    let tx_ref = tx.clone();
                    thread::spawn(move || {
                        thread::sleep(delay);
                        let boxed = Box::from(&buf[..read]);
                        tx_ref.send(boxed).unwrap();
                    });
                }
                Err(e) => eprintln!("recv error: {e}"),
            }
        }
    })
}

pub struct Client {
    simulated_ping: Duration,
    socket: net::UdpSocket,
    sending_thread: thread::JoinHandle<()>,
    receiver: mpsc::Receiver<Box<[u8]>>,
}

impl Client {
    const HOST: net::Ipv4Addr = net::Ipv4Addr::new(127, 0, 0, 1);
    const PORT: u16 = 0;

    pub fn connect<A>(remote: A, simulated_ping: Duration) -> io::Result<Client>
    where
        A: net::ToSocketAddrs,
    {
        let socket = net::UdpSocket::bind((Self::HOST, Self::PORT))?;
        socket.connect(remote)?;

        let (tx, receiver) = mpsc::channel();

        let socket_ref = socket.try_clone()?;
        let sending_thread = spawn_sender(socket_ref, tx, simulated_ping / 2);

        Ok(Self {
            simulated_ping,
            socket,
            sending_thread,
            receiver,
        })
    }

    pub fn try_iter(&self) -> mpsc::TryIter<Box<[u8]>> {
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
