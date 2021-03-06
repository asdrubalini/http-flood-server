use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use byte_unit::Byte;
use rand::prelude::SmallRng;
use rand::{RngCore, SeedableRng};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

const RESPONSE_HEADER: &str = include_str!("./response-header.txt");
const BUFFER_SIZE: usize = 1024;

async fn http_send_header(socket: &mut TcpStream) -> Result<u128, std::io::Error> {
    socket.write_all(RESPONSE_HEADER.as_bytes()).await?;
    Ok(RESPONSE_HEADER.as_bytes().len() as u128)
}

async fn handle_client(
    mut socket: TcpStream,
    address: SocketAddr,
    resume: Resume,
) -> Result<(), std::io::Error> {
    let bytes_written = http_send_header(&mut socket).await?;
    resume.increment_address_by(&address, bytes_written).await;

    println!("[{address}] started serving");

    let mut small_rng = SmallRng::from_entropy();
    let mut buffer = vec![0; BUFFER_SIZE];

    loop {
        small_rng.fill_bytes(&mut buffer);
        let write_result = socket.write_all(&buffer).await;

        if let Err(error) = write_result {
            println!("[{address}] error while writing: {error}");
            break;
        } else {
            resume
                .increment_address_by(&address, BUFFER_SIZE as u128)
                .await;
        }
    }

    Ok(())
}

#[derive(Debug, Default)]
pub struct ClientResume {
    bytes_sent: u128,
}

#[derive(Default, Clone)]
pub struct Resume(Arc<RwLock<HashMap<SocketAddr, ClientResume>>>);

impl Resume {
    async fn increment_address_by(&self, address: &SocketAddr, increment_amount: u128) {
        let mut inner = self.0.write().await;

        // Get or insert default
        let client_resume = match inner.entry(*address) {
            std::collections::hash_map::Entry::Occupied(o) => o.into_mut(),
            std::collections::hash_map::Entry::Vacant(v) => v.insert(ClientResume::default()),
        };

        client_resume.bytes_sent += increment_amount;
    }

    async fn println_debug(&self) {
        let inner = self.0.read().await;

        let clients_count = inner.len();
        let bytes_total: u128 = inner.iter().map(|(_addr, resume)| resume.bytes_sent).sum();
        let bytes_pretty = Byte::from_bytes(bytes_total).get_appropriate_unit(false);

        println!("clients_count: {clients_count}, bytes_total: {bytes_pretty}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let resume = Resume::default();

    // Logging task
    {
        let resume = resume.clone();
        tokio::spawn(async move {
            loop {
                resume.println_debug().await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    // Handle clients
    loop {
        let (socket, address) = listener.accept().await?;
        let resume = resume.clone();

        tokio::spawn(async move {
            match handle_client(socket, address, resume).await {
                Ok(_) => println!("done with client"),
                Err(_) => println!("err"),
            }
        });
    }
}
