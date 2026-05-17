/// Resource Pool — models CPU cores and physical memory for the simulated OS.
///
/// Provides safe request/release semantics to prevent over-allocation.

/// Pool of hardware resources available to the simulated operating system.
#[derive(Debug, Clone)]
pub struct ResourcePool {
    /// Number of free CPU cores available for scheduling.
    pub cpu_cores_free: u8,
    /// Free RAM available in megabytes.
    pub memory_free_mb: u32,
}

impl ResourcePool {
    /// Creates a new ResourcePool with 1 CPU core and 4096 MB RAM.
    pub fn new() -> Self {
        ResourcePool {
            cpu_cores_free: 1,
            memory_free_mb: 4096,
        }
    }

    /// Requests hardware resources for a process.
    ///
    /// Returns `Ok(())` if enough resources are available, or `Err(String)`
    /// describing the shortage.
    pub fn request(&mut self, cpu: u8, mem_mb: u32) -> Result<(), String> {
        if cpu > self.cpu_cores_free {
            return Err(format!(
                "CPU insuficiente: solicitados {} cores, disponibles {}",
                cpu, self.cpu_cores_free
            ));
        }
        if mem_mb > self.memory_free_mb {
            return Err(format!(
                "Memoria insuficiente: solicitados {}MB, disponibles {}MB",
                mem_mb, self.memory_free_mb
            ));
        }
        self.cpu_cores_free -= cpu;
        self.memory_free_mb -= mem_mb;
        Ok(())
    }

    /// Releases hardware resources back to the pool.
    pub fn release(&mut self, cpu: u8, mem_mb: u32) {
        self.cpu_cores_free = self.cpu_cores_free.saturating_add(cpu);
        self.memory_free_mb = self.memory_free_mb.saturating_add(mem_mb);
    }
}

impl Default for ResourcePool {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_resources() {
        let pool = ResourcePool::new();
        assert_eq!(pool.cpu_cores_free, 1);
        assert_eq!(pool.memory_free_mb, 4096);
    }

    #[test]
    fn request_within_limits() {
        let mut pool = ResourcePool::new();
        assert!(pool.request(1, 2048).is_ok());
        assert_eq!(pool.cpu_cores_free, 0);
        assert_eq!(pool.memory_free_mb, 2048);
    }

    #[test]
    fn request_exceeds_cpu() {
        let mut pool = ResourcePool::new();
        assert!(pool.request(2, 1024).is_err());
    }

    #[test]
    fn request_exceeds_memory() {
        let mut pool = ResourcePool::new();
        assert!(pool.request(1, 5000).is_err());
    }

    #[test]
    fn release_restores_resources() {
        let mut pool = ResourcePool::new();
        pool.request(1, 2048).unwrap();
        pool.release(1, 2048);
        assert_eq!(pool.cpu_cores_free, 1);
        assert_eq!(pool.memory_free_mb, 4096);
    }
}
