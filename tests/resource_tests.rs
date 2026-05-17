/// Pruebas de integración de caja negra — Pool de Recursos.
///
/// Valida los límites de asignación de CPU y memoria del ResourcePool.

use simulador_procesos::core::resource::ResourcePool;

#[test]
fn initial_resources_are_correct() {
    let pool = ResourcePool::new();
    assert_eq!(pool.cpu_cores_free, 1);
    assert_eq!(pool.memory_free_mb, 4096);
}

#[test]
fn request_within_limits_succeeds() {
    let mut pool = ResourcePool::new();
    assert!(pool.request(1, 2048).is_ok());
    assert_eq!(pool.cpu_cores_free, 0);
    assert_eq!(pool.memory_free_mb, 2048);
}

#[test]
fn request_exceeds_cpu_fails() {
    let mut pool = ResourcePool::new();
    let result = pool.request(2, 1024);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("CPU insuficiente"));
}

#[test]
fn request_exceeds_memory_fails() {
    let mut pool = ResourcePool::new();
    let result = pool.request(1, 5000);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Memoria insuficiente"));
}

#[test]
fn release_restores_resources() {
    let mut pool = ResourcePool::new();
    pool.request(1, 2048).unwrap();
    pool.release(1, 2048);
    assert_eq!(pool.cpu_cores_free, 1);
    assert_eq!(pool.memory_free_mb, 4096);
}

#[test]
fn multiple_requests_accumulate() {
    let mut pool = ResourcePool::new();
    assert!(pool.request(1, 1024).is_ok());
    // CPU is now 0, second request should fail
    assert!(pool.request(1, 1024).is_err());
}

#[test]
fn default_is_same_as_new() {
    let pool = ResourcePool::default();
    assert_eq!(pool.cpu_cores_free, 1);
    assert_eq!(pool.memory_free_mb, 4096);
}
