use bytes::{Buf, BufMut, BytesMut};
use snafu::{whatever, Whatever};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

const BITTORRENT_UDP_MAGIC: u64 = 0x041727101980;

pub(crate) async fn send_connect_request(
    socket: &UdpSocket,
    addr: SocketAddr,
) -> Result<u32, Whatever> {
    let transaction_id: u32 = rand::random();

    let mut bytes = BytesMut::with_capacity(16);
    bytes.put_u64(BITTORRENT_UDP_MAGIC);
    bytes.put_u32(0);
    bytes.put_u32(transaction_id);

    match socket.send_to(bytes.as_ref(), addr).await {
        Ok(_) => Ok(transaction_id),
        Err(e) => whatever!("{}", e),
    }
}

/// (transaction_id, connection_id)
pub(crate) async fn recv_connect_response(socket: &UdpSocket) -> Result<(u32, u64), Whatever> {
    let mut buf = BytesMut::new();
    buf.put_bytes(0, 16);

    let (l, addr) = socket.recv_from(buf.as_mut()).await.unwrap();
    if l != 16 {
        whatever!("Invalid length");
    }

    if buf.get_u32() != 0 {
        whatever!("Not connect request");
    }

    let transaction_id = buf.get_u32();
    let connection_id = buf.get_u64();

    Ok((transaction_id, connection_id))
}
