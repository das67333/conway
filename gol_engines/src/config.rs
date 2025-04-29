use std::sync::atomic::{AtomicU32, Ordering};

struct Config {
    memory_manager_cap_log2: AtomicU32,
}

static CONFIG: Config = Config {
    memory_manager_cap_log2: AtomicU32::new(20),
};

pub struct ConfigSnapshot {
    pub memory_manager_cap_log2: u32,
}

pub fn get_config() -> ConfigSnapshot {
    ConfigSnapshot {
        memory_manager_cap_log2: CONFIG.memory_manager_cap_log2.load(Ordering::Relaxed),
    }
}

pub fn set_memory_manager_cap_log2(cap_log2: u32) {
    CONFIG
        .memory_manager_cap_log2
        .store(cap_log2, Ordering::Relaxed);
}
