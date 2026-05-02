/// Scheduler module — trait-based dispatching for scheduling algorithms.
///
/// The `Scheduler` struct is algorithm-agnostic. Each scheduling policy
/// implements the `SchedulingAlgorithm` trait in its own file.

pub mod fcfs;
pub mod priority;
pub mod priority_preemptive;
pub mod round_robin;
pub mod sjf;
pub mod srtf;

use std::collections::VecDeque;

use crate::process::{PCB, ProcessState};

// ─── Algorithm Enum ──────────────────────────────────────────────────────────

/// Available scheduling algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    FCFS,
    SJF,
    SRTF,
    RoundRobin,
    Priority,
    PriorityPreemptive,
}

impl Algorithm {
    /// Creates an Algorithm from an integer index (used by UI dropdown).
    pub fn from_index(index: i32) -> Self {
        match index {
            0 => Algorithm::FCFS,
            1 => Algorithm::SJF,
            2 => Algorithm::SRTF,
            3 => Algorithm::RoundRobin,
            4 => Algorithm::Priority,
            5 => Algorithm::PriorityPreemptive,
            _ => Algorithm::FCFS,
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Algorithm::FCFS => "FCFS",
            Algorithm::SJF => "SJF",
            Algorithm::SRTF => "SRTF",
            Algorithm::RoundRobin => "Round Robin",
            Algorithm::Priority => "Prioridad",
            Algorithm::PriorityPreemptive => "Prioridad Preemptivo",
        }
    }
}

// ─── Scheduling Algorithm Trait ──────────────────────────────────────────────

/// Trait that all scheduling algorithms must implement.
pub trait SchedulingAlgorithm {
    /// Selects the index of the next process to run from the ready queue.
    /// Returns `None` if the queue is empty.
    fn select_next(&self, ready_queue: &VecDeque<PCB>) -> Option<usize>;

    /// Whether the current process should be preempted.
    /// Default: no preemption (non-preemptive algorithms).
    fn should_preempt(&self, _current: &PCB, _ready_queue: &VecDeque<PCB>) -> bool {
        false
    }

    /// Whether this algorithm uses a quantum (only Round Robin).
    fn uses_quantum(&self) -> bool {
        false
    }

    /// Display name for logging.
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}

// ─── Gantt & Log Entries ─────────────────────────────────────────────────────

/// A single entry in the Gantt chart timeline.
#[derive(Debug, Clone)]
pub struct GanttEntry {
    pub pid: u32,
    pub start: u32,
    pub end: u32,
}

/// A single entry in the system event log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u32,
    pub message: String,
}

// ─── Scheduler ───────────────────────────────────────────────────────────────

/// The main scheduler that manages process queues and dispatching.
pub struct Scheduler {
    /// The scheduling algorithm implementation.
    algorithm: Box<dyn SchedulingAlgorithm>,
    /// Algorithm enum value (for display purposes).
    #[allow(dead_code)]
    pub algorithm_type: Algorithm,
    /// Quantum size in ticks (only used for Round Robin).
    pub quantum: u32,
    /// Remaining quantum for the current process.
    pub quantum_remaining: u32,
    /// Queue of processes ready to execute.
    pub ready_queue: VecDeque<PCB>,
    /// Queue of processes waiting for I/O.
    pub blocked_queue: VecDeque<PCB>,
    /// List of terminated processes.
    pub terminated: Vec<PCB>,
    /// Process currently executing on the CPU.
    pub current_process: Option<PCB>,
    /// Global system clock (tick counter).
    pub clock: u32,
    /// Gantt chart timeline entries.
    pub gantt_log: Vec<GanttEntry>,
    /// System event log.
    pub sys_log: Vec<LogEntry>,
    /// Start tick of the current Gantt segment.
    gantt_segment_start: Option<u32>,
    /// Number of idle ticks (no user process running).
    pub idle_ticks: u32,
}

impl Scheduler {
    /// Creates a new Scheduler with the given algorithm and quantum.
    pub fn new(algo: Algorithm, quantum: u32) -> Self {
        let algorithm: Box<dyn SchedulingAlgorithm> = match algo {
            Algorithm::FCFS => Box::new(fcfs::Fcfs),
            Algorithm::SJF => Box::new(sjf::Sjf),
            Algorithm::SRTF => Box::new(srtf::Srtf),
            Algorithm::RoundRobin => Box::new(round_robin::RoundRobin),
            Algorithm::Priority => Box::new(priority::PriorityNonPreemptive),
            Algorithm::PriorityPreemptive => {
                Box::new(priority_preemptive::PriorityPreemptive)
            }
        };

        Scheduler {
            algorithm,
            algorithm_type: algo,
            quantum,
            quantum_remaining: quantum,
            ready_queue: VecDeque::new(),
            blocked_queue: VecDeque::new(),
            terminated: Vec::new(),
            current_process: None,
            clock: 0,
            gantt_log: Vec::new(),
            sys_log: Vec::new(),
            gantt_segment_start: None,
            idle_ticks: 0,
        }
    }

    /// Adds a process to the ready queue.
    pub fn add_process(&mut self, mut pcb: PCB) {
        pcb.state = ProcessState::Ready;
        self.log_event(format!(
            "Proceso {} ({}) ingresó a Cola Listos. BT: {}ms",
            pcb.pid_hex(),
            pcb.name,
            pcb.burst_time
        ));
        self.ready_queue.push_back(pcb);
    }

    /// Logs a system event with the current timestamp.
    pub fn log_event(&mut self, message: String) {
        self.sys_log.push(LogEntry {
            timestamp: self.clock,
            message,
        });
    }

    /// Advances the simulation by one tick.
    ///
    /// This is the core simulation step:
    /// 1. Handle I/O completions in blocked queue
    /// 2. Execute current process (decrement remaining_time)
    /// 3. Check for termination
    /// 4. Check for preemption / quantum expiry
    /// 5. If CPU is free, dispatch next process
    pub fn tick(&mut self) {
        self.clock += 1;

        // Step 1: Handle I/O completions
        self.handle_io_completion();

        // Step 2-4: Handle current process
        if let Some(ref mut proc) = self.current_process {
            // Skip kernel daemon for actual execution decrement
            if !proc.is_kernel_daemon() {
                proc.remaining_time = proc.remaining_time.saturating_sub(1);
            }

            // Check if process finished
            if proc.remaining_time == 0 && !proc.is_kernel_daemon() {
                self.finish_current_gantt_segment();
                let mut finished = self.current_process.take().unwrap();
                finished.terminate(self.clock);
                self.log_event(format!(
                    "Proceso {} finalizado. Estado: Terminado. Memoria liberada: {}MB",
                    finished.pid_hex(),
                    finished.memory_mb
                ));
                self.terminated.push(finished);
            } else {
                // Check quantum expiry for Round Robin
                if self.algorithm.uses_quantum() {
                    self.quantum_remaining = self.quantum_remaining.saturating_sub(1);
                    if self.quantum_remaining == 0 {
                        self.preempt_current("Quantum expirado");
                    }
                }

                // Check preemption for preemptive algorithms
                if self.current_process.is_some()
                    && self
                        .algorithm
                        .should_preempt(self.current_process.as_ref().unwrap(), &self.ready_queue)
                {
                    self.preempt_current("Preemption por algoritmo");
                }
            }
        } else {
            self.idle_ticks += 1;
        }

        // Step 5: If CPU is free, dispatch next
        if self.current_process.is_none() {
            self.dispatch();
        }
    }

    /// Handles I/O completion: decrements io_burst for blocked processes,
    /// moves them to ready queue when I/O is done.
    fn handle_io_completion(&mut self) {
        let mut completed_indices = Vec::new();

        for (i, proc) in self.blocked_queue.iter_mut().enumerate() {
            if let Some(ref mut io) = proc.io_burst {
                *io = io.saturating_sub(1);
                if *io == 0 {
                    proc.io_burst = None;
                    completed_indices.push(i);
                }
            }
        }

        // Move completed I/O processes to ready queue (reverse order to preserve indices)
        for i in completed_indices.into_iter().rev() {
            let mut proc = self.blocked_queue.remove(i).unwrap();
            proc.state = ProcessState::Ready;
            self.log_event(format!(
                "I/O completado para {}. Movido a Cola Listos.",
                proc.pid_hex()
            ));
            self.ready_queue.push_back(proc);
        }
    }

    /// Moves the current process to the blocked queue for I/O.
    pub fn block_current_for_io(&mut self, io_duration: u32) {
        if let Some(mut proc) = self.current_process.take() {
            self.finish_current_gantt_segment();
            proc.state = ProcessState::Blocked;
            proc.io_burst = Some(io_duration);
            self.log_event(format!(
                "Interrupción: I/O request de {}. Duración: {}ms",
                proc.pid_hex(),
                io_duration
            ));
            self.blocked_queue.push_back(proc);
        }
    }

    /// Preempts the current process, moving it back to the ready queue.
    fn preempt_current(&mut self, reason: &str) {
        if let Some(mut proc) = self.current_process.take() {
            self.finish_current_gantt_segment();
            let pid_hex = proc.pid_hex();
            proc.state = ProcessState::Ready;
            self.ready_queue.push_back(proc);
            self.log_event(format!(
                "Context switch: {} preemptado. Razón: {}",
                pid_hex, reason
            ));
        }
    }

    /// Dispatches the next process from the ready queue to the CPU.
    fn dispatch(&mut self) {
        if let Some(idx) = self.algorithm.select_next(&self.ready_queue) {
            let mut proc = self.ready_queue.remove(idx).unwrap();
            let prev_pid = self
                .gantt_log
                .last()
                .map(|e| format!("0x{:04X}", e.pid))
                .unwrap_or_else(|| "ninguno".to_string());
            proc.state = ProcessState::Running;
            self.log_event(format!(
                "Dispatcher: Context switch {} -> {}. BT restante: {}ms",
                prev_pid,
                proc.pid_hex(),
                proc.remaining_time
            ));
            self.gantt_segment_start = Some(self.clock);
            self.quantum_remaining = self.quantum;
            self.current_process = Some(proc);
        }
    }

    /// Closes the current Gantt segment for the running process.
    fn finish_current_gantt_segment(&mut self) {
        if let (Some(start), Some(ref proc)) = (self.gantt_segment_start, &self.current_process) {
            if !proc.is_kernel_daemon() {
                self.gantt_log.push(GanttEntry {
                    pid: proc.pid,
                    start,
                    end: self.clock,
                });
            }
            self.gantt_segment_start = None;
        }
    }

    /// Returns true if all user processes (non-kernel) have terminated.
    pub fn is_simulation_complete(&self) -> bool {
        let has_user_ready = self
            .ready_queue
            .iter()
            .any(|p| !p.is_kernel_daemon());
        let has_user_blocked = self
            .blocked_queue
            .iter()
            .any(|p| !p.is_kernel_daemon());
        let has_user_running = self
            .current_process
            .as_ref()
            .map_or(false, |p| !p.is_kernel_daemon());

        !has_user_ready && !has_user_blocked && !has_user_running && !self.terminated.is_empty()
    }

    /// Edits a process in the ready or blocked queue.
    pub fn edit_process(&mut self, pid: u32, name: String, burst: u32, priority: u8) {
        let mut edited_pid_hex = None;
        for proc in self.ready_queue.iter_mut().chain(self.blocked_queue.iter_mut()) {
            if proc.pid == pid {
                proc.name = name.clone();
                let old_burst = proc.burst_time;
                // Adjust remaining time proportionally
                if old_burst > 0 {
                    let ratio = proc.remaining_time as f64 / old_burst as f64;
                    proc.burst_time = burst;
                    proc.remaining_time = (burst as f64 * ratio).round() as u32;
                } else {
                    proc.burst_time = burst;
                    proc.remaining_time = burst;
                }
                proc.priority = priority;
                edited_pid_hex = Some(proc.pid_hex());
                break;
            }
        }
        if let Some(pid_hex) = edited_pid_hex {
            self.log_event(format!("Proceso {} editado.", pid_hex));
        }
    }

    /// Removes a process from the ready or blocked queue.
    pub fn remove_process(&mut self, pid: u32) {
        if let Some(pos) = self.ready_queue.iter().position(|p| p.pid == pid) {
            let proc = self.ready_queue.remove(pos).unwrap();
            self.log_event(format!(
                "Proceso {} eliminado. Memoria liberada: {}MB",
                proc.pid_hex(),
                proc.memory_mb
            ));
            return;
        }
        if let Some(pos) = self.blocked_queue.iter().position(|p| p.pid == pid) {
            let proc = self.blocked_queue.remove(pos).unwrap();
            self.log_event(format!(
                "Proceso {} eliminado. Memoria liberada: {}MB",
                proc.pid_hex(),
                proc.memory_mb
            ));
        }
    }

    /// Returns total memory used by all active processes (MB).
    pub fn total_memory_used(&self) -> f32 {
        let mut total: f32 = 0.0;
        for p in &self.ready_queue {
            total += p.memory_mb;
        }
        for p in &self.blocked_queue {
            total += p.memory_mb;
        }
        if let Some(ref p) = self.current_process {
            total += p.memory_mb;
        }
        total
    }

    /// Returns the total count of active (non-terminated) processes.
    pub fn active_process_count(&self) -> usize {
        self.ready_queue.len()
            + self.blocked_queue.len()
            + if self.current_process.is_some() { 1 } else { 0 }
    }
}
