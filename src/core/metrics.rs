/// Post-simulation metrics calculation.
///
/// Computes aggregate statistics (CPU utilization, average waiting/turnaround
/// times) from the list of terminated processes and the total simulation clock.

use crate::core::process::PCB;

/// Aggregate simulation metrics computed after a simulation run.
#[derive(Debug, Clone)]
pub struct SimulationMetrics {
    /// Total simulation time in ticks (ms).
    pub total_time: u32,
    /// CPU utilization as a percentage (0.0–100.0).
    pub cpu_utilization: f32,
    /// Average waiting time across all terminated processes (ms).
    pub avg_waiting_time: f32,
    /// Average turnaround time across all terminated processes (ms).
    pub avg_turnaround_time: f32,
    /// Delta in avg waiting time vs the previous simulation run.
    pub delta_waiting: f32,
    /// Delta in avg turnaround time vs the previous simulation run.
    pub delta_turnaround: f32,
}

impl SimulationMetrics {
    /// Returns a zeroed-out metrics struct.
    pub fn zero() -> Self {
        SimulationMetrics {
            total_time: 0,
            cpu_utilization: 0.0,
            avg_waiting_time: 0.0,
            avg_turnaround_time: 0.0,
            delta_waiting: 0.0,
            delta_turnaround: 0.0,
        }
    }
}

/// Calculates simulation metrics from the terminated process list.
///
/// # Arguments
/// * `terminated` - Slice of terminated PCBs (excluding sys_kernel_daemon).
/// * `total_clock` - The final clock tick when the simulation ended.
/// * `idle_ticks` - Number of ticks where no user process was running.
/// * `previous` - Optional previous metrics for delta calculation.
///
/// # Returns
/// A `SimulationMetrics` with all fields populated.
pub fn calculate_metrics(
    terminated: &[PCB],
    total_clock: u32,
    idle_ticks: u32,
    previous: Option<&SimulationMetrics>,
) -> SimulationMetrics {
    if terminated.is_empty() || total_clock == 0 {
        return SimulationMetrics::zero();
    }

    let n = terminated.len() as f32;

    let sum_waiting: f32 = terminated
        .iter()
        .filter_map(|p| p.waiting_time)
        .map(|w| w as f32)
        .sum();

    let sum_turnaround: f32 = terminated
        .iter()
        .filter_map(|p| p.turnaround_time)
        .map(|t| t as f32)
        .sum();

    let avg_waiting = sum_waiting / n;
    let avg_turnaround = sum_turnaround / n;

    let busy_ticks = total_clock.saturating_sub(idle_ticks);
    let cpu_utilization = (busy_ticks as f32 / total_clock as f32) * 100.0;

    let (delta_waiting, delta_turnaround) = match previous {
        Some(prev) => (
            avg_waiting - prev.avg_waiting_time,
            avg_turnaround - prev.avg_turnaround_time,
        ),
        None => (0.0, 0.0),
    };

    SimulationMetrics {
        total_time: total_clock,
        cpu_utilization,
        avg_waiting_time: avg_waiting,
        avg_turnaround_time: avg_turnaround,
        delta_waiting,
        delta_turnaround,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::process::ProcessState;

    fn make_terminated(arrival: u32, burst: u32, finish: u32) -> PCB {
        let pcb = PCB {
            pid: 1,
            name: "test".to_string(),
            state: ProcessState::Terminated,
            burst_time: burst,
            remaining_time: 0,
            arrival_time: arrival,
            priority: 5,
            memory_mb: 64.0,
            io_burst: None,
            finish_time: Some(finish),
            turnaround_time: Some(finish - arrival),
            waiting_time: Some(finish - arrival - burst),
        };
        pcb
    }

    #[test]
    fn metrics_with_empty_list() {
        let metrics = calculate_metrics(&[], 100, 0, None);
        assert_eq!(metrics.total_time, 0);
    }

    #[test]
    fn metrics_calculation_correctness() {
        let terminated = vec![
            make_terminated(0, 10, 10),  // wait=0, turn=10
            make_terminated(0, 5, 15),   // wait=10, turn=15
        ];
        let metrics = calculate_metrics(&terminated, 15, 0, None);
        assert_eq!(metrics.total_time, 15);
        assert!((metrics.avg_waiting_time - 5.0).abs() < 0.01);
        assert!((metrics.avg_turnaround_time - 12.5).abs() < 0.01);
        assert!((metrics.cpu_utilization - 100.0).abs() < 0.01);
    }

    #[test]
    fn delta_calculation() {
        let prev = SimulationMetrics {
            total_time: 10,
            cpu_utilization: 90.0,
            avg_waiting_time: 10.0,
            avg_turnaround_time: 20.0,
            delta_waiting: 0.0,
            delta_turnaround: 0.0,
        };
        let terminated = vec![make_terminated(0, 10, 22)]; // wait=12, turn=22
        let metrics = calculate_metrics(&terminated, 22, 0, Some(&prev));
        assert!((metrics.delta_waiting - 2.0).abs() < 0.01);
        assert!((metrics.delta_turnaround - 2.0).abs() < 0.01);
    }
}
