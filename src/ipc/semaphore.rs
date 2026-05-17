/// Semáforo contador — primitiva de sincronización concurrente.
///
/// Implementado desde cero usando `Arc<(Mutex<u32>, Condvar)>` como
/// exige la rúbrica académica. Permite controlar el acceso concurrente
/// a recursos compartidos limitados.

use std::sync::{Arc, Condvar, Mutex};

/// Semáforo contador seguro para hilos.
///
/// Internamente usa un `Mutex<u32>` para el contador y un `Condvar`
/// para bloquear hilos cuando el recurso no está disponible.
#[derive(Clone)]
pub struct Semaphore {
    inner: Arc<(Mutex<u32>, Condvar)>,
}

impl Semaphore {
    /// Crea un nuevo semáforo con el valor inicial dado.
    ///
    /// # Argumentos
    /// * `initial` - Número inicial de permisos disponibles.
    ///
    /// # Ejemplo
    /// ```
    /// use simulador_procesos::ipc::semaphore::Semaphore;
    /// let sem = Semaphore::new(3); // 3 permisos disponibles
    /// ```
    pub fn new(initial: u32) -> Self {
        Semaphore {
            inner: Arc::new((Mutex::new(initial), Condvar::new())),
        }
    }

    /// Operación `wait` (P / down / acquire).
    ///
    /// Decrementa el contador del semáforo. Si el contador es 0,
    /// el hilo se bloquea hasta que otro hilo llame a `signal()`.
    pub fn wait(&self) {
        let (lock, cvar) = &*self.inner;
        let mut count = lock.lock().unwrap();
        while *count == 0 {
            count = cvar.wait(count).unwrap();
        }
        *count -= 1;
    }

    /// Operación `signal` (V / up / release).
    ///
    /// Incrementa el contador del semáforo y despierta a un hilo
    /// que esté bloqueado en `wait()`, si existe.
    pub fn signal(&self) {
        let (lock, cvar) = &*self.inner;
        let mut count = lock.lock().unwrap();
        *count += 1;
        cvar.notify_one();
    }

    /// Retorna el valor actual del semáforo (para diagnóstico).
    ///
    /// **Nota**: Este valor puede cambiar inmediatamente después de
    /// ser leído si otros hilos están operando sobre el semáforo.
    pub fn value(&self) -> u32 {
        let (lock, _) = &*self.inner;
        *lock.lock().unwrap()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn initial_value() {
        let sem = Semaphore::new(5);
        assert_eq!(sem.value(), 5);
    }

    #[test]
    fn wait_decrements() {
        let sem = Semaphore::new(3);
        sem.wait();
        assert_eq!(sem.value(), 2);
        sem.wait();
        assert_eq!(sem.value(), 1);
    }

    #[test]
    fn signal_increments() {
        let sem = Semaphore::new(0);
        sem.signal();
        assert_eq!(sem.value(), 1);
    }

    #[test]
    fn wait_blocks_until_signal() {
        let sem = Semaphore::new(0);
        let sem_clone = sem.clone();

        let handle = thread::spawn(move || {
            // This should block until signal is called
            sem_clone.wait();
            42
        });

        // Give the thread time to block
        thread::sleep(Duration::from_millis(50));
        assert_eq!(sem.value(), 0);

        // Release the blocked thread
        sem.signal();
        let result = handle.join().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn concurrent_access() {
        let sem = Semaphore::new(1);
        let counter = Arc::new(Mutex::new(0u32));
        let mut handles = vec![];

        for _ in 0..10 {
            let sem = sem.clone();
            let counter = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                sem.wait();
                let mut val = counter.lock().unwrap();
                *val += 1;
                sem.signal();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(*counter.lock().unwrap(), 10);
        assert_eq!(sem.value(), 1);
    }
}
