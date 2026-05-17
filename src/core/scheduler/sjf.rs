/// SJF (Shortest Job First) scheduling algorithm — non-preemptive.
///
/// Selects the process with the smallest total burst time from the ready queue.
/// Only evaluated when the CPU becomes free.

use std::collections::VecDeque;

use crate::core::process::PCB;
use super::SchedulingAlgorithm;

pub struct Sjf;

impl SchedulingAlgorithm for Sjf {
    fn select_next(&self, ready_queue: &VecDeque<PCB>) -> Option<usize> {
        if ready_queue.is_empty() {
            return None;
        }

        // Find the process with the shortest burst time (skip kernel daemon)
        let mut best_idx: Option<usize> = None;
        let mut best_burst = u32::MAX;

        for (i, proc) in ready_queue.iter().enumerate() {
            if proc.is_kernel_daemon() {
                continue;
            }
            if proc.remaining_time < best_burst {
                best_burst = proc.remaining_time;
                best_idx = Some(i);
            }
        }

        // If all are kernel daemon, return first
        best_idx.or(Some(0))
    }

    fn name(&self) -> &'static str {
        "SJF"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scheduler::test_helpers::make_pcb;

    #[test]
    fn selects_shortest_burst() {
        let algo = Sjf;
        let mut queue = VecDeque::new();
        queue.push_back(make_pcb(1, 10));
        queue.push_back(make_pcb(2, 3));
        queue.push_back(make_pcb(3, 7));
        assert_eq!(algo.select_next(&queue), Some(1)); // PID 2, burst=3
    }
}
