/// KERNEL_OS — Simulador de Gestión de Procesos
///
/// Entry point: initializes the Slint UI, connects callbacks to the
/// simulation engine, and runs the main event loop with a Timer.
/// Supports --cli flag for interactive console mode.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use slint::{ModelRc, SharedString, Timer, TimerMode, VecModel};

// Import from the library crate (3-layer architecture)
use simulador_procesos::core::process::PCB;
use simulador_procesos::core::scheduler::{Algorithm, GanttEntry, LogEntry};
use simulador_procesos::core::simulation::{SimulationEngine, SimConfig, SimState as EngineState};
use simulador_procesos::core::metrics;

slint::include_modules!();

// ─── Type Conversion Helpers ─────────────────────────────────────────────────

/// Converts a Rust PCB to a Slint ProcessItem struct.
fn pcb_to_slint(pcb: &PCB) -> ProcessItem {
    ProcessItem {
        pid: pcb.pid as i32,
        pid_hex: SharedString::from(pcb.pid_hex()),
        name: SharedString::from(pcb.name.as_str()),
        state: pcb.state as i32,
        priority: pcb.priority as i32,
        priority_label: SharedString::from(pcb.priority_label()),
        arrival_time: pcb.arrival_time as i32,
        burst_time: pcb.burst_time as i32,
        remaining_time: pcb.remaining_time as i32,
        memory_mb: pcb.memory_mb,
        io_remaining: pcb.io_burst.unwrap_or(0) as i32,
        finish_time: pcb.finish_time.unwrap_or(0) as i32,
        turnaround_time: pcb.turnaround_time.unwrap_or(0) as i32,
        waiting_time: pcb.waiting_time.unwrap_or(0) as i32,
        selected: false,
    }
}

/// Converts a Rust LogEntry to a Slint LogItem.
fn log_to_slint(entry: &LogEntry) -> LogItem {
    LogItem {
        timestamp: entry.timestamp as i32,
        message: SharedString::from(entry.message.as_str()),
    }
}

/// Converts a Rust GanttEntry to a Slint GanttItem.
fn gantt_to_slint(entry: &GanttEntry, color_map: &std::collections::HashMap<u32, i32>) -> GanttItem {
    let color_idx = color_map.get(&entry.pid).copied().unwrap_or(0);
    GanttItem {
        pid: entry.pid as i32,
        pid_label: SharedString::from(format!("P{}", entry.pid - 0x00A1)),
        start_time: entry.start as i32,
        end_time: entry.end as i32,
        color_index: color_idx,
    }
}

/// Converts SimulationMetrics to Slint MetricsData.
fn metrics_to_slint(m: &metrics::SimulationMetrics) -> MetricsData {
    MetricsData {
        total_time: m.total_time as i32,
        cpu_utilization: m.cpu_utilization,
        avg_waiting: m.avg_waiting_time,
        avg_turnaround: m.avg_turnaround_time,
        delta_waiting: m.delta_waiting,
        delta_turnaround: m.delta_turnaround,
    }
}

/// Pushes all simulation state to the UI.
fn update_ui(ui: &AppWindow, sim: &SimulationEngine) {
    // Ready queue
    let ready: Vec<ProcessItem> = sim.ready_queue().iter().map(pcb_to_slint).collect();
    ui.set_ready_queue(ModelRc::new(VecModel::from(ready)));

    // Blocked queue
    let blocked: Vec<ProcessItem> = sim.blocked_queue().iter().map(pcb_to_slint).collect();
    ui.set_blocked_queue(ModelRc::new(VecModel::from(blocked)));

    // Terminated (exclude kernel daemon)
    let terminated: Vec<ProcessItem> = sim
        .terminated()
        .iter()
        .filter(|p| !p.is_kernel_daemon())
        .map(pcb_to_slint)
        .collect();
    ui.set_terminated_list(ModelRc::new(VecModel::from(terminated)));

    // Current process
    if let Some(proc) = sim.current_process() {
        ui.set_current_process(pcb_to_slint(proc));
        ui.set_has_current_process(true);
    } else {
        ui.set_has_current_process(false);
    }

    // All processes for table
    let all: Vec<ProcessItem> = sim
        .all_processes_for_table()
        .iter()
        .filter(|p| !p.is_kernel_daemon())
        .map(pcb_to_slint)
        .collect();
    let total = all.len() as i32;
    ui.set_all_processes(ModelRc::new(VecModel::from(all)));

    // Sys log (last 100 entries)
    let log: Vec<LogItem> = sim
        .sys_log()
        .iter()
        .rev()
        .take(100)
        .rev()
        .map(log_to_slint)
        .collect();
    ui.set_sys_log(ModelRc::new(VecModel::from(log)));

    // Gantt entries
    let mut color_map = std::collections::HashMap::new();
    let mut color_counter = 0i32;
    let gantt: Vec<GanttItem> = sim
        .gantt_log()
        .iter()
        .map(|e| {
            if !color_map.contains_key(&e.pid) {
                color_map.insert(e.pid, color_counter % 10);
                color_counter += 1;
            }
            gantt_to_slint(e, &color_map)
        })
        .collect();
    let max_time = sim.gantt_log().iter().map(|e| e.end).max().unwrap_or(50) as i32;
    ui.set_gantt_entries(ModelRc::new(VecModel::from(gantt)));
    ui.set_max_gantt_time(max_time);

    // Metrics
    ui.set_metrics(metrics_to_slint(&sim.current_metrics));

    // Global state
    let state = ui.global::<SimState>();
    state.set_clock(sim.clock() as i32);
    state.set_cpu_load(sim.cpu_load());
    state.set_memory_used_mb(sim.memory_used());
    state.set_total_processes(sim.active_count() as i32);
    state.set_alu_cycles(sim.clock() as i32);
    state.set_sim_running(sim.state == EngineState::Running);
    state.set_sim_paused(sim.state == EngineState::Paused);
    state.set_sim_completed(sim.state == EngineState::Completed);
    state.set_quantum_remaining(sim.quantum_remaining() as i32);
    state.set_table_total(total);

    // Auto-navigate to Gantt on completion
    if sim.state == EngineState::Completed && state.get_current_screen() != 3 {
        state.set_current_screen(3);
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() -> Result<(), slint::PlatformError> {
    // Check for --cli flag
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--cli") {
        run_cli_mode();
        return Ok(());
    }

    let ui = AppWindow::new()?;

    // Shared simulation engine (created on start)
    let sim: Rc<RefCell<Option<SimulationEngine>>> = Rc::new(RefCell::new(None));
    let timer = Rc::new(Timer::default());

    // ── Start Simulation Callback ────────────────────────
    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    let timer_clone = timer.clone();
    ui.on_start_simulation(move |procs, mem, algo, quantum| {
        let config = SimConfig {
            initial_processes: procs as u32,
            memory_capacity: mem as u32,
            algorithm: Algorithm::from_index(algo),
            quantum: quantum as u32,
        };

        let engine = SimulationEngine::new(config);

        if let Some(ui) = ui_weak.upgrade() {
            let state = ui.global::<SimState>();
            state.set_current_screen(1);
            state.set_quantum_max(quantum);
            state.set_algorithm_label(SharedString::from(
                Algorithm::from_index(algo).label(),
            ));
            state.set_speed(1);

            update_ui(&ui, &engine);
        }

        *sim_clone.borrow_mut() = Some(engine);

        // Start the timer
        let sim_inner = sim_clone.clone();
        let ui_inner = ui_weak.clone();
        timer_clone.start(
            TimerMode::Repeated,
            Duration::from_millis(1000),
            move || {
                let mut sim_ref = sim_inner.borrow_mut();
                if let Some(ref mut engine) = *sim_ref {
                    if engine.state == EngineState::Running {
                        engine.tick();
                    }
                    if let Some(ui) = ui_inner.upgrade() {
                        update_ui(&ui, engine);
                    }
                }
            },
        );
    });

    // ── Pause/Resume/Step/Reset ──────────────────────────
    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    ui.on_pause_simulation(move || {
        let mut sim_ref = sim_clone.borrow_mut();
        if let Some(ref mut engine) = *sim_ref {
            engine.pause();
            if let Some(ui) = ui_weak.upgrade() {
                update_ui(&ui, engine);
            }
        }
    });

    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    ui.on_resume_simulation(move || {
        let mut sim_ref = sim_clone.borrow_mut();
        if let Some(ref mut engine) = *sim_ref {
            engine.resume();
            if let Some(ui) = ui_weak.upgrade() {
                update_ui(&ui, engine);
            }
        }
    });

    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    ui.on_step_simulation(move || {
        let mut sim_ref = sim_clone.borrow_mut();
        if let Some(ref mut engine) = *sim_ref {
            engine.step();
            if let Some(ui) = ui_weak.upgrade() {
                update_ui(&ui, engine);
            }
        }
    });

    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    ui.on_reset_simulation(move || {
        let mut sim_ref = sim_clone.borrow_mut();
        if let Some(ref mut engine) = *sim_ref {
            engine.reset();
            if let Some(ui) = ui_weak.upgrade() {
                let state = ui.global::<SimState>();
                state.set_current_screen(1);
                state.set_table_page(0);
                update_ui(&ui, engine);
            }
        }
    });

    // ── Speed Change ─────────────────────────────────────
    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    let timer_clone = timer.clone();
    ui.on_change_speed(move |speed| {
        let mut sim_ref = sim_clone.borrow_mut();
        if let Some(ref mut engine) = *sim_ref {
            engine.set_speed(speed as u32);
            if let Some(ui) = ui_weak.upgrade() {
                ui.global::<SimState>().set_speed(speed);
            }

            // Restart timer with new interval
            let interval = engine.timer_interval_ms();
            let sim_inner = sim_clone.clone();
            let ui_inner = ui_weak.clone();
            timer_clone.start(
                TimerMode::Repeated,
                Duration::from_millis(interval),
                move || {
                    let mut sim_r = sim_inner.borrow_mut();
                    if let Some(ref mut e) = *sim_r {
                        if e.state == EngineState::Running {
                            e.tick();
                        }
                        if let Some(ui) = ui_inner.upgrade() {
                            update_ui(&ui, e);
                        }
                    }
                },
            );
        }
    });

    // ── Edit / Delete Process ────────────────────────────
    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    ui.on_request_edit(move |pid| {
        let sim_ref = sim_clone.borrow();
        if let Some(ref engine) = *sim_ref {
            // Find process data
            for proc in engine.all_processes_for_table() {
                if proc.pid == pid as u32 {
                    if let Some(ui) = ui_weak.upgrade() {
                        let state = ui.global::<SimState>();
                        state.set_edit_pid(pid);
                        state.set_edit_pid_hex(SharedString::from(proc.pid_hex()));
                        state.set_edit_name(SharedString::from(proc.name.as_str()));
                        state.set_edit_burst(proc.burst_time as i32);
                        state.set_edit_priority(proc.priority as i32);
                        state.set_show_edit_modal(true);
                    }
                    break;
                }
            }
        }
    });

    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    ui.on_edit_process(move |pid, name, burst, priority| {
        let mut sim_ref = sim_clone.borrow_mut();
        if let Some(ref mut engine) = *sim_ref {
            engine.edit_process(
                pid as u32,
                name.to_string(),
                burst as u32,
                priority as u8,
            );
            if let Some(ui) = ui_weak.upgrade() {
                update_ui(&ui, engine);
            }
        }
    });

    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    ui.on_create_process(move |name, burst, priority, memory| {
        let mut sim_ref = sim_clone.borrow_mut();
        if let Some(ref mut engine) = *sim_ref {
            engine.add_custom_process(
                name.to_string(),
                burst as u32,
                priority as u8,
                memory,
            );
            if let Some(ui) = ui_weak.upgrade() {
                update_ui(&ui, engine);
            }
        }
    });

    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    ui.on_delete_process(move |pid| {
        let mut sim_ref = sim_clone.borrow_mut();
        if let Some(ref mut engine) = *sim_ref {
            engine.remove_process(pid as u32);
            if let Some(ui) = ui_weak.upgrade() {
                update_ui(&ui, engine);
            }
        }
    });

    // ── Toggle Select (for batch actions) ────────────────
    ui.on_toggle_select(move |_pid| {
        // Selection state is managed in the UI model for now
    });

    // ── Return to Init ───────────────────────────────────
    let sim_clone = sim.clone();
    let ui_weak = ui.as_weak();
    ui.on_return_to_init(move || {
        let mut sim_ref = sim_clone.borrow_mut();
        *sim_ref = None;
        if let Some(ui) = ui_weak.upgrade() {
            ui.global::<SimState>().set_current_screen(0);
        }
    });

    ui.run()
}

/// Runs the CLI-only mode of the simulator.
fn run_cli_mode() {
    use simulador_procesos::ui::cli;

    let config = SimConfig {
        initial_processes: 5,
        memory_capacity: 4096,
        algorithm: Algorithm::FCFS,
        quantum: 4,
    };

    let mut engine = SimulationEngine::new(config);
    cli::run_menu(&mut engine);
}
