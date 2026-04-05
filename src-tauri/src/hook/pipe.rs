use std::ffi::c_void;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use windows::core::PCWSTR;
use windows::Win32::Foundation::*;
use windows::Win32::Security::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Pipes::*;

const PIPE_NAME: &str = "\\\\.\\pipe\\broken_latch";
const BUFFER_SIZE: u32 = 512;

pub enum PipeMessage {
    DX11Hooked,
    DX12Hooked,
    HookFailed,
    Custom(String),
}

pub struct PipeServer {
    receiver: Receiver<PipeMessage>,
}

impl PipeServer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();

        // Start pipe server thread
        thread::spawn(move || {
            if let Err(e) = run_pipe_server(tx) {
                eprintln!("Pipe server error: {}", e);
            }
        });

        Ok(Self { receiver: rx })
    }

    pub fn try_recv(&self) -> Option<PipeMessage> {
        self.receiver.try_recv().ok()
    }

    pub fn recv(&self) -> Option<PipeMessage> {
        self.receiver.recv().ok()
    }
}

fn run_pipe_server(tx: Sender<PipeMessage>) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        loop {
            // Create named pipe
            let pipe_name_wide: Vec<u16> = PIPE_NAME
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let h_pipe = CreateNamedPipeW(
                PCWSTR(pipe_name_wide.as_ptr()),
                PIPE_ACCESS_INBOUND,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                1, // Max instances
                BUFFER_SIZE,
                BUFFER_SIZE,
                0,
                None,
            )?;

            if h_pipe.is_invalid() {
                return Err("Failed to create named pipe".into());
            }

            // Wait for client connection
            let connected = ConnectNamedPipe(h_pipe, None).is_ok()
                || GetLastError() == ERROR_PIPE_CONNECTED;

            if connected {
                // Read message from pipe
                let mut buffer = vec![0u8; BUFFER_SIZE as usize];
                let mut bytes_read = 0;

                let read_result = ReadFile(
                    h_pipe,
                    Some(buffer.as_mut_ptr() as *mut c_void),
                    BUFFER_SIZE,
                    Some(&mut bytes_read),
                    None,
                );

                if read_result.is_ok() && bytes_read > 0 {
                    let message =
                        String::from_utf8_lossy(&buffer[..bytes_read as usize]).to_string();

                    let pipe_msg = match message.as_str() {
                        "DX11_HOOKED" => PipeMessage::DX11Hooked,
                        "DX12_HOOKED" => PipeMessage::DX12Hooked,
                        "HOOK_FAILED" => PipeMessage::HookFailed,
                        _ => PipeMessage::Custom(message),
                    };

                    let _ = tx.send(pipe_msg);
                }

                DisconnectNamedPipe(h_pipe);
            }

            CloseHandle(h_pipe);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe_server_creation() {
        let server = PipeServer::new();
        assert!(server.is_ok());
    }
}
