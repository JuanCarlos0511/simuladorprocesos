# KERNEL_OS — Simulador de Gestión de Procesos

Aplicación de escritorio nativa que simula la gestión de procesos de un sistema operativo con interfaz visual estilo cyberpunk/terminal de kernel.

## Características

- **6 algoritmos de planificación**: FCFS, SJF, SRTF, Round Robin, Prioridad, Prioridad Preemptiva
- **Simulación en tiempo real** con velocidad ajustable (1x, 2x, 5x, 10x)
- **Dashboard interactivo** con colas de procesos, CPU panel y logs
- **Diagrama de Gantt** con métricas post-simulación
- **Gestión de procesos** con tabla, edición y acciones de lote
- **Eventos de I/O aleatorios** con 15% de probabilidad por tick
- **Ejecutable portable** sin dependencias en tiempo de ejecución

## Requisitos de Compilación

### Windows

1. Instalar [Rust](https://rustup.rs/) (rustup)
2. Desde la terminal:
```powershell
cargo build --release
```
3. El ejecutable estará en `target/release/simulador-procesos.exe`

### Linux

1. Instalar Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2. Instalar dependencias del sistema (para el backend de renderizado):
```bash
# Ubuntu/Debian
sudo apt install libfontconfig1-dev libfreetype6-dev

# Fedora
sudo dnf install fontconfig-devel freetype-devel
```
3. Compilar:
```bash
cargo build --release
```
4. El ejecutable estará en `target/release/simulador-procesos`

## Ejecución en Modo Desarrollo

```bash
cargo run
```

## Stack Técnico

| Capa | Tecnología |
|------|-----------|
| Backend / Lógica | Rust (Edition 2021) |
| GUI | Slint 1.16 |
| RNG | rand 0.8 |

## Estructura del Proyecto

```
simuladorprocesos/
├── docs/                       # Documentación y especificaciones
│   ├── reference_images/       # Capturas de pantalla de la interfaz
│   └── coding_standards.md     # Estándares de desarrollo de código
├── src/                        # Capa de Software (Backend Computacional en Rust)
│   ├── constants/              # Constantes y parámetros globales del sistema
│   │   └── mod.rs
│   ├── core/                   # Módulo Núcleo: Orquestación del Sistema Operativo
│   │   ├── mod.rs              # Exportación y visibilidad del núcleo
│   │   ├── process.rs          # Bloque de Control de Procesos (PCB) y transiciones de estado
│   │   ├── resource.rs         # Pool y control de asignación de CPU y 4 GB de memoria RAM
│   │   └── scheduler.rs        # Algoritmos de planificación (FCFS, SJF, RR, Prioridad)
│   ├── ipc/                    # Módulo IPC: Sincronización y Comunicación Concurrente
│   │   ├── mod.rs              # Exportación de primitivas de sincronización
│   │   ├── channel.rs          # Paso de mensajes asíncronos mediante canales (mpsc)
│   │   └── semaphore.rs        # Control de exclusión mutua mediante Mutex y Condvar
│   ├── ui/                     # Capa de Interfaz y Registro Exigida por Rúbrica
│   │   ├── mod.rs              # Exportación de la interfaz de soporte
│   │   ├── cli.rs              # Menú interactivo CLI de consola (Requisito del sistema)
│   │   └── logger.rs           # Sistema de logs cronológicos de eventos del simulador
│   ├── lib.rs                  # Archivo de biblioteca para reexportar los módulos públicos
│   └── main.rs                 # Punto de entrada principal y arranque coordinado
├── tests/                      # Pruebas de Integración de Caja Negra (Obligatorio)
│   ├── ipc_tests.rs            # Validación de canales y semáforos concurrentes
│   ├── resource_tests.rs       # Pruebas de límites de asignación de hardware
│   └── scheduler_tests.rs      # Certificación de cálculo de tiempos por algoritmo
├── examples/                   # Casos de Uso Demostrativos Ejecutables (Obligatorio)
│   ├── productor_consumidor.rs # Simulación del problema clásico con semáforos
│   └── round_robin_demo.rs     # Script aislado demostrativo de ráfagas temporales
├── benches/                    # Pruebas de Rendimiento y Carga
│   └── scheduler_bench.rs      # Análisis de latencia de las colas de planificación
├── ui/                         # Frontend Avanzado (Código nativo de Slint UI)
│   ├── components/             # Modales, Dashboard, barras y tablas de la interfaz
│   ├── theme/                  # Paleta de colores HUD (Modo Oscuro) y fuentes
│   ├── app.slint               # Ventana principal del entorno gráfico
│   ├── globals.slint           # Callbacks y propiedades globales de la interfaz
│   └── structs.slint           # Modelos de datos compartidos de la UI con Rust
├── .gitignore                  # Exclusiones de Git (Previene el rastreo de target/)
├── build.rs                    # Script de enlace de compilación nativa Rust-Slint
├── build_helper.bat            # Script de asistencia para compilación en Windows
├── Cargo.lock                  # Registro estricto de control de versiones de crates
├── Cargo.toml                  # Configuración del proyecto de Cargo
└── env_check.bat               # Verificador de dependencias del entorno local
```

## Licencia

Uso educativo.