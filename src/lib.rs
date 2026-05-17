/// KERNEL_OS — Simulador de Gestión de Procesos (Library Crate)
///
/// Reexporta los módulos públicos del sistema organizados en capas:
/// - `core`: Núcleo del sistema operativo simulado (PCB, scheduler, simulación)
/// - `ipc`: Comunicación interproceso y sincronización concurrente
/// - `ui`: Interfaces de soporte (CLI y logger)
/// - `constants`: Parámetros y constantes globales del sistema

pub mod core;
pub mod ipc;
pub mod ui;
pub mod constants;
