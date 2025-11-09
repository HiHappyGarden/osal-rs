# Testing OSAL-RS

## Panoramica

OSAL-RS contiene test per verificare la correttezza del codice. Tuttavia, i test sono divisi in due categorie:

### 1. Test Unitari (funzionano su host)

Questi test verificano la logica del codice Rust senza richiedere l'esecuzione effettiva di FreeRTOS:

```bash
# Test solo per i moduli che non richiedono runtime RTOS
cargo test --lib --no-default-features

# Test per il modulo POSIX (funziona su Linux/macOS)
cargo test --lib --no-default-features --features posix
```

### 2. Test FreeRTOS (richiedono ambiente embedded)

I test per il backend FreeRTOS richiedono un ambiente embedded reale o un emulatore perché FreeRTOS:
- Non fornisce una funzione `main` standard
- Richiede configurazioni hardware specifiche  
- Non può essere eseguito nativamente su Linux/macOS/Windows

#### Test Disponibili per FreeRTOS

Nel file `src/freertos/thread.rs`, i test sono divisi in:

**Test unitari** (già attivi):
- `test_thread_priority_values` - Verifica i valori delle priorità
- `test_thread_priority_trait` - Test del trait ThreadPriority
- `test_thread_priority_clone` - Test della clonazione delle priorità
- `test_arc_callback_type` - Test del tipo di callback
- `test_thread_struct_clone` - Test della clonazione di Thread

**Test di integrazione** (commentati, richiedono FreeRTOS):
- `test_thread_creation` - Creazione di thread reali
- `test_thread_with_param` - Thread con parametri
- `test_thread_priorities` - Test con diverse priorità
- `test_thread_name_validation` - Validazione nomi thread

#### Come Eseguire i Test FreeRTOS

Per eseguire i test di integrazione FreeRTOS, è necessario:

1. **Usare un target embedded** (es. ARM Cortex-M):
   ```bash
   cargo test --target thumbv7em-none-eabihf --features freertos
   ```

2. **Usare QEMU o un emulatore**:
   ```bash
   # Esempio con QEMU (richiede configurazione specifica)
   qemu-system-arm -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native \
     -kernel target/thumbv7em-none-eabihf/debug/deps/osal_rs-<hash>
   ```

3. **Usare hardware reale**:
   - Compilare per il target specifico
   - Flashare il firmware
   - Eseguire i test on-device

#### Abilitare i Test di Integrazione

Per abilitare i test commentati nel file `thread.rs`, rimuovere i commenti `/*` e `*/` attorno alla sezione `integration_tests` quando si lavora in un ambiente embedded.

## Continuous Integration

Per CI/CD su GitHub Actions o simili:

```yaml
# Test su host (senza FreeRTOS)
- name: Run unit tests
  run: cargo test --lib --no-default-features --features posix

# Test di compilazione FreeRTOS (senza esecuzione)
- name: Check FreeRTOS compilation
  run: cargo check --lib --features freertos
```

## Test POSIX

I test POSIX possono essere eseguiti normalmente su Linux/macOS:

```bash
cargo test --lib --no-default-features --features posix
```

## Note

- I test unitari nel modulo `freertos/thread.rs` verificano la correttezza della logica senza eseguire codice FreeRTOS
- Per test completi end-to-end, è necessario un ambiente embedded
- Il codice compila correttamente con `cargo build --features freertos`
- Gli esempi possono essere compilati per target embedded specifici

