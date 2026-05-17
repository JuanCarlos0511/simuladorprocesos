/// Pruebas de integración de caja negra — Algoritmos de Planificación.
///
/// Valida el comportamiento correcto de los algoritmos FCFS, SJF,
/// Round Robin y Prioridad a través de la API pública de la biblioteca.

use std::collections::VecDeque;
use simulador_procesos::core::process::{PCB, ProcessState};
use simulador_procesos::core::scheduler::{Algorithm, Scheduler, SchedulingAlgorithm};
use simulador_procesos::core::scheduler::fcfs::Fcfs;
use simulador_procesos::core::scheduler::sjf::Sjf;
use simulador_procesos::core::scheduler::round_robin::RoundRobin;
use simulador_procesos::core::scheduler::priority_preemptive::PriorityPreemptive;

fn make_pcb(pid: u32, burst: u32) -> PCB {
    PCB {
        pid, name: format!("P{}", pid), state: ProcessState::Ready,
        burst_time: burst, remaining_time: burst, arrival_time: 0,
        priority: 5, memory_mb: 64.0, io_burst: None,
        finish_time: None, turnaround_time: None, waiting_time: None,
    }
}

fn make_pcb_prio(pid: u32, priority: u8) -> PCB {
    PCB {
        pid, name: format!("P{}", pid), state: ProcessState::Ready,
        burst_time: 10, remaining_time: 10, arrival_time: 0,
        priority, memory_mb: 64.0, io_burst: None,
        finish_time: None, turnaround_time: None, waiting_time: None,
    }
}

#[test]
fn fcfs_selects_first_arrived() {
    let algo = Fcfs;
    let mut queue = VecDeque::new();
    queue.push_back(make_pcb(1, 20));
    queue.push_back(make_pcb(2, 5));
    queue.push_back(make_pcb(3, 10));
    assert_eq!(algo.select_next(&queue), Some(0));
}

#[test]
fn sjf_selects_shortest_burst() {
    let algo = Sjf;
    let mut queue = VecDeque::new();
    queue.push_back(make_pcb(1, 20));
    queue.push_back(make_pcb(2, 3));
    queue.push_back(make_pcb(3, 10));
    assert_eq!(algo.select_next(&queue), Some(1));
}

#[test]
fn round_robin_uses_quantum() {
    let algo = RoundRobin;
    assert!(algo.uses_quantum());
}

#[test]
fn priority_preempts_higher_priority() {
    let algo = PriorityPreemptive;
    let current = make_pcb_prio(1, 5);
    let mut queue = VecDeque::new();
    queue.push_back(make_pcb_prio(2, 2)); // higher priority
    assert!(algo.should_preempt(&current, &queue));
}

#[test]
fn priority_no_preempt_lower_priority() {
    let algo = PriorityPreemptive;
    let current = make_pcb_prio(1, 2);
    let mut queue = VecDeque::new();
    queue.push_back(make_pcb_prio(2, 8)); // lower priority
    assert!(!algo.should_preempt(&current, &queue));
}

#[test]
fn scheduler_next_process_dequeues() {
    let mut scheduler = Scheduler::new(Algorithm::FCFS, 4);
    scheduler.add_process(make_pcb(1, 10));
    scheduler.add_process(make_pcb(2, 5));

    let next = scheduler.next_process();
    assert!(next.is_some());
    assert_eq!(next.unwrap().pid, 1);
}

#[test]
fn scheduler_empty_returns_none() {
    let mut scheduler = Scheduler::new(Algorithm::FCFS, 4);
    assert!(scheduler.next_process().is_none());
}

#[test]
fn algorithm_labels_are_correct() {
    assert_eq!(Algorithm::FCFS.label(), "FCFS");
    assert_eq!(Algorithm::SJF.label(), "SJF");
    assert_eq!(Algorithm::RoundRobin.label(), "Round Robin");
    assert_eq!(Algorithm::Priority.label(), "Prioridad");
}

#[test]
fn algorithm_from_index_covers_all() {
    assert_eq!(Algorithm::from_index(0), Algorithm::FCFS);
    assert_eq!(Algorithm::from_index(1), Algorithm::SJF);
    assert_eq!(Algorithm::from_index(2), Algorithm::RoundRobin);
    assert_eq!(Algorithm::from_index(3), Algorithm::Priority);
    assert_eq!(Algorithm::from_index(99), Algorithm::FCFS); // default
}
