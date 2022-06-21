use bytes::{Buf, BufMut, BytesMut, Bytes};
use snafu::{whatever, Whatever};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::torrent::InfoHash;

const BITTORRENT_UDP_MAGIC: u64 = 0x41727101980;

pub(crate) mod packet {
    use bytes::{Buf, BufMut, BytesMut, Bytes};
    use std::{borrow::Borrow, net::Ipv4Addr};

    use crate::torrent::InfoHash;

    pub(crate) fn connect_request(transaction_id: u32) -> [u8; 16] {
        let mut rv = [0_u8; 16];
        let mut ptr = &mut rv[..];
        ptr.put_u64(super::BITTORRENT_UDP_MAGIC);
        ptr.put_u32(0);
        ptr.put_u32(transaction_id);
        rv
    }

    pub(crate) fn announce_response(
        connection_id: u64,
        transaction_id: u32,
        info_hash: impl Borrow<InfoHash>,
        peer_id: impl Borrow<[u8; 20]>,
        downloaded: u64,
        left: u64,
        uploaded: u64,
        event: Option<u32>,
        ip: Option<impl Borrow<Ipv4Addr>>,
        key: u32,
        num_want: Option<i32>,
        port: u16,
    ) -> [u8; 98] {
        let mut rv = [0u8; 98];
        let mut ptr = &mut rv[..];
        ptr.put_u64(connection_id);
        ptr.put_u32(1_u32);
        ptr.put_u32(transaction_id);
        ptr.put_slice(info_hash.borrow());
        ptr.put_slice(peer_id.borrow());
        ptr.put_u64(downloaded);
        ptr.put_u64(left);
        ptr.put_u64(uploaded);
        ptr.put_u32(event.unwrap_or(0));
        ptr.put_slice(
            match ip {
                Some(ip) => ip.borrow().octets(),
                None => [0_u8; 4],
            }
            .as_ref(),
        );
        ptr.put_u32(key);
        ptr.put_i32(num_want.unwrap_or(-1));
        ptr.put_u16(port);
        rv
    }
}

// pub(crate) async fn send_connect_request(
//     socket: &UdpSocket,
//     addr: SocketAddr,
// ) -> Result<u32, Whatever> {
//     let transaction_id: u32 = rand::random();

//     let mut bytes = BytesMut::with_capacity(16);
//     bytes.put_u64(BITTORRENT_UDP_MAGIC);
//     bytes.put_u32(0);
//     bytes.put_u32(transaction_id);

//     match socket.send_to(bytes.as_ref(), addr).await {
//         Ok(_) => Ok(transaction_id),
//         Err(e) => whatever!("{}", e),
//     }
// }

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

// pub(crate) async fn send_announce_request(
//     socket: &UdpSocket,
//     connection_id: u64,
//     info_hash: &[u8; 20],
//     peer_id: &[u8; 20],
// ) -> Result<(), Whatever> {
//     let transaction_id: u32 = rand::random();

//     let mut bytes = BytesMut::with_capacity(98);
//     bytes.put_u64(connection_id);
//     bytes.put_u32(1);
//     bytes.put_u32(transaction_id);
//     bytes.put_slice(info_hash);
//     bytes.put_slice(peer_id);
//     // bytes.put

//     Ok(())
// }
