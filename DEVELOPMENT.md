# Guida allo Sviluppo OSAL-RS

## Panoramica Tecnica

OSAL-RS è un progetto Rust che integra FreeRTOS kernel v11.2.0 utilizzando CMake come sistema di build per gestire la dipendenza da GitHub.

## Architettura

### Build System

Il progetto utilizza un sistema di build a più livelli:

1. **Cargo** - Build system principale di Rust
2. **build.rs** - Script Rust che invoca CMake
3. **CMake** - Gestisce il download e la compilazione di FreeRTOS
4. **FetchContent** - Modulo CMake per scaricare dipendenze da Git

### Flusso di Build

```
cargo build
    ↓
build.rs (Rust)
    ↓
CMake Configuration
    ↓
cmake/FreeRTOS.cmake
    ↓
FetchContent_Declare (GitHub)
    ↓
Download FreeRTOS v11.2.0
    ↓
Compile libfreertos.a
    ↓
Link con Rust
```

## Struttura del Progetto

```
osal-rs/
├── Cargo.toml              # Manifesto Rust con dipendenze
├── build.rs                # Script di build che invoca CMake
├── CMakeLists.txt          # Configurazione CMake principale
│
├── cmake/
│   └── FreeRTOS.cmake      # Download FreeRTOS da GitHub v11.2.0
│
├── include/
│   └── FreeRTOSConfig.h    # Configurazione FreeRTOS
│
├── src/
│   ├── lib.rs              # API principale OSAL
│   ├── bindings.rs         # FFI bindings per FreeRTOS
│   ├── task.rs             # Wrapper Rust per task
│   ├── queue.rs            # Wrapper Rust per code
│   └── semaphore.rs        # Wrapper Rust per semafori
│
└── examples/
    └── basic.rs            # Esempio di utilizzo
```

## Componenti Principali

### 1. build.rs

Script di build che:
- Configura CMake
- Passa la versione di FreeRTOS (V11.2.0)
- Specifica i path per il linker
- Linka la libreria statica libfreertos.a

### 2. cmake/FreeRTOS.cmake

Modulo CMake che:
- Usa `FetchContent_Declare` per scaricare da GitHub
- Repository: `https://github.com/FreeRTOS/FreeRTOS-Kernel.git`
- Tag: `V11.2.0`
- Scarica solo la versione specifica (GIT_SHALLOW TRUE)

### 3. CMakeLists.txt

Configura la build di FreeRTOS:
- Seleziona automaticamente la porta corretta (Posix per Linux)
- Compila i file sorgente core di FreeRTOS
- Crea la libreria statica `libfreertos.a`
- Installa header e libreria

### 4. FreeRTOSConfig.h

Configurazione runtime di FreeRTOS:
- Dimensioni heap, stack
- Priorità e scheduling
- Funzionalità abilitate

## Porte Supportate

Il sistema di build seleziona automaticamente la porta:

- **Linux**: `ThirdParty/GCC/Posix` (simulatore)
- **ARM Cortex-M4F**: `GCC/ARM_CM4F`
- **Altre architetture**: configurabili in CMakeLists.txt

## Sviluppo

### Aggiungere Nuove Funzionalità

1. **Definire i binding FFI** in `src/bindings.rs`
2. **Creare wrapper Rust** in moduli dedicati
3. **Implementare trait** per sicurezza e ergonomia
4. **Documentare** con esempi

### Esempio di Wrapper

```rust
// src/thread
pub struct Task {
    handle: *mut TaskHandle_t,
}

impl Task {
    pub fn create(name: &str, stack_size: usize) -> Result<Self, Error> {
        // Chiamata FFI a xTaskCreate
    }
    
    pub fn delete(self) {
        // Chiamata FFI a vTaskDelete
    }
}
```

## Testing

```bash
# Test unitari
cargo test

# Esegui esempio
cargo run --example basic

# Build release
cargo build --release
```

## Dipendenze

### Rust
- `cmake = "0.1"` - Integrazione CMake

### System
- CMake 3.15+
- Compilatore C (gcc, clang)
- Git

## FreeRTOS v11.2.0

### Novità della Versione

Dalla documentazione ufficiale:
- Supporto PAC/BTI per ARMv8-M
- Supporto FPU per ARM_AARCH64
- Miglioramenti porte CC-RH
- Aggiornamenti sicurezza e stabilità

### File Scaricati

Durante la build, vengono scaricati:
- Core kernel (tasks.c, queue.c, list.c, timers.c, etc.)
- Header files (FreeRTOS.h, task.h, queue.h, etc.)
- Portable layer per la piattaforma target
- Memory management (heap_4.c)

## Performance

### Dimensioni Build

- **Debug**: ~315KB (libfreertos.a)
- **Release**: ~200KB (con ottimizzazioni)

### Link Time

- Prima build: ~20s (download + compile)
- Build incrementali: <1s

## Troubleshooting

### Errore: portmacro.h not found

Verifica che la porta corretta sia selezionata in CMakeLists.txt.

### Errore: CMake non trovato

```bash
# Ubuntu/Debian
sudo apt-get install cmake

# Fedora
sudo dnf install cmake
```

### Link errors

Verifica che `build.rs` passi correttamente i path al linker.

## Riferimenti

- [FreeRTOS Kernel v11.2.0](https://github.com/FreeRTOS/FreeRTOS-Kernel/tree/V11.2.0)
- [CMake FetchContent](https://cmake.org/cmake/help/latest/module/FetchContent.html)
- [Rust FFI](https://doc.rust-lang.org/nomicon/ffi.html)
- [Cargo Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
