use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Client du protocole binaire réseau
pub struct BinProtoClient {
    stream: TcpStream,
}

impl BinProtoClient {
    /// Connecte le client à l'adresse spécifiée
    pub async fn connect(addr: &str) -> tokio::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }

    /// Envoie une trame et attend la réponse
    pub async fn send_raw(
        &mut self,
        msg_type: u16,
        payload: Vec<u8>,
    ) -> tokio::io::Result<(u16, Vec<u8>)> {
        // Construction de la trame
        let mut frame = Vec::with_capacity(6 + payload.len());
        frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        frame.extend_from_slice(&msg_type.to_be_bytes());
        frame.extend_from_slice(&payload);

        // Envoi
        self.stream.write_all(&frame).await?;
        self.stream.flush().await?;

        // Lecture de la réponse
        let mut header = [0u8; 6];
        self.stream.read_exact(&mut header).await?;

        let payload_len = u32::from_be_bytes(header[0..4].try_into().unwrap()) as usize;
        let resp_type = u16::from_be_bytes(header[4..6].try_into().unwrap());

        let mut response_payload = vec![0u8; payload_len];
        if payload_len > 0 {
            self.stream.read_exact(&mut response_payload).await?;
        }

        Ok((resp_type, response_payload))
    }

    /// Envoie une trame sans attendre de réponse (fire-and-forget)
    pub async fn send_one_way(&mut self, msg_type: u16, payload: Vec<u8>) -> tokio::io::Result<()> {
        let mut frame = Vec::with_capacity(6 + payload.len());
        frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        frame.extend_from_slice(&msg_type.to_be_bytes());
        frame.extend_from_slice(&payload);

        self.stream.write_all(&frame).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Version générique utilisant les traits Encode (quand binproto-core sera disponible)
    // pub async fn send<T: binproto_core::Encode>(
    //     &mut self,
    //     msg_type: u16,
    //     msg: &T,
    // ) -> tokio::io::Result<(u16, Vec<u8>)> {
    //     let mut buf = Vec::new();
    //     msg.encode(&mut buf);
    //     self.send_raw(msg_type, buf).await
    // }

    pub fn peer_addr(&self) -> tokio::io::Result<std::net::SocketAddr> {
        self.stream.peer_addr()
    }
}