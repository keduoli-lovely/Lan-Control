use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures::{StreamExt, SinkExt};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
pub async fn start_vnc_proxy(
    ws_addr: &str,
    vnc_addr: &str,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(ws_addr).await?;
    println!("VNC Proxy listening on {}", ws_addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let vnc_addr = vnc_addr.to_string();

        tokio::spawn(async move {
            if let Err(e) = handle_conn(stream, &vnc_addr).await {
                eprintln!("connection error: {}", e);
            }
        });
    }
}

async fn handle_conn(
    stream: TcpStream,
    vnc_addr: &str,
) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_write, mut ws_read) = ws_stream.split();

    let tcp = TcpStream::connect(vnc_addr).await?;
    let (mut tcp_read, mut tcp_write) = tcp.into_split();

    // WS → TCP
    let ws_to_tcp = async {
        while let Some(msg) = ws_read.next().await {
            let msg = msg?;
            if msg.is_binary() {
                tcp_write.write_all(&msg.into_data()).await?;
            }
        }
        Ok::<_, anyhow::Error>(())
    };

    // TCP → WS
    let tcp_to_ws = async {
        let mut buf = [0u8; 8192];
        loop {
            let n = tcp_read.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            ws_write
                .send(tokio_tungstenite::tungstenite::Message::Binary(
                    buf[..n].to_vec(),
                ))
                .await?;
        }
        Ok::<_, anyhow::Error>(())
    };

    tokio::select! {
        _ = ws_to_tcp => {},
        _ = tcp_to_ws => {},
    }

    Ok(())
}