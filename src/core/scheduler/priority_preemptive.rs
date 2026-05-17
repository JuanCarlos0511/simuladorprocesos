/// Priority Preemptive scheduling algorithm.
///
/// Selects the process with the highest priority (lowest numeric value).
/// At each tick, if a process with higher priority than the current one
/// exists in the ready queue, the current process is preempted.

use std::collections::VecDeque;

use crate::core::process::PCB;
use super::SchedulingAlgorithm;

pub struct PriorityPreemptive;

impl SchedulingAlgorithm for PriorityPreemptive {
    fn select_next(&self, ready_queue: &VecDeque<PCB>) -> Option<usize> {
        if ready_queue.is_empty() {
            return None;
        }

        let mut best_idx: Option<usize> = None;
        let mut best_priority = u8::MAX;

        for (i, proc) in ready_queue.iter().enumerate() {
            if proc.is_kernel_daemon() {
                continue;
            }
            if proc.priority < best_priority {
                best_priority = proc.priority;
                best_idx = Some(i);
            }
        }

        best_idx.or(Some(0))
    }

    fn should_preempt(&self, current: &PCB, ready_queue: &VecDeque<PCB>) -> bool {
        if current.is_kernel_daemon() {
            return ready_queue.iter().any(|p| !p.is_kernel_daemon());
        }
        ready_queue
            .iter()
            .filter(|p| !p.is_kernel_daemon())
            .any(|p| p.priority < current.priority)
    }

    fn name(&self) -> &'static str {
        "Prioridad Preemptivo"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scheduler::test_helpers::make_pcb_with_priority as make_pcb;

    #[test]
    fn preempts_for_higher_priority() {
        let algo = PriorityPreemptive;
        let current = make_pcb(1, 5);
        let mut queue = VecDeque::new();
        queue.push_back(make_pcb(2, 2));
        assert!(algo.should_preempt(&current, &queue));
    }

    #[test]
    fn no_preempt_for_lower_priority() {
        let algo = PriorityPreemptive;
        let current = make_pcb(1, 2);
        let mut queue = VecDeque::new();
        queue.push_back(make_pcb(2, 8));
        assert!(!algo.should_preempt(&current, &queue));
    }
}
