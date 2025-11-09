# Compilazione con Feature Flags

Il progetto OSAL-RS supporta due backend differenti che possono essere selezionati tramite feature flags di Cargo.

## Backend Disponibili

### FreeRTOS (default)
- Feature flag: `freertos`
- Compila e linka FreeRTOS kernel v11.2.0
- Richiede CMake per la build
- Supporto per sistemi embedded

### POSIX
- Feature flag: `posix`
- Usa implementazioni POSIX native
- Non richiede dipendenze esterne
- Supporto per sistemi Unix-like

## Come Compilare

### Con FreeRTOS (default)
```bash
cargo build
# oppure esplicitamente
cargo build --features freertos
```

### Con POSIX
```bash
cargo build --no-default-features --features posix
```

### Per gli esempi

#### Esempio con FreeRTOS
```bash
cargo run --example basic
# oppure
cargo run --example basic --features freertos
```

#### Esempio con POSIX
```bash
cargo run --example basic --no-default-features --features posix
```

## API Unificata

Il modulo `os` viene automaticamente mappato al backend selezionato:
- Con feature `freertos`: `os` = `freertos`
- Con feature `posix`: `os` = `posix`

```rust
use osal_rs::{os, os_version};

fn main() {
    println!("Sistema: {}", os_version());
    let task = os::task::Task::new();
}
```

## Note

- Non è possibile abilitare entrambe le feature contemporaneamente
- La feature `freertos` è abilitata di default
- Quando si usa `posix`, assicurarsi di disabilitare la feature default con `--no-default-features`