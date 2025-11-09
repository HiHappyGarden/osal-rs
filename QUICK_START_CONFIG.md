# Come Leggere CONFIG_TICK_RATE_HZ da Rust - Guida Rapida

## 📖 Metodo 1: Import Diretto dalla Libreria (Raccomandato)

```rust
use osal_rs::constants::CONFIG_TICK_RATE_HZ;

fn main() {
    println!("Tick rate: {} Hz", CONFIG_TICK_RATE_HZ);
}
```

## 📖 Metodo 2: Import con Wildcard

```rust
use osal_rs::constants::*;

fn main() {
    println!("Tick rate: {} Hz", CONFIG_TICK_RATE_HZ);
    println!("Max priorities: {}", CONFIG_MAX_PRIORITIES);
}
```

## 📖 Metodo 3: Accesso dal Modulo FreeRTOS

```rust
use osal_rs::freertos;

fn main() {
    println!("Tick rate: {} Hz", freertos::constants::CONFIG_TICK_RATE_HZ);
}
```

## 🔧 Uso nelle Funzioni

```rust
use osal_rs::constants::{CONFIG_TICK_RATE_HZ, ms_to_ticks};

fn calculate_delay(milliseconds: u32) -> u32 {
    // Calcola il numero di tick necessari
    (milliseconds * CONFIG_TICK_RATE_HZ) / 1000
}

// O usa la funzione helper
fn easy_delay(milliseconds: u32) -> u32 {
    ms_to_ticks(milliseconds)
}

fn main() {
    let delay = calculate_delay(500);
    println!("500ms = {} ticks", delay);
    
    let easy = easy_delay(500);
    println!("500ms = {} ticks (helper)", easy);
}
```

## 🎯 Esempio Pratico: Thread con Delay Calcolato

```rust
use osal_rs::{Thread, ThreadDefaultPriority};
use osal_rs::constants::{CONFIG_TICK_RATE_HZ, ms_to_ticks};
use alloc::sync::Arc;

fn main() {
    println!("Sistema configurato a {} Hz", CONFIG_TICK_RATE_HZ);
    
    let thread = Thread::new(
        |_| {
            // Delay calcolato in base a CONFIG_TICK_RATE_HZ
            let delay_ticks = ms_to_ticks(1000); // 1 secondo
            println!("Delay di {} ticks", delay_ticks);
            Arc::new(())
        },
        "delay_thread",
        1024,
        None,
        ThreadDefaultPriority::Normal,
    ).unwrap();
}
```

## 📊 Tabella di Conversione con 1000 Hz

| Millisecondi | Ticks |
|--------------|-------|
| 1 ms         | 1     |
| 10 ms        | 10    |
| 100 ms       | 100   |
| 500 ms       | 500   |
| 1000 ms      | 1000  |

Formula: `ticks = (ms * CONFIG_TICK_RATE_HZ) / 1000`

## ✅ Test Rapido

Copia e incolla questo codice per testare:

```rust
use osal_rs::constants::*;

fn main() {
    // Test lettura CONFIG_TICK_RATE_HZ
    assert_eq!(CONFIG_TICK_RATE_HZ, 1000);
    println!("✓ CONFIG_TICK_RATE_HZ = {} Hz", CONFIG_TICK_RATE_HZ);
    
    // Test conversioni
    assert_eq!(ms_to_ticks(1000), 1000);
    println!("✓ 1000ms = {} ticks", ms_to_ticks(1000));
    
    assert_eq!(ticks_to_ms(1000), 1000);
    println!("✓ 1000 ticks = {} ms", ticks_to_ms(1000));
    
    println!("\n🎉 Tutti i test passano!");
}
```

## 🚀 Compila ed Esegui

```bash
# Compila la libreria
cargo build --features freertos

# Esegui l'esempio completo
cargo build --example freertos_config --features freertos

# Esegui i test
cargo test --lib --features freertos constants
```

## 📁 File Coinvolti

- **Costanti**: `src/freertos/constants.rs`
- **Configurazione C**: `include/FreeRTOSConfig.h`
- **Esempio**: `examples/freertos_config.rs`
- **Documentazione**: `FREERTOS_CONSTANTS.md`

## 💡 Suggerimenti

1. **Usa le funzioni helper** (`ms_to_ticks`, `ticks_to_ms`) invece di calcolare manualmente
2. **Verifica sempre** che le costanti in `constants.rs` corrispondano a `FreeRTOSConfig.h`
3. **Esegui i test** dopo ogni modifica alla configurazione
4. **Considera bindgen** per sincronizzazione automatica in progetti complessi

## ⚠️ Nota Importante

Se modifichi `CONFIG_TICK_RATE_HZ` in `FreeRTOSConfig.h`, ricorda di aggiornare anche il valore in `src/freertos/constants.rs`!

```c
// FreeRTOSConfig.h
#define configTICK_RATE_HZ  ( ( TickType_t ) 1000 )
```

```rust
// constants.rs
pub const CONFIG_TICK_RATE_HZ: TickType_t = 1000;
```

