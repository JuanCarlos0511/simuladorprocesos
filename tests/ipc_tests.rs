/// Pruebas de integración de caja negra — IPC (Canales y Semáforos).
///
/// Valida la comunicación concurrente entre hilos mediante las
/// primitivas de sincronización del módulo IPC.

use simulador_procesos::ipc::channel;
use simulador_procesos::ipc::semaphore::Semaphore;

use std::sync::{Arc, Mutex};
use std::thread;

// ─── Tests de Canal ──────────────────────────────────────────────────────────

#[test]
fn channel_single_message() {
    let (tx, rx) = channel::create::<String>();
    tx.send("hello".to_string()).unwrap();
    assert_eq!(rx.recv().unwrap(), "hello");
}

#[test]
fn channel_multiple_messages() {
    let (tx, rx) = channel::create::<u32>();
    for i in 0..100 {
        tx.send(i).unwrap();
    }
    for i in 0..100 {
        assert_eq!(rx.recv().unwrap(), i);
    }
}

#[test]
fn channel_multiple_senders() {
    let (tx, rx) = channel::create::<u32>();
    let tx2 = tx.clone();
    let tx3 = tx.clone();

    thread::spawn(move || { tx.send(1).unwrap(); });
    thread::spawn(move || { tx2.send(2).unwrap(); });
    thread::spawn(move || { tx3.send(3).unwrap(); });

    let mut received = vec![];
    for _ in 0..3 {
        received.push(rx.recv().unwrap());
    }
    received.sort();
    assert_eq!(received, vec![1, 2, 3]);
}

// ─── Tests de Semáforo ───────────────────────────────────────────────────────

#[test]
fn semaphore_initial_value() {
    let sem = Semaphore::new(3);
    assert_eq!(sem.value(), 3);
}

#[test]
fn semaphore_wait_signal_sequence() {
    let sem = Semaphore::new(2);
    sem.wait();
    assert_eq!(sem.value(), 1);
    sem.wait();
    assert_eq!(sem.value(), 0);
    sem.signal();
    assert_eq!(sem.value(), 1);
}

#[test]
fn semaphore_concurrent_counter() {
    let sem = Semaphore::new(1); // binary semaphore (mutex)
    let counter = Arc::new(Mutex::new(0u32));
    let mut handles = vec![];

    for _ in 0..20 {
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

    assert_eq!(*counter.lock().unwrap(), 20);
}

#[test]
fn semaphore_blocks_correctly() {
    let sem = Semaphore::new(0);
    let sem_clone = sem.clone();
    let flag = Arc::new(Mutex::new(false));
    let flag_clone = Arc::clone(&flag);

    let handle = thread::spawn(move || {
        sem_clone.wait(); // should block
        *flag_clone.lock().unwrap() = true;
    });

    thread::sleep(std::time::Duration::from_millis(50));
    assert!(!*flag.lock().unwrap()); // still blocked

    sem.signal(); // unblock
    handle.join().unwrap();
    assert!(*flag.lock().unwrap()); // now unblocked
}
