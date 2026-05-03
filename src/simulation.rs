/// Simulation engine — orchestrates the process simulator.
///
/// Manages the lifecycle: initialization, tick execution, I/O events,
/// speed control, and state conversion for the UI layer.

use rand::Rng;

use crate::metrics::{self, SimulationMetrics};
use crate::constants::{IO_PROBABILITY, MIN_IO_BURST, MAX_IO_BURST, SPEED_1X_MS, SPEED_2X_MS, SPEED_5X_MS, SPEED_10X_MS};
use crate::process::PCB;
use crate::scheduler::{Algorithm, GanttEntry, LogEntry, Scheduler};

// ─── Simulation State ────────────────────────────────────────────────────────

/// High-level state of the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimState {
    /// Waiting for configuration (init modal).
    Idle,
    /// Simulation is running tick-by-tick.
    Running,
    /// Simulation is paused.
    Paused,
    /// All user processes have terminated.
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
    /// The process scheduler.
    pub scheduler: Scheduler,
    /// Engine configuration.
    pub config: SimConfig,
    /// Current simulation state.
    pub state: SimState,
    /// Random number generator.
    rng: rand::rngs::ThreadRng,
    /// All processes generated (for arrival tracking).
    all_processes: Vec<PCB>,
    /// Next PID to assign.
    next_pid: u32,
    /// Speed multiplier (1x, 2x, 5x, 10x).
    pub speed: u32,
    /// Previous simulation metrics (for delta calculation).
    pub previous_metrics: Option<SimulationMetrics>,
    /// Current computed metrics.
    pub current_metrics: SimulationMetrics,
}

impl SimulationEngine {
    /// Creates and initializes a new simulation with the given config.
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

    /// Generates initial processes and sets up the simulation.
    fn initialize(&mut self) {
        // Add kernel daemon (always present)
        let daemon = PCB::new_kernel_daemon();
        self.scheduler.add_process(daemon);

        // Generate N-1 random user processes with staggered arrivals
        let count = self.config.initial_processes.saturating_sub(1);
        for i in 0..count {
            let pid = self.next_pid;
            self.next_pid += 1;

            // Stagger arrival times: 0, 0-2, 1-4, 2-6, etc.
            let base_arrival = (i as u32) / 2;
            let jitter = self.rng.gen_range(0..=(i.min(4)));
            let arrival = base_arrival + jitter;

            let pcb = PCB::new_random(pid, arrival, &mut self.rng);
            self.all_processes.push(pcb);
        }

        // Sort by arrival time
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

    /// Advances the simulation by one tick.
    ///
    /// Returns `true` if the simulation is still running, `false` if completed.
    pub fn tick(&mut self) -> bool {
        if self.state != SimState::Running {
            return self.state != SimState::Completed;
        }

        // Check for new arrivals
        self.check_arrivals();

        // Random I/O event for current process
        self.maybe_trigger_io();

        // Advance scheduler by one tick
        self.scheduler.tick();

        // Check if simulation is complete
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

    /// Checks if processes should arrive at the current clock tick.
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

    /// Whether all pending processes have arrived.
    fn all_pending_arrived(&self) -> bool {
        self.all_processes.is_empty()
    }

    /// Dynamically add a custom process created by the user.
    pub fn add_custom_process(&mut self, name: String, burst: u32, priority: u8, memory: f32) {
        let pid = self.next_pid;
        self.next_pid += 1;
        
        let pcb = PCB {
            pid,
            name,
            state: crate::process::ProcessState::New,
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

    /// Random I/O event: 15% chance per tick for the running process.
    fn maybe_trigger_io(&mut self) {
        if let Some(ref proc) = self.scheduler.current_process {
            // Don't interrupt kernel daemon
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

    /// Computes final simulation metrics.
    fn compute_final_metrics(&mut self) {
        // Filter out kernel daemon from metrics
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

    /// Pauses the simulation.
    pub fn pause(&mut self) {
        if self.state == SimState::Running {
            self.state = SimState::Paused;
            self.scheduler.log_event("Simulación pausada.".to_string());
        }
    }

    /// Resumes the simulation.
    pub fn resume(&mut self) {
        if self.state == SimState::Paused {
            self.state = SimState::Running;
            self.scheduler.log_event("Simulación reanudada.".to_string());
        }
    }

    /// Executes a single step (for step-by-step mode).
    pub fn step(&mut self) {
        if self.state == SimState::Paused || self.state == SimState::Running {
            self.state = SimState::Running;
            self.tick();
            if self.state != SimState::Completed {
                self.state = SimState::Paused;
            }
        }
    }

    /// Resets the simulation with the same config.
    pub fn reset(&mut self) {
        self.previous_metrics = Some(self.current_metrics.clone());
        let config = self.config.clone();
        self.scheduler = Scheduler::new(config.algorithm, config.quantum);
        self.all_processes.clear();
        self.next_pid = 0x00A2;
        self.current_metrics = SimulationMetrics::zero();
        self.initialize();
    }

    /// Sets the simulation speed multiplier.
    pub fn set_speed(&mut self, speed: u32) {
        self.speed = speed.clamp(1, 10);
    }

    /// Returns the timer interval in milliseconds based on current speed.
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

    /// Returns the current clock value.
    pub fn clock(&self) -> u32 {
        self.scheduler.clock
    }

    /// Returns CPU load as a percentage.
    pub fn cpu_load(&self) -> f32 {
        if self.scheduler.clock == 0 {
            return 0.0;
        }
        let busy = self.scheduler.clock.saturating_sub(self.scheduler.idle_ticks);
        (busy as f32 / self.scheduler.clock as f32) * 100.0
    }

    /// Returns total memory used by active processes.
    pub fn memory_used(&self) -> f32 {
        self.scheduler.total_memory_used()
    }

    /// Returns total active process count.
    pub fn active_count(&self) -> usize {
        self.scheduler.active_process_count()
    }

    /// Returns reference to ready queue.
    pub fn ready_queue(&self) -> &std::collections::VecDeque<PCB> {
        &self.scheduler.ready_queue
    }

    /// Returns reference to blocked queue.
    pub fn blocked_queue(&self) -> &std::collections::VecDeque<PCB> {
        &self.scheduler.blocked_queue
    }

    /// Returns reference to terminated list.
    pub fn terminated(&self) -> &[PCB] {
        &self.scheduler.terminated
    }

    /// Returns reference to current running process.
    pub fn current_process(&self) -> Option<&PCB> {
        self.scheduler.current_process.as_ref()
    }

    /// Returns reference to system log.
    pub fn sys_log(&self) -> &[LogEntry] {
        &self.scheduler.sys_log
    }

    /// Returns reference to Gantt log.
    pub fn gantt_log(&self) -> &[GanttEntry] {
        &self.scheduler.gantt_log
    }

    /// Returns the quantum remaining for the current process.
    pub fn quantum_remaining(&self) -> u32 {
        self.scheduler.quantum_remaining
    }

    /// Returns all processes (ready + blocked + running + terminated) for the table view.
    pub fn all_processes_for_table(&self) -> Vec<PCB> {
        let mut all = Vec::new();

        if let Some(ref p) = self.scheduler.current_process {
            all.push(p.clone());
        }
        for p in &self.scheduler.ready_queue {
            all.push(p.clone());
        }
        for p in &self.scheduler.blocked_queue {
            all.push(p.clone());
        }
        for p in &self.scheduler.terminated {
            all.push(p.clone());
        }

        all.sort_by_key(|p| p.pid);
        all
    }

    /// Edit a process by PID.
    pub fn edit_process(&mut self, pid: u32, name: String, burst: u32, priority: u8) {
        self.scheduler.edit_process(pid, name, burst, priority);
    }

    /// Remove a process by PID.
    pub fn remove_process(&mut self, pid: u32) {
        self.scheduler.remove_process(pid);
    }
}
