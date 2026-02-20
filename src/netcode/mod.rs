use std::{
    io,
    net::{ToSocketAddrs, UdpSocket},
    time::Instant,
};

pub fn connect(ip: impl ToSocketAddrs) -> io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;

    let msg = "ping";
    let now = Instant::now();
    socket.send_to(msg.as_bytes(), ip)?;

    println!("Send {msg}");

    let mut buf = [0; 2048];
    let (len, src) = socket.recv_from(&mut buf)?;

    println!(
        "received {len} bytes: {}, from {src} in {}ms",
        unsafe { str::from_utf8_unchecked(&buf) },
        now.elapsed().as_secs_f64() * 1000.
    );

    Ok(())
}
