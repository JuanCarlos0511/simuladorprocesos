/// Canal de comunicación interproceso — envoltura idiomática sobre `std::sync::mpsc`.
///
/// Provee una interfaz simplificada para el paso asíncrono de mensajes
/// seguros entre hilos productores y consumidores.

use std::sync::mpsc;

/// Extremo transmisor del canal. Puede ser clonado para múltiples productores.
pub struct Sender<T> {
    inner: mpsc::Sender<T>,
}

/// Extremo receptor del canal. Solo puede haber un consumidor.
pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

/// Crea un nuevo canal asíncrono de mensajes.
///
/// Retorna una tupla `(Sender<T>, Receiver<T>)` donde el `Sender` puede
/// ser clonado para múltiples productores (MPSC).
///
/// # Ejemplo
/// ```
/// use simulador_procesos::ipc::channel;
/// let (tx, rx) = channel::create::<String>();
/// tx.send("Hola desde otro hilo".to_string()).unwrap();
/// let msg = rx.recv().unwrap();
/// assert_eq!(msg, "Hola desde otro hilo");
/// ```
pub fn create<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::channel();
    (Sender { inner: tx }, Receiver { inner: rx })
}

impl<T> Sender<T> {
    /// Envía un mensaje a través del canal.
    ///
    /// Retorna `Ok(())` si el receptor aún existe, o `Err` si fue desconectado.
    pub fn send(&self, value: T) -> Result<(), mpsc::SendError<T>> {
        self.inner.send(value)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Receiver<T> {
    /// Recibe un mensaje del canal, bloqueando hasta que uno esté disponible.
    ///
    /// Retorna `Err` si todos los transmisores fueron desconectados.
    pub fn recv(&self) -> Result<T, mpsc::RecvError> {
        self.inner.recv()
    }

    /// Intenta recibir un mensaje sin bloquear.
    ///
    /// Retorna `Ok(T)` si hay un mensaje disponible, o `Err` si el canal
    /// está vacío o desconectado.
    pub fn try_recv(&self) -> Result<T, mpsc::TryRecvError> {
        self.inner.try_recv()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn send_and_receive_single_message() {
        let (tx, rx) = create::<String>();
        tx.send("test_message".to_string()).unwrap();
        assert_eq!(rx.recv().unwrap(), "test_message");
    }

    #[test]
    fn send_from_multiple_producers() {
        let (tx, rx) = create::<i32>();
        let tx2 = tx.clone();

        thread::spawn(move || {
            tx.send(1).unwrap();
        });

        thread::spawn(move || {
            tx2.send(2).unwrap();
        });

        let mut received = vec![rx.recv().unwrap(), rx.recv().unwrap()];
        received.sort();
        assert_eq!(received, vec![1, 2]);
    }

    #[test]
    fn try_recv_empty_channel() {
        let (_tx, rx) = create::<i32>();
        assert!(rx.try_recv().is_err());
    }
}
