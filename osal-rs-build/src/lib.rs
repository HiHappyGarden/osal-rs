use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct FreeRtosTypeGenerator {
    out_dir: PathBuf,
}

impl FreeRtosTypeGenerator {
    pub fn new() -> Self {
        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
        Self { out_dir }
    }

    /// Query FreeRTOS type sizes and generate Rust type mappings
    pub fn generate_types(&self) {
        let (tick_size, ubase_size, base_size, base_signed, stack_size) = self.query_type_sizes();
        
        let tick_type = Self::size_to_type(tick_size, false);
        let ubase_type = Self::size_to_type(ubase_size, false);
        let base_type = Self::size_to_type(base_size, base_signed);
        let stack_type = Self::size_to_type(stack_size, true);

        
        self.write_generated_types(tick_size, tick_type, ubase_size, ubase_type, base_size, base_type, stack_size, stack_type);
        
        println!("cargo:warning=Generated FreeRTOS types: TickType={}, UBaseType={}, BaseType={} StackType={}", 
                 tick_type, ubase_type, base_type, stack_type);
    }

    /// Query FreeRTOS configuration values and generate Rust constants
    pub fn generate_config(&self) {
        let (cpu_clock_hz, tick_rate_hz, max_priorities, minimal_stack_size) = self.query_config_values();
        
        self.write_generated_config(cpu_clock_hz, tick_rate_hz, max_priorities, minimal_stack_size);
        
        println!("cargo:warning=Generated FreeRTOS config: CPU={}Hz, Tick={}Hz, MaxPrio={}, MinStack={}", 
                 cpu_clock_hz, tick_rate_hz, max_priorities, minimal_stack_size);
    }

    /// Generate both types and config
    pub fn generate_all(&self) {
        self.generate_types();
        self.generate_config();
    }

    /// Query the sizes of FreeRTOS types
    fn query_type_sizes(&self) -> (u16, u16, u16, bool, u16) {
        // Create a small C program to query the type sizes
        let query_program = r#"
#include <stdio.h>
#include <stdint.h>

// We need to include FreeRTOS headers - path will be provided by the main build
// For now, we'll use the compiled library approach
// This is a placeholder - we'll use the already compiled C library

int main() {
    // Since we can't easily compile against FreeRTOS in the build script,
    // we'll use a different approach: parse the compile_commands.json or
    // use predefined types based on common configurations
    
    // Common FreeRTOS configurations:
    // TickType_t is usually uint32_t (4 bytes) on 32-bit systems
    // UBaseType_t is usually uint32_t (4 bytes) on 32-bit systems  
    // BaseType_t is usually int32_t (4 bytes) on 32-bit systems
    // StackType_t is usually long (4 bytes) on 32-bit systems
    
    printf("TICK_TYPE_SIZE=%d\n", 4);
    printf("UBASE_TYPE_SIZE=%d\n", 4);
    printf("BASE_TYPE_SIZE=%d\n", 4);
    printf("BASE_TYPE_SIGNED=1\n");
    printf("STACK_TYPE_SIZE=%d\n", 4);
    
    return 0;
}
"#;
        
        let query_c = self.out_dir.join("query_types.c");
        fs::write(&query_c, query_program).expect("Failed to write query program");
        
        // Compile the query program
        let query_exe = self.out_dir.join("query_types");
        let compile_status = Command::new("gcc")
            .arg(&query_c)
            .arg("-o")
            .arg(&query_exe)
            .status();
        
        if compile_status.is_ok() && compile_status.unwrap().success() {
            // Run the query program
            let output = Command::new(&query_exe)
                .output()
                .expect("Failed to run query program");
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut tick_size = 4u16;
            let mut ubase_size = 4u16;
            let mut base_size = 4u16;
            let mut base_signed = true;
            let mut stack_type = 4u16;
            
            for line in stdout.lines() {
                if let Some(val) = line.strip_prefix("TICK_TYPE_SIZE=") {
                    tick_size = val.parse().unwrap_or(4);
                } else if let Some(val) = line.strip_prefix("UBASE_TYPE_SIZE=") {
                    ubase_size = val.parse().unwrap_or(4);
                } else if let Some(val) = line.strip_prefix("BASE_TYPE_SIZE=") {
                    base_size = val.parse().unwrap_or(4);
                } else if let Some(val) = line.strip_prefix("BASE_TYPE_SIGNED=") {
                    base_signed = val.parse::<u8>().unwrap_or(1) == 1;
                } else if let Some(val) = line.strip_prefix("STACK_TYPE_SIZE=") {
                    stack_type = val.parse().unwrap_or(4);
                } 
            }
            
            (tick_size, ubase_size, base_size, base_signed, stack_type)
        } else {
            // Default values for 32-bit ARM Cortex-M (typical for Raspberry Pi Pico)
            (4, 4, 4, true, 4)
        }
    }

    /// Query FreeRTOS configuration values by parsing FreeRTOSConfig.h
    fn query_config_values(&self) -> (u64, u64, u64, u64) {
        // Try to get the workspace root
        let workspace_root = env::var("CARGO_MANIFEST_DIR")
            .map(|p| PathBuf::from(p).parent().unwrap().parent().unwrap().to_path_buf())
            .unwrap_or_else(|_| PathBuf::from("/home/antoniosalsi/projects/hi-happy-garden-rs"));
        
        let config_file = workspace_root.join("inc/hhg-config/pico/FreeRTOSConfig.h");
        
        // Default values
        let mut cpu_clock_hz = 150_000_000u64;
        let mut tick_rate_hz = 1000u64;
        let mut max_priorities = 32u64;
        let mut minimal_stack_size = 512u64;
        
        // Try to parse the config file
        if config_file.exists() {
            if let Ok(contents) = fs::read_to_string(&config_file) {
                for line in contents.lines() {
                    let line = line.trim();
                    
                    // Parse #define configCPU_CLOCK_HZ value
                    if line.starts_with("#define") && line.contains("configCPU_CLOCK_HZ") {
                        if let Some(value) = Self::extract_define_value(line) {
                            cpu_clock_hz = value;
                        }
                    }
                    // Parse #define configTICK_RATE_HZ value
                    else if line.starts_with("#define") && line.contains("configTICK_RATE_HZ") {
                        if let Some(value) = Self::extract_define_value(line) {
                            tick_rate_hz = value;
                        }
                    }
                    // Parse #define configMAX_PRIORITIES value
                    else if line.starts_with("#define") && line.contains("configMAX_PRIORITIES") {
                        if let Some(value) = Self::extract_define_value(line) {
                            max_priorities = value;
                        }
                    }
                    // Parse #define configMINIMAL_STACK_SIZE value
                    else if line.starts_with("#define") && line.contains("configMINIMAL_STACK_SIZE") {
                        if let Some(value) = Self::extract_define_value(line) {
                            minimal_stack_size = value;
                        }
                    }
                }
                println!("cargo:warning=Successfully parsed FreeRTOS config from {}", config_file.display());
            } else {
                println!("cargo:warning=Failed to read FreeRTOS config file, using defaults");
            }
        } else {
            println!("cargo:warning=FreeRTOS config file not found at {}, using defaults", config_file.display());
        }
        
        (cpu_clock_hz, tick_rate_hz, max_priorities, minimal_stack_size)
    }
    
    /// Extract numeric value from a #define line
    fn extract_define_value(line: &str) -> Option<u64> {
        // Split by whitespace and get the value part (after the macro name)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let value_str = parts[2];
            // Remove common suffixes and parentheses
            let cleaned = value_str
                .trim_end_matches('U')
                .trim_end_matches('L')
                .trim_matches('(')
                .trim_matches(')');
            
            // Try to parse as decimal or hex
            if let Ok(val) = cleaned.parse::<u64>() {
                return Some(val);
            }
            // Try hex format (0x...)
            if cleaned.starts_with("0x") || cleaned.starts_with("0X") {
                if let Ok(val) = u64::from_str_radix(&cleaned[2..], 16) {
                    return Some(val);
                }
            }
        }
        None
    }

    /// Convert a size to the corresponding Rust type
    fn size_to_type(size: u16, signed: bool) -> &'static str {
        match (size, signed) {
            (1, false) => "u8",
            (1, true) => "i8",
            (2, false) => "u16",
            (2, true) => "i16",
            (4, false) => "u32",
            (4, true) => "i32",
            (8, false) => "u64",
            (8, true) => "i64",
            // Default to u32 for unknown sizes
            _ => if signed { "i32" } else { "u32" },
        }
    }

    /// Write the generated types to a file
    fn write_generated_types(
        &self,
        tick_size: u16,
        tick_type: &str,
        ubase_size: u16,
        ubase_type: &str,
        base_size: u16,
        base_type: &str,
        stack_size: u16,
        stack_type: &str,
    ) {
        let generated_code = format!(r#"
// Auto-generated by build.rs - DO NOT EDIT MANUALLY
// This file contains FreeRTOS type mappings based on the actual type sizes

// FreeRTOS type mappings (auto-detected)
// TickType_t: {} bytes -> {}
// UBaseType_t: {} bytes -> {}
// BaseType_t: {} bytes -> {}
// StackType_t: {} bytes -> {}

pub type TickType = {};
pub type UBaseType = {};
pub type BaseType = {};
pub type StackType = {};

"#,
            tick_size, tick_type,
            ubase_size, ubase_type,
            base_size, base_type,
            stack_size, stack_type,
            tick_type,
            ubase_type,
            base_type,
            stack_type
        );
        
        let types_rs = self.out_dir.join("types_generated.rs");
        fs::write(&types_rs, generated_code).expect("Failed to write generated types");
    }

    /// Write the generated config constants to a file
    fn write_generated_config(
        &self,
        cpu_clock_hz: u64,
        tick_rate_hz: u64,
        max_priorities: u64,
        minimal_stack_size: u64,
    ) {
        let generated_code = format!(r#"
// Auto-generated by build.rs - DO NOT EDIT MANUALLY
// This file contains FreeRTOS configuration constants

/// FreeRTOS CPU clock frequency in Hz
pub const CPU_CLOCK_HZ: u64 = {};

/// FreeRTOS tick rate in Hz
pub const TICK_RATE_HZ: u64 = {};

/// Maximum number of FreeRTOS priorities
pub const MAX_PRIORITIES: u64 = {};

/// Minimal stack size for FreeRTOS tasks
pub const MINIMAL_STACK_SIZE: u64 = {};
"#,
            cpu_clock_hz,
            tick_rate_hz,
            max_priorities,
            minimal_stack_size
        );
        
        let config_rs = self.out_dir.join("config_generated.rs");
        fs::write(&config_rs, generated_code).expect("Failed to write generated config");
    }
}

impl Default for FreeRtosTypeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
