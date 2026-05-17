/// Pruebas de rendimiento — Latencia del Scheduler.
///
/// Mide el tiempo de ciclo de planificación para evaluar la
/// velocidad de las colas de despacho algorítmico.
///
/// Ejecutar: `cargo test --test scheduler_bench` (o como parte de benches)
/// Nota: Se usa std::time::Instant ya que Criterion requiere dependencia externa.

use std::collections::VecDeque;
use std::time::Instant;
use simulador_procesos::core::process::{PCB, ProcessState};
use simulador_procesos::core::scheduler::{Algorithm, Scheduler};

fn make_pcb(pid: u32, burst: u32) -> PCB {
    PCB {
        pid, name: format!("P{}", pid), state: ProcessState::Ready,
        burst_time: burst, remaining_time: burst, arrival_time: 0,
        priority: 5, memory_mb: 64.0, io_burst: None,
        finish_time: None, turnaround_time: None, waiting_time: None,
    }
}

fn bench_scheduler(algo: Algorithm, process_count: usize) {
    let mut scheduler = Scheduler::new(algo, 4);

    // Populate ready queue
    for i in 0..process_count {
        scheduler.add_process(make_pcb(i as u32 + 0x00A2, (i as u32 % 50) + 5));
    }

    // Measure dispatch time
    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        let _ = scheduler.next_process();
        // Re-add a process to keep the queue populated
        scheduler.add_process(make_pcb(0xFFFF, 10));
    }

    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations;

    println!(
        "  [{:>15}] {} dispatches con {} procesos: {:.2}ms total, {}ns/dispatch",
        algo.label(), iterations, process_count,
        elapsed.as_secs_f64() * 1000.0, avg_ns
    );
}

fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║  Benchmark de Latencia del Scheduler         ║");
    println!("╚══════════════════════════════════════════════╝\n");

    let sizes = [10, 50, 100, 500];
    let algorithms = [Algorithm::FCFS, Algorithm::SJF, Algorithm::RoundRobin, Algorithm::Priority];

    for &size in &sizes {
        println!("  ── Cola de {} procesos ──", size);
        for &algo in &algorithms {
            bench_scheduler(algo, size);
        }
        println!();
    }

    println!("✓ Benchmark completado.");
}
