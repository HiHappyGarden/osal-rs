// Esempio semplicissimo: leggere CONFIG_TICK_RATE_HZ

use osal_rs::constants::CONFIG_TICK_RATE_HZ;

fn main() {
    println!("CONFIG_TICK_RATE_HZ = {} Hz", CONFIG_TICK_RATE_HZ);
}

// Output: CONFIG_TICK_RATE_HZ = 1000 Hz

