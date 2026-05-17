# Simulador de Gestor de Procesos para Sistemas Operativos

## Descripción del Proyecto
Un simulador básico de un gestor de procesos desarrollado en Rust que emula el comportamiento de un sistema operativo al administrar múltiples procesos y recursos.

## Información del Curso
* **Materia:** Sistemas Operativos
* **Institución:** Universidad Autónoma de Tamaulipas
* **Semestre:** Periodo 2026-1 (Grupo 6 K)
* **Profesor(es):** Muñoz Quintero Dante Adolfo

## Integrantes del Equipo
* Hernández Del Angel José Ismael
* Hernández Torres José Leonardo
* Guzmán Antonio Juan Carlos
* Blanco Alejandre Eder Gael

## Estructura del Repositorio
```text
simulador-gestor-procesos/
├── README.md
├── Cargo.toml
├── .gitignore
├── LICENSE
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── process.rs
│   │   ├── scheduler.rs
│   │   └── resource.rs
│   ├── ipc/
│   │   ├── mod.rs
│   │   ├── channel.rs
│   │   └── semaphore.rs
│   └── ui/
│       ├── mod.rs
│       ├── cli.rs
│       └── logger.rs
├── tests/
│   ├── scheduler_tests.rs
│   ├── resource_tests.rs
│   └── ipc_tests.rs
├── examples/
│   ├── productor_consumidor.rs
│   └── round_robin_demo.rs
├── docs/
│   ├── entregable_final.pdf
│   └── diagrama_estados.png
├── capturas/
│   ├── screenshot01.png
│   └── screenshot_n.png
└── benches/
    └── scheduler_bench.rs
```
## 🛠️ Características Principales

El simulador implementa los 6 componentes requeridos por la rúbrica oficial:

* **Operaciones sobre Procesos:** Creación, suspensión temporizada, reanudación y liberación controlada mediante un Bloque de Control de Procesos (PCB) personalizado.
* **Asignación de Recursos:** Gestión restrictiva basada en un entorno físico simulado que limita la ejecución a un máximo de 1 CPU y 4 GB de RAM, detectando solicitudes inválidas o falta de recursos de forma preventiva.
* **Algoritmos de Planificación:** Soporte nativo para First-Come-First-Served (FCFS), Shortest Job First (SJF), Round Robin (con tamaño de Quantum dinámico y configurable) y selección por Prioridades.
* **Comunicación y Sincronización (IPC):** Sincronización libre de condiciones de carrera gracias al compilador de Rust, empleando exclusión mutua (Mutex), punteros de referencia contada (Arc) y canales asíncronos de paso de mensajes (mpsc). Incluye una demostración funcional para el problema del Productor-Consumidor.
* **Manejo de Terminación:** Rutinas dedicadas a la finalización normal y forzada de hilos, asegurando la devolución inmediata de los recursos al sistema operativo y el registro pormenorizado del motivo de salida.
* **Interfaz de Usuario Interactiva (CLI):** Pantalla interactiva en consola que muestra el estado actual del CPU, porcentaje libre de memoria, colas de planificación y un flujo de logs dinámico en la parte inferior.

---

## 🚀 Instalación y Ejecución
2. Compilar el Proyecto
Para generar el ejecutable de producción optimizado:

```Bash
cargo build --release
```
3. Ejecutar el Simulador Principal
Inicia la consola interactiva donde podrás operar el gestor:

```Bash
cargo run
```
4. Ejecutar Caso Demostrativo (Productor-Consumidor)
Para correr el escenario aislado de sincronización e hilos:

```Bash
cargo run --example productor_consumidor
```
5. Ejecutar Pruebas Automatizadas
Verifica que todos los algoritmos y restricciones de memoria funcionen correctamente:

```Bash
cargo test
```

### Prerrequisitos
Asegúrate de contar con la suite oficial de Rust instalada en tu sistema. Si no la tienes, puedes instalarla ejecutando:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
