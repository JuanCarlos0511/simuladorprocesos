/// PCB (Process Control Block) and related types for the OS process simulator.
///
/// This module defines the core data structures for process representation,
/// including states, the PCB struct, and a pool of descriptive process names
/// for random generation.

use rand::Rng;

// ─── Constants ───────────────────────────────────────────────────────────────

/// PID of the system kernel daemon (always present).
pub const SYS_KERNEL_PID: u32 = 0x00A1;

/// Minimum CPU burst time for random generation (ms).
pub const MIN_BURST: u32 = 5;
/// Maximum CPU burst time for random generation (ms).
pub const MAX_BURST: u32 = 50;

/// Minimum priority value (highest priority).
pub const MIN_PRIORITY: u8 = 1;
/// Maximum priority value (lowest priority).
pub const MAX_PRIORITY: u8 = 10;

/// Minimum memory allocation for random generation (MB).
pub const MIN_MEMORY: f32 = 16.0;
/// Maximum memory allocation for random generation (MB).
pub const MAX_MEMORY: f32 = 512.0;

/// Probability that a running process requests I/O each tick.
pub const IO_PROBABILITY: f64 = 0.15;

/// Minimum I/O burst duration (ms).
pub const MIN_IO_BURST: u32 = 5;
/// Maximum I/O burst duration (ms).
pub const MAX_IO_BURST: u32 = 20;

// ─── Process State ───────────────────────────────────────────────────────────

/// Represents the lifecycle state of a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    New = 0,
    Ready = 1,
    Running = 2,
    Blocked = 3,
    Terminated = 4,
}

// ─── Process Control Block ───────────────────────────────────────────────────

/// Process Control Block — core data structure for each process.
#[derive(Debug, Clone)]
pub struct PCB {
    /// Unique process identifier.
    pub pid: u32,
    /// Descriptive process name.
    pub name: String,
    /// Current process state.
    pub state: ProcessState,
    /// Total CPU burst time required (ms).
    pub burst_time: u32,
    /// Remaining CPU time (ms).
    pub remaining_time: u32,
    /// Time when the process arrived in the system.
    pub arrival_time: u32,
    /// Priority level (1 = highest, 10 = lowest).
    pub priority: u8,
    /// Memory allocated (MB).
    pub memory_mb: f32,
    /// Remaining I/O burst time, if process is blocked.
    pub io_burst: Option<u32>,
    /// Time when the process finished execution.
    pub finish_time: Option<u32>,
    /// Turnaround time (finish_time - arrival_time).
    pub turnaround_time: Option<u32>,
    /// Waiting time (turnaround_time - burst_time).
    pub waiting_time: Option<u32>,
}

impl PCB {
    /// Creates the system kernel daemon process.
    ///
    /// This process always exists with PID 0x00A1, maximum priority,
    /// and effectively infinite burst time.
    pub fn new_kernel_daemon() -> Self {
        PCB {
            pid: SYS_KERNEL_PID,
            name: "sys_kernel_daemon".to_string(),
            state: ProcessState::Ready,
            burst_time: u32::MAX,
            remaining_time: u32::MAX,
            arrival_time: 0,
            priority: 0,
            memory_mb: 124.5,
            io_burst: None,
            finish_time: None,
            turnaround_time: None,
            waiting_time: None,
        }
    }

    /// Creates a random process with the given PID and arrival time.
    pub fn new_random(pid: u32, arrival_time: u32, rng: &mut impl Rng) -> Self {
        let burst = rng.gen_range(MIN_BURST..=MAX_BURST);
        let priority = rng.gen_range(MIN_PRIORITY..=MAX_PRIORITY);
        let memory = (rng.gen_range(MIN_MEMORY..=MAX_MEMORY) * 10.0).round() / 10.0;
        let name = PROCESS_NAMES[rng.gen_range(0..PROCESS_NAMES.len())].to_string();

        PCB {
            pid,
            name,
            state: ProcessState::New,
            burst_time: burst,
            remaining_time: burst,
            arrival_time,
            priority,
            memory_mb: memory,
            io_burst: None,
            finish_time: None,
            turnaround_time: None,
            waiting_time: None,
        }
    }

    /// Returns the PID formatted as a hex string (e.g., "0x00A1").
    pub fn pid_hex(&self) -> String {
        format!("0x{:04X}", self.pid)
    }

    /// Whether this process is the kernel daemon.
    pub fn is_kernel_daemon(&self) -> bool {
        self.pid == SYS_KERNEL_PID
    }

    /// Returns a priority label based on the numeric value.
    pub fn priority_label(&self) -> &'static str {
        match self.priority {
            0..=3 => "Alta",
            4..=6 => "Normal",
            _ => "Baja",
        }
    }

    /// Marks the process as terminated and calculates final metrics.
    pub fn terminate(&mut self, clock: u32) {
        self.state = ProcessState::Terminated;
        self.remaining_time = 0;
        self.finish_time = Some(clock);
        self.turnaround_time = Some(clock.saturating_sub(self.arrival_time));
        self.waiting_time = Some(
            clock
                .saturating_sub(self.arrival_time)
                .saturating_sub(self.burst_time),
        );
    }
}

// ─── Process Name Pool ───────────────────────────────────────────────────────

/// Pool of descriptive process names for random generation.
const PROCESS_NAMES: &[&str] = &[
    "nginx_worker",
    "db_query_analyzer",
    "node_auth_service",
    "temp_file_cleanup",
    "log_rotate_daemon",
    "cache_invalidator",
    "ssl_handshake_mgr",
    "packet_inspector",
    "mem_page_allocator",
    "task_scheduler_svc",
    "io_buffer_manager",
    "dns_resolver_worker",
    "file_index_builder",
    "session_gc_sweep",
    "api_gateway_proxy",
    "data_ingestion_svc",
    "metric_collector",
    "event_stream_proc",
    "backup_snapshot_mgr",
    "config_hot_reload",
    "health_check_probe",
    "rate_limiter_svc",
    "queue_consumer_wrk",
    "image_resize_worker",
    "pdf_renderer_svc",
    "email_dispatch_svc",
    "webhook_relay_proc",
    "cron_job_executor",
    "audit_log_writer",
    "compression_engine",
];

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_daemon_has_correct_defaults() {
        let daemon = PCB::new_kernel_daemon();
        assert_eq!(daemon.pid, SYS_KERNEL_PID);
        assert_eq!(daemon.priority, 0);
        assert_eq!(daemon.burst_time, u32::MAX);
        assert!(daemon.is_kernel_daemon());
    }

    #[test]
    fn random_process_has_valid_ranges() {
        let mut rng = rand::thread_rng();
        for i in 0..100 {
            let pcb = PCB::new_random(0x00A2 + i, i, &mut rng);
            assert!(pcb.burst_time >= MIN_BURST && pcb.burst_time <= MAX_BURST);
            assert!(pcb.priority >= MIN_PRIORITY && pcb.priority <= MAX_PRIORITY);
            assert!(pcb.memory_mb >= MIN_MEMORY && pcb.memory_mb <= MAX_MEMORY);
            assert_eq!(pcb.state, ProcessState::New);
        }
    }

    #[test]
    fn pid_hex_format() {
        let daemon = PCB::new_kernel_daemon();
        assert_eq!(daemon.pid_hex(), "0x00A1");
    }

    #[test]
    fn terminate_calculates_metrics() {
        let mut pcb = PCB::new_kernel_daemon();
        pcb.burst_time = 10;
        pcb.remaining_time = 0;
        pcb.arrival_time = 5;
        pcb.terminate(20);
        assert_eq!(pcb.finish_time, Some(20));
        assert_eq!(pcb.turnaround_time, Some(15)); // 20 - 5
        assert_eq!(pcb.waiting_time, Some(5));      // 15 - 10
    }
}
