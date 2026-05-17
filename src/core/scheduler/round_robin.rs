/// Round Robin scheduling algorithm.
///
/// Selects the first process in the ready queue (FIFO). The process runs
/// for at most `quantum` ticks before being preempted and sent to the
/// back of the queue. Quantum management is handled by the Scheduler.

use std::collections::VecDeque;

use crate::core::process::PCB;
use super::SchedulingAlgorithm;

pub struct RoundRobin;

impl SchedulingAlgorithm for RoundRobin {
    fn select_next(&self, ready_queue: &VecDeque<PCB>) -> Option<usize> {
        if ready_queue.is_empty() {
            None
        } else {
            // Skip kernel daemon if there are other processes
            let non_kernel = ready_queue
                .iter()
                .position(|p| !p.is_kernel_daemon());
            non_kernel.or(Some(0))
        }
    }

    fn uses_quantum(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "Round Robin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scheduler::test_helpers::make_pcb;

    #[test]
    fn selects_first_in_queue() {
        let algo = RoundRobin;
        let mut queue = VecDeque::new();
        queue.push_back(make_pcb(1, 10));
        queue.push_back(make_pcb(2, 5));
        assert_eq!(algo.select_next(&queue), Some(0));
    }

    #[test]
    fn uses_quantum_flag() {
        let algo = RoundRobin;
        assert!(algo.uses_quantum());
    }
}
