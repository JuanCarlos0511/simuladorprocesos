/// Problema clásico Productor-Consumidor con búfer limitado.
///
/// Demostración de sincronización utilizando hilos de la biblioteca estándar,
/// contadores de referencia síncronos (Arc) y exclusión mutua (Mutex).
///
/// Ejecutar: `cargo run --example productor_consumidor`

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

const BUFFER_SIZE: usize = 5;
const ITEMS_TO_PRODUCE: usize = 15;

fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║  Problema Productor-Consumidor               ║");
    println!("║  Búfer limitado: {} espacios                  ║", BUFFER_SIZE);
    println!("║  Items a producir: {}                        ║", ITEMS_TO_PRODUCE);
    println!("╚══════════════════════════════════════════════╝\n");

    // Búfer compartido protegido por Mutex + Condvar
    let buffer: Arc<(Mutex<VecDeque<u32>>, Condvar, Condvar)> = Arc::new((
        Mutex::new(VecDeque::with_capacity(BUFFER_SIZE)),
        Condvar::new(), // not_empty: señala que hay items disponibles
        Condvar::new(), // not_full: señala que hay espacio disponible
    ));

    let buffer_prod = Arc::clone(&buffer);
    let buffer_cons = Arc::clone(&buffer);

    // ── Hilo Productor ──
    let producer = thread::spawn(move || {
        let (lock, not_empty, not_full) = &*buffer_prod;

        for i in 1..=ITEMS_TO_PRODUCE {
            let mut buf = lock.lock().unwrap();

            // Esperar mientras el búfer esté lleno
            while buf.len() >= BUFFER_SIZE {
                println!("  [Productor] Búfer lleno, esperando...");
                buf = not_full.wait(buf).unwrap();
            }

            buf.push_back(i as u32);
            println!("  [Productor] Produjo item {:>2} | Búfer: {}/{}", i, buf.len(), BUFFER_SIZE);

            // Notificar al consumidor que hay datos
            not_empty.notify_one();

            // Simular tiempo de producción
            drop(buf);
            thread::sleep(Duration::from_millis(100));
        }

        println!("\n  [Productor] Finalizó producción de {} items.", ITEMS_TO_PRODUCE);
    });

    // ── Hilo Consumidor ──
    let consumer = thread::spawn(move || {
        let (lock, not_empty, not_full) = &*buffer_cons;
        let mut consumed = 0;

        while consumed < ITEMS_TO_PRODUCE {
            let mut buf = lock.lock().unwrap();

            // Esperar mientras el búfer esté vacío
            while buf.is_empty() {
                buf = not_empty.wait(buf).unwrap();
            }

            let item = buf.pop_front().unwrap();
            consumed += 1;
            println!("  [Consumidor] Consumió item {:>2} | Búfer: {}/{}", item, buf.len(), BUFFER_SIZE);

            // Notificar al productor que hay espacio
            not_full.notify_one();

            // Simular tiempo de consumo
            drop(buf);
            thread::sleep(Duration::from_millis(150));
        }

        println!("\n  [Consumidor] Finalizó consumo de {} items.", consumed);
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    println!("\n✓ Simulación completada exitosamente. Sin deadlocks detectados.");
}
