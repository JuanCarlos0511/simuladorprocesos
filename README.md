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
simuladorprocesos/
├── docs/                       # Documentación y especificaciones
│   ├── reference_images/       # Capturas de pantalla de la interfaz
│   │   ├── image copy 2.png
│   │   ├── image copy 3.png
│   │   ├── image copy 4.png
│   │   ├── image copy.png
│   │   └── image.png
│   └── coding_standards.md     # Estándares de desarrollo de código
├── src/                        # Código fuente del Backend (Rust)
│   ├── constants/              # Constantes globales del sistema
│   │   └── mod.rs
│   ├── scheduler/              # Algoritmos de planificación de CPU
│   │   ├── fcfs.rs
│   │   ├── mod.rs
│   │   ├── priority_preemptive.rs
│   │   ├── round_robin.rs
│   │   └── sjf.rs
│   ├── utils/                  # Herramientas de utilidad y pruebas
│   │   ├── mod.rs
│   │   └── test_helpers.rs
│   ├── main.rs                 # Punto de entrada de la aplicación
│   ├── metrics.rs              # Lógica de cálculo de tiempos del sistema
│   ├── process.rs              # Estructura del Bloque de Control de Procesos (PCB)
│   └── simulation.rs           # Controlador del ciclo de simulación
├── target/                     # Archivos de compilación e intermediarios
│   ├── debug/                  # Binarios de depuración
│   ├── release/                # Binarios optimizados
│   └── .rustc_info.json        # Información del compilador de Rust
├── ui/                         # Código fuente del Frontend (Slint)
│   ├── components/             # Interfaces modulares y vistas
│   │   ├── common/             # Elementos de estilo HUD base
│   │   │   ├── hud_badge.slint
│   │   │   ├── hud_button.slint
│   │   │   ├── hud_card.slint
│   │   │   ├── hud_input.slint
│   │   │   └── hud_slider.slint
│   │   ├── dashboard/          # Contenedores métricos y colas
│   │   │   ├── dashboard.slint
│   │   │   ├── metric_card.slint
│   │   │   └── queue_item.slint
│   │   ├── gantt/              # Componentes de la gráfica de Gantt
│   │   │   ├── gantt_bar.slint
│   │   │   └── gantt_view.slint
│   │   ├── process_table/      # Tablas de administración de procesos
│   │   │   ├── create_modal.slint
│   │   │   ├── process_table.slint
│   │   │   └── table_row.slint
│   │   ├── edit_modal.slint    # Modal para modificar procesos
│   │   ├── init_modal.slint    # Modal de configuración inicial
│   │   └── sidebar.slint       # Panel de control lateral
│   ├── theme/                  # Estilos visuales generales
│   │   ├── colors.slint        # Paleta de colores (Modo Oscuro)
│   │   └── typography.slint    # Fuentes y tamaños tipográficos
│   ├── app.slint               # Ventana principal del entorno gráfico
│   ├── globals.slint           # Callbacks y propiedades globales de la interfaz
│   └── structs.slint           # Modelos de datos compartidos UI-Rust
├── .gitignore                  # Exclusiones de seguimiento de Git
├── Cargo.lock                  # Registro exacto de versiones de dependencias
├── Cargo.toml                  # Archivo de configuración del proyecto Cargo
├── README.md                   # Documentación principal del repositorio
├── build.rs                    # Script de compilación nativa de Rust
├── build_helper.bat            # Script de asistencia para compilación en Windows
└── env_check.bat               # Script de verificación de entorno de desarrollo

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
