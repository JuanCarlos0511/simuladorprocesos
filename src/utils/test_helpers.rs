#[cfg(test)]
use crate::process::{PCB, ProcessState};

/// Creates a standard PCB for unit testing algorithms.
#[cfg(test)]
pub fn make_pcb(pid: u32, burst: u32) -> PCB {
    PCB {
        pid,
        name: format!("P{}", pid),
        state: ProcessState::Ready,
        burst_time: burst,
        remaining_time: burst,
        arrival_time: 0,
        priority: 5,
        memory_mb: 64.0,
        io_burst: None,
        estimated_burst: burst as f32,
        last_burst_actual: burst,
        finish_time: None,
        turnaround_time: None,
        waiting_time: None,
    }
}

/// Creates a standard PCB with a specific priority for priority-based algorithms.
#[cfg(test)]
pub fn make_pcb_with_priority(pid: u32, priority: u8) -> PCB {
    PCB {
        pid,
        name: format!("P{}", pid),
        state: ProcessState::Ready,
        burst_time: 10, // Default burst
        remaining_time: 10,
        arrival_time: 0,
        priority,
        memory_mb: 64.0,
        io_burst: None,
        estimated_burst: 10.0,
        last_burst_actual: 10,
        finish_time: None,
        turnaround_time: None,
        waiting_time: None,
    }
}
