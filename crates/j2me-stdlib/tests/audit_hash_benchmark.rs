use std::time::Instant;

fn hash_framebuffer_old(fb: &[u32]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &p in fb {
        h ^= p as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn hash_framebuffer_new(fb: &[u32]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i + 1 < fb.len() {
        let p = (fb[i] as u64) | ((fb[i + 1] as u64) << 32);
        h ^= p;
        h = h.wrapping_mul(0x100000001b3);
        i += 2;
    }
    if i < fb.len() {
        h ^= fb[i] as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[test]
fn test_hash_properties_and_isolated_benchmark() {
    let len = 240 * 320; // 76,800 pixels
    let mut buffer: Vec<u32> = (0..len as u32).map(|i| i.wrapping_mul(0x9e3779b9)).collect();

    // 1. Self-consistency check
    let hash_new_1 = hash_framebuffer_new(&buffer);
    let hash_new_2 = hash_framebuffer_new(&buffer);
    assert_eq!(hash_new_1, hash_new_2, "Hash must be deterministic");

    // 2. Comparison between old and new
    let hash_old = hash_framebuffer_old(&buffer);
    println!("[AUDIT] Old Hash: 0x{:016x}", hash_old);
    println!("[AUDIT] New Hash: 0x{:016x}", hash_new_1);
    // They are NOT identical!
    assert_ne!(hash_old, hash_new_1, "Old and new produce different numerical hashes!");

    // 3. Odd buffer length test
    let mut odd_buffer = buffer.clone();
    odd_buffer.push(0xDEADBEEF);
    let hash_odd = hash_framebuffer_new(&odd_buffer);
    assert_ne!(hash_new_1, hash_odd, "Odd buffer trailing pixel must alter hash");

    // 4. Change sensitivity test
    buffer[76799] ^= 1; // 1-bit flip on last pixel
    let hash_new_flipped = hash_framebuffer_new(&buffer);
    assert_ne!(hash_new_1, hash_new_flipped, "1-bit flip must alter hash");

    // 5. Isolated performance benchmark (1,000 runs)
    let iterations = 1000;

    // Warm-up
    for _ in 0..50 {
        std::hint::black_box(hash_framebuffer_old(&buffer));
        std::hint::black_box(hash_framebuffer_new(&buffer));
    }

    let t0 = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(hash_framebuffer_old(&buffer));
    }
    let elapsed_old = t0.elapsed();

    let t1 = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(hash_framebuffer_new(&buffer));
    }
    let elapsed_new = t1.elapsed();

    let old_micros = elapsed_old.as_micros() as f64 / iterations as f64;
    let new_micros = elapsed_new.as_micros() as f64 / iterations as f64;
    let speedup = old_micros / new_micros;

    println!("[ISOLATED BENCHMARK]");
    println!("  Old hash per frame: {:.2} µs", old_micros);
    println!("  New hash per frame: {:.2} µs", new_micros);
    println!("  Speedup: {:.2}x ({:+.1}%)", speedup, ((old_micros - new_micros) / old_micros) * 100.0);
}
