use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

/// Signature d'un handler : reçoit le payload brut, retourne la réponse brute.
pub type HandlerFn = Arc<dyn Fn(Vec<u8>) -> Vec<u8> + Send + Sync>;

/// Serveur de protocole binaire réseau.
///
/// Format de trame :
/// ┌─────────────────────────┬──────────────────────┬───────────────────┐
/// │  4 octets (big-endian)  │  2 octets (big-endian)│   N octets        │
/// │  longueur du payload    │  type de message      │   payload binaire │
/// └─────────────────────────┴──────────────────────┴───────────────────┘
pub struct BinProtoServer {
    handlers: Arc<Mutex<HashMap<u16, HandlerFn>>>,
}

impl BinProtoServer {
    /// Crée un nouveau serveur sans handlers enregistrés.
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Enregistre un handler pour un type de message donné.
    pub async fn register_handler(&self, msg_type: u16, handler: HandlerFn) {
        self.handlers.lock().await.insert(msg_type, handler);
    }

    /// Démarre l'écoute sur `addr` et traite les connexions entrantes.
    pub async fn listen(&self, addr: &str) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("[BinProto] Écoute sur {addr}");

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    println!("[BinProto] Nouvelle connexion : {peer}");
                    let handlers = Arc::clone(&self.handlers);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, handlers).await {
                            eprintln!("[BinProto] Connexion {peer} terminée : {e}");
                        }
                    });
                }
                Err(e) => {
                    eprintln!("[BinProto] Erreur accept() : {e}");
                    return Err(e);
                }
            }
        }
    }
}

impl Default for BinProtoServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Gère une connexion TCP unique jusqu'à sa fermeture.
async fn handle_connection(
    mut stream: TcpStream,
    handlers: Arc<Mutex<HashMap<u16, HandlerFn>>>,
) -> tokio::io::Result<()> {
    loop {
        // Lecture de l'en-tête (6 octets)
        let mut header = [0u8; 6];
        match stream.read_exact(&mut header).await {
            Ok(_) => {}
            Err(e) if is_connection_closed(&e) => return Ok(()),
            Err(e) => return Err(e),
        }

        let payload_len = u32::from_be_bytes(header[0..4].try_into().unwrap()) as usize;
        let msg_type = u16::from_be_bytes(header[4..6].try_into().unwrap());

        // Lecture du payload
        let mut payload = vec![0u8; payload_len];
        if payload_len > 0 {
            match stream.read_exact(&mut payload).await {
                Ok(_) => {}
                Err(e) if is_connection_closed(&e) => return Ok(()),
                Err(e) => return Err(e),
            }
        }

        // Dispatch vers le handler
        let response_payload: Vec<u8> = {
            let guard = handlers.lock().await;
            match guard.get(&msg_type) {
                Some(handler) => {
                    let handler = Arc::clone(handler);
                    drop(guard);
                    handler(payload)
                }
                None => {
                    eprintln!("[BinProto] Type inconnu : 0x{msg_type:04X}");
                    Vec::new()
                }
            }
        };

        // Envoi de la réponse
        let frame = build_frame(msg_type, &response_payload);

        match stream.write_all(&frame).await {
            Ok(_) => {}
            Err(e) if is_connection_closed(&e) => return Ok(()),
            Err(e) => return Err(e),
        }
    }
}

pub fn build_frame(msg_type: u16, payload: &[u8]) -> Vec<u8> {
    let resp_len = payload.len() as u32;
    let mut frame = Vec::with_capacity(6 + payload.len());
    frame.extend_from_slice(&resp_len.to_be_bytes());
    frame.extend_from_slice(&msg_type.to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn is_connection_closed(e: &tokio::io::Error) -> bool {
    use tokio::io::ErrorKind;
    matches!(
        e.kind(),
        ErrorKind::UnexpectedEof
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_frame() {
        let payload = vec![1, 2, 3, 4];
        let frame = build_frame(0x1234, &payload);
        assert_eq!(&frame[0..4], &(4u32).to_be_bytes());
        assert_eq!(&frame[4..6], &(0x1234u16).to_be_bytes());
        assert_eq!(&frame[6..], payload.as_slice());
    }
}
