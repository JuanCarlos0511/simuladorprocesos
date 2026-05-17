/// Demostración de Round Robin en consola CLI.
///
/// Simula la reducción secuencial de ráfagas de tiempo mediante
/// un Quantum fijo y predecible. Muestra el estado de los procesos
/// en cada ronda de planificación.
///
/// Ejecutar: `cargo run --example round_robin_demo`

fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║  Demostración Round Robin — CLI              ║");
    println!("╚══════════════════════════════════════════════╝\n");

    let quantum: u32 = 4;
    println!("  Quantum configurado: {} ms\n", quantum);

    // Procesos de ejemplo: (nombre, burst_time)
    let mut processes: Vec<(&str, u32, u32)> = vec![
        ("nginx_worker",     12, 12),
        ("db_query",          8,  8),
        ("log_rotate",        6,  6),
        ("cache_invalidator", 10, 10),
    ];

    // Header
    println!("  {:<20} {:<10} {:<12}", "Proceso", "BT Total", "BT Restante");
    println!("  {}", "─".repeat(42));
    for (name, bt, _) in &processes {
        println!("  {:<20} {:<10} {:<12}", name, bt, bt);
    }

    let mut clock: u32 = 0;
    let mut round = 0;

    println!("\n  ═══ Inicio de Simulación ═══\n");

    while processes.iter().any(|(_, _, remaining)| *remaining > 0) {
        round += 1;
        println!("  ┌── Ronda {} ──────────────────────────────┐", round);

        for (name, _bt, remaining) in processes.iter_mut() {
            if *remaining == 0 {
                continue;
            }

            let exec_time = (*remaining).min(quantum);
            let old = *remaining;
            *remaining -= exec_time;
            clock += exec_time;

            let status = if *remaining == 0 { "✓ Terminado" } else { "→ En espera" };
            println!("  │ {:<18} ejecutó {}ms ({} → {}) {}",
                name, exec_time, old, remaining, status);
        }

        println!("  │ Reloj del sistema: {} ms", clock);
        println!("  └────────────────────────────────────────┘\n");
    }

    println!("  ═══ Simulación Completada ═══");
    println!("  Tiempo total: {} ms", clock);
    println!("  Rondas ejecutadas: {}", round);
    println!("\n✓ Todos los procesos finalizaron correctamente.");
}
