//! Wrappers around UDP sockets, used for server-client communication and simulation of ping delay.

use std::{
    io, net,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

fn spawn_sender(
    socket: net::UdpSocket,
    tx: mpsc::Sender<Box<[u8]>>,
    ping_ms: Arc<AtomicU64>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0; u16::MAX as _];
        loop {
            if let Ok(read) = socket.recv(&mut buf) {
                let delay = load_delay(&ping_ms);
                let tx_ref = tx.clone();
                thread::spawn(move || {
                    thread::sleep(delay);
                    let boxed = Box::from(&buf[..read]);
                    let _ = tx_ref.send(boxed);
                });
            }
            // if server is not running, busy wait
        }
    })
}

fn load_delay(ms: &AtomicU64) -> Duration {
    let delay_ms = ms.load(Ordering::Relaxed) / 2;
    Duration::from_millis(delay_ms)
}

/// Wrapper used by the client to receive server responses and simulate ping delay.
pub struct Client {
    ping_ms: Arc<AtomicU64>,
    socket: net::UdpSocket,
    receiver: mpsc::Receiver<Box<[u8]>>,
}

impl Client {
    const PORT: u16 = 0;

    /// Create a new socket and connect to the remote address.
    ///
    /// The specified ping will be used to delay incoming and outgoing packets.
    pub fn connect<A>(remote: A, simulated_ping_ms: u64) -> io::Result<Client>
    where
        A: net::ToSocketAddrs,
    {
        let socket = net::UdpSocket::bind((net::Ipv4Addr::UNSPECIFIED, Self::PORT))?;
        socket.connect(remote)?;

        let (tx, receiver) = mpsc::channel();
        let ping_ms = Arc::new(AtomicU64::new(simulated_ping_ms));
        let socket_ref = socket.try_clone()?;

        spawn_sender(socket_ref, tx, Arc::clone(&ping_ms));

        Ok(Self {
            ping_ms,
            socket,
            receiver,
        })
    }

    /// Set the value used for simulated packet delays.
    pub fn set_ping(&self, ms: u64) {
        self.ping_ms.store(ms, Ordering::Relaxed);
    }

    /// Iterate over all pending packets from the server.
    pub fn recv(&self) -> impl Iterator<Item = Box<[u8]>> {
        self.receiver.try_iter()
    }

    /// Send a packet to the server.
    pub fn send(&self, msg: &impl serde::Serialize) -> io::Result<()> {
        let socket = self.socket.try_clone()?;
        let serialized = serde_json::to_vec(msg).unwrap();
        let delay = load_delay(&self.ping_ms);
        thread::spawn(move || {
            thread::sleep(delay);
            let _ = socket.send(&serialized);
        });
        Ok(())
    }
}

/// Wrapper used by the server send state to clients and receive messages.
///
/// None of the functions in this implementation will block the thread while waiting to send or receive packets.
pub struct Server {
    socket: net::UdpSocket,
    buf: Box<[u8]>,
}

impl Server {
    /// Create a socket and bind it to the specified host address.
    pub fn bind(host: net::Ipv4Addr, port: u16) -> io::Result<Self> {
        let socket = net::UdpSocket::bind((host, port))?;
        socket.set_nonblocking(true)?;

        let buf = std::iter::repeat_n(0, u16::MAX as _).collect();

        Ok(Self { socket, buf })
    }

    /// Receive one packet from a client.
    ///
    /// Returns the bytes received and the origin of the packet.
    pub fn recv(&mut self) -> io::Result<(&[u8], net::SocketAddr)> {
        let (read, origin) = self.socket.recv_from(&mut self.buf)?;
        Ok((&self.buf[..read], origin))
    }

    /// Send one packet to the specified client address.
    pub fn send<A: net::ToSocketAddrs>(&self, data: &[u8], addr: A) -> io::Result<()> {
        self.socket.send_to(data, addr).map(drop)
    }
}
