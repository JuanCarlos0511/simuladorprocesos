/// Simulation engine — orchestrates the process simulator.
///
/// Manages the lifecycle: initialization, tick execution, I/O events,
/// speed control, and state conversion for the UI layer.

use rand::Rng;

use crate::core::metrics::{self, SimulationMetrics};
use crate::constants::{IO_PROBABILITY, MIN_IO_BURST, MAX_IO_BURST, SPEED_1X_MS, SPEED_2X_MS, SPEED_5X_MS, SPEED_10X_MS};
use crate::core::process::{PCB, ProcessState};
use crate::core::scheduler::{Algorithm, GanttEntry, LogEntry, Scheduler};

// ─── Simulation State ────────────────────────────────────────────────────────

/// High-level state of the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimState {
    Idle,
    Running,
    Paused,
    Completed,
}

// ─── Configuration ───────────────────────────────────────────────────────────

/// Configuration from the initialization modal.
#[derive(Debug, Clone)]
pub struct SimConfig {
    pub initial_processes: u32,
    pub memory_capacity: u32,
    pub algorithm: Algorithm,
    pub quantum: u32,
}

// ─── Simulation Engine ───────────────────────────────────────────────────────

/// The main simulation engine that wraps the Scheduler and manages the
/// full simulation lifecycle.
pub struct SimulationEngine {
    pub scheduler: Scheduler,
    pub config: SimConfig,
    pub state: SimState,
    rng: rand::rngs::ThreadRng,
    all_processes: Vec<PCB>,
    next_pid: u32,
    pub speed: u32,
    pub previous_metrics: Option<SimulationMetrics>,
    pub current_metrics: SimulationMetrics,
}

impl SimulationEngine {
    pub fn new(config: SimConfig) -> Self {
        let mut engine = SimulationEngine {
            scheduler: Scheduler::new(config.algorithm, config.quantum),
            config: config.clone(),
            state: SimState::Idle,
            rng: rand::thread_rng(),
            all_processes: Vec::new(),
            next_pid: 0x00A2,
            speed: 1,
            previous_metrics: None,
            current_metrics: SimulationMetrics::zero(),
        };
        engine.initialize();
        engine
    }

    fn initialize(&mut self) {
        let daemon = PCB::new_kernel_daemon();
        self.scheduler.add_process(daemon);

        let count = self.config.initial_processes.saturating_sub(1);
        for i in 0..count {
            let pid = self.next_pid;
            self.next_pid += 1;
            let base_arrival = (i as u32) / 2;
            let jitter = self.rng.gen_range(0..=(i.min(4)));
            let arrival = base_arrival + jitter;
            let pcb = PCB::new_random(pid, arrival, &mut self.rng);
            self.all_processes.push(pcb);
        }

        self.all_processes.sort_by_key(|p| p.arrival_time);
        self.state = SimState::Running;
        self.scheduler.log_event("KERNEL_OS inicializado. Simulación comenzada.".to_string());
        self.scheduler.log_event(format!(
            "Algoritmo: {}. Procesos: {}. Memoria: {}MB",
            self.config.algorithm.label(),
            self.config.initial_processes,
            self.config.memory_capacity
        ));
    }

    pub fn tick(&mut self) -> bool {
        if self.state != SimState::Running {
            return self.state != SimState::Completed;
        }
        self.check_arrivals();
        self.maybe_trigger_io();
        self.scheduler.tick();

        if self.scheduler.is_simulation_complete() && self.all_pending_arrived() {
            self.state = SimState::Completed;
            self.scheduler.log_event(
                "Simulación completada. Todos los procesos finalizados.".to_string(),
            );
            self.compute_final_metrics();
            return false;
        }
        true
    }

    fn check_arrivals(&mut self) {
        let clock = self.scheduler.clock;
        let mut arrived = Vec::new();
        self.all_processes.retain(|p| {
            if p.arrival_time <= clock {
                arrived.push(p.clone());
                false
            } else {
                true
            }
        });
        for pcb in arrived {
            self.scheduler.add_process(pcb);
        }
    }

    fn all_pending_arrived(&self) -> bool {
        self.all_processes.is_empty()
    }

    pub fn add_custom_process(&mut self, name: String, burst: u32, priority: u8, memory: f32) {
        let pid = self.next_pid;
        self.next_pid += 1;
        let pcb = PCB {
            pid,
            name,
            state: ProcessState::New,
            burst_time: burst,
            remaining_time: burst,
            arrival_time: self.clock(),
            priority,
            memory_mb: memory,
            io_burst: None,
            finish_time: None,
            turnaround_time: None,
            waiting_time: None,
        };
        self.scheduler.add_process(pcb);
    }

    fn maybe_trigger_io(&mut self) {
        if let Some(ref proc) = self.scheduler.current_process {
            if proc.is_kernel_daemon() {
                return;
            }
            let roll: f64 = self.rng.gen();
            if roll < IO_PROBABILITY {
                let io_duration = self.rng.gen_range(MIN_IO_BURST..=MAX_IO_BURST);
                self.scheduler.block_current_for_io(io_duration);
            }
        }
    }

    fn compute_final_metrics(&mut self) {
        let user_terminated: Vec<_> = self
            .scheduler
            .terminated
            .iter()
            .filter(|p| !p.is_kernel_daemon())
            .cloned()
            .collect();
        self.current_metrics = metrics::calculate_metrics(
            &user_terminated,
            self.scheduler.clock,
            self.scheduler.idle_ticks,
            self.previous_metrics.as_ref(),
        );
    }

    pub fn pause(&mut self) {
        if self.state == SimState::Running {
            self.state = SimState::Paused;
            self.scheduler.log_event("Simulación pausada.".to_string());
        }
    }

    pub fn resume(&mut self) {
        if self.state == SimState::Paused {
            self.state = SimState::Running;
            self.scheduler.log_event("Simulación reanudada.".to_string());
        }
    }

    pub fn step(&mut self) {
        if self.state == SimState::Paused || self.state == SimState::Running {
            self.state = SimState::Running;
            self.tick();
            if self.state != SimState::Completed {
                self.state = SimState::Paused;
            }
        }
    }

    pub fn reset(&mut self) {
        self.previous_metrics = Some(self.current_metrics.clone());
        let config = self.config.clone();
        self.scheduler = Scheduler::new(config.algorithm, config.quantum);
        self.all_processes.clear();
        self.next_pid = 0x00A2;
        self.current_metrics = SimulationMetrics::zero();
        self.initialize();
    }

    pub fn set_speed(&mut self, speed: u32) {
        self.speed = speed.clamp(1, 10);
    }

    pub fn timer_interval_ms(&self) -> u64 {
        match self.speed {
            1 => SPEED_1X_MS,
            2 => SPEED_2X_MS,
            5 => SPEED_5X_MS,
            10 => SPEED_10X_MS,
            _ => SPEED_1X_MS,
        }
    }

    // ─── Accessors for UI ────────────────────────────────────────────────

    pub fn clock(&self) -> u32 { self.scheduler.clock }

    pub fn cpu_load(&self) -> f32 {
        if self.scheduler.clock == 0 { return 0.0; }
        let busy = self.scheduler.clock.saturating_sub(self.scheduler.idle_ticks);
        (busy as f32 / self.scheduler.clock as f32) * 100.0
    }

    pub fn memory_used(&self) -> f32 { self.scheduler.total_memory_used() }
    pub fn active_count(&self) -> usize { self.scheduler.active_process_count() }

    pub fn ready_queue(&self) -> &std::collections::VecDeque<PCB> { &self.scheduler.ready_queue }
    pub fn blocked_queue(&self) -> &std::collections::VecDeque<PCB> { &self.scheduler.blocked_queue }
    pub fn terminated(&self) -> &[PCB] { &self.scheduler.terminated }
    pub fn current_process(&self) -> Option<&PCB> { self.scheduler.current_process.as_ref() }
    pub fn sys_log(&self) -> &[LogEntry] { &self.scheduler.sys_log }
    pub fn gantt_log(&self) -> &[GanttEntry] { &self.scheduler.gantt_log }
    pub fn quantum_remaining(&self) -> u32 { self.scheduler.quantum_remaining }

    pub fn all_processes_for_table(&self) -> Vec<PCB> {
        let mut all = Vec::new();
        if let Some(ref p) = self.scheduler.current_process { all.push(p.clone()); }
        for p in &self.scheduler.ready_queue { all.push(p.clone()); }
        for p in &self.scheduler.blocked_queue { all.push(p.clone()); }
        for p in &self.scheduler.terminated { all.push(p.clone()); }
        all.sort_by_key(|p| p.pid);
        all
    }

    pub fn edit_process(&mut self, pid: u32, name: String, burst: u32, priority: u8) {
        self.scheduler.edit_process(pid, name, burst, priority);
    }

    pub fn remove_process(&mut self, pid: u32) {
        self.scheduler.remove_process(pid);
    }
}
