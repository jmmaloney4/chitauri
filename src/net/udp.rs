use bytes::{Buf, BufMut, BytesMut};
use snafu::{whatever, Whatever};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::torrent::InfoHash;

const BITTORRENT_UDP_MAGIC: u64 = 0x041727101980;

mod packet {
    use bytes::{Buf, BufMut};
    use std::net::Ipv4Addr;

    fn connect_request(transaction_id: u32) -> [u8; 16] {
        let mut rv = [0u8; 16];
        BufMut::put_u64(&mut rv.as_mut(), super::BITTORRENT_UDP_MAGIC);
        BufMut::put_u32(&mut rv.as_mut(), 0_u32);
        BufMut::put_u32(&mut rv.as_mut(), transaction_id);
        rv
    }

    fn announce_response(
        connection_id: u64,
        transaction_id: u32,
        info_hash: impl AsRef<[u8; 20]>,
        peer_id: impl AsRef<[u8; 20]>,
        downloaded: u64,
        left: u64,
        uploaded: u64,
        event: u32,
        ip: impl AsRef<Ipv4Addr>,
        key: u32,
        num_want: u32,
        port: u16,
    ) -> [u8; 16] {
        let mut rv = [0u8; 16];
        BufMut::put_u64(&mut rv.as_mut(), connection_id);
        BufMut::put_u32(&mut rv.as_mut(), 1_u32);
        BufMut::put_u32(&mut rv.as_mut(), transaction_id);
        BufMut::put_slice(&mut rv.as_mut(), info_hash.as_ref());
        BufMut::put_slice(&mut rv.as_mut(), peer_id.as_ref());
        BufMut::put_u64(&mut rv.as_mut(), downloaded);
        BufMut::put_u64(&mut rv.as_mut(), left);
        BufMut::put_u64(&mut rv.as_mut(), uploaded);
        BufMut::put_u32(&mut rv.as_mut(), event);
        BufMut::put_slice(&mut rv.as_mut(), ip.as_ref().octets().as_ref());
        BufMut::put_u32(&mut rv.as_mut(), key);
        BufMut::put_u32(&mut rv.as_mut(), num_want);
        BufMut::put_u16(&mut rv.as_mut(), port);
        rv
    }
}

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

pub(crate) async fn send_announce_request(
    socket: &UdpSocket,
    connection_id: u64,
    info_hash: &[u8; 20],
    peer_id: &[u8; 20],
) -> Result<(), Whatever> {
    let transaction_id: u32 = rand::random();

    let mut bytes = BytesMut::with_capacity(98);
    bytes.put_u64(connection_id);
    bytes.put_u32(1);
    bytes.put_u32(transaction_id);
    bytes.put_slice(info_hash);
    bytes.put_slice(peer_id);
    // bytes.put

    Ok(())
}
