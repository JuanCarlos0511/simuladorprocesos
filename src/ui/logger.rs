/// Sistema de logging cronológico de eventos del simulador.
///
/// Registra eventos desacoplados a través de canales concurrentes,
/// almacenando marcas de tiempo, PIDs y motivos de salida.

use std::sync::mpsc;
use std::time::Instant;

/// Motivo de finalización de un proceso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitReason {
    Normal,
    Error,
    Deadlock,
    UserKill,
}

impl std::fmt::Display for ExitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitReason::Normal => write!(f, "Normal"),
            ExitReason::Error => write!(f, "Error"),
            ExitReason::Deadlock => write!(f, "Deadlock"),
            ExitReason::UserKill => write!(f, "UserKill"),
        }
    }
}

/// Evento registrado en el historial del simulador.
#[derive(Debug, Clone)]
pub struct LogEvent {
    pub timestamp_ms: u64,
    pub pid: u32,
    pub message: String,
    pub exit_reason: Option<ExitReason>,
}

/// Logger concurrente basado en canales para registro de eventos.
pub struct EventLogger {
    receiver: mpsc::Receiver<LogEvent>,
    pub history: Vec<LogEvent>,
    start_time: Instant,
}

/// Handle para enviar eventos al logger desde cualquier hilo.
#[derive(Clone)]
pub struct LogHandle {
    sender: mpsc::Sender<LogEvent>,
    start_time: Instant,
}

impl LogHandle {
    pub fn log(&self, pid: u32, message: String) {
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        let _ = self.sender.send(LogEvent {
            timestamp_ms: elapsed, pid, message, exit_reason: None,
        });
    }

    pub fn log_exit(&self, pid: u32, reason: ExitReason) {
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        let _ = self.sender.send(LogEvent {
            timestamp_ms: elapsed, pid,
            message: format!("Proceso 0x{:04X} terminado: {}", pid, reason),
            exit_reason: Some(reason),
        });
    }
}

impl EventLogger {
    pub fn new() -> (Self, LogHandle) {
        let (sender, receiver) = mpsc::channel();
        let start_time = Instant::now();
        let logger = EventLogger { receiver, history: Vec::new(), start_time };
        let handle = LogHandle { sender, start_time };
        (logger, handle)
    }

    pub fn drain(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            self.history.push(event);
        }
    }

    pub fn events(&self) -> &[LogEvent] { &self.history }

    pub fn last_events(&self, n: usize) -> &[LogEvent] {
        let start = self.history.len().saturating_sub(n);
        &self.history[start..]
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

impl Default for EventLogger {
    fn default() -> Self { Self::new().0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_and_drain_events() {
        let (mut logger, handle) = EventLogger::new();
        handle.log(0x00A2, "Proceso creado".to_string());
        handle.log(0x00A3, "Proceso en ejecución".to_string());
        logger.drain();
        assert_eq!(logger.events().len(), 2);
        assert_eq!(logger.events()[0].pid, 0x00A2);
    }

    #[test]
    fn log_exit_event() {
        let (mut logger, handle) = EventLogger::new();
        handle.log_exit(0x00A2, ExitReason::Normal);
        handle.log_exit(0x00A3, ExitReason::UserKill);
        logger.drain();
        assert_eq!(logger.events().len(), 2);
        assert_eq!(logger.events()[0].exit_reason, Some(ExitReason::Normal));
    }
}
