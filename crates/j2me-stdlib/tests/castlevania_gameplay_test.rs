use j2me_stdlib::J2meEmulator;
use std::fs;
use std::io::Write;

pub fn sha256_digest(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64) * 8;
    msg.push(0x80);
    while (msg.len() + 8) % 64 != 0 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_var = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_var
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_var = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_var);
    }

    let mut out = String::with_capacity(64);
    for val in h {
        out.push_str(&format!("{:08x}", val));
    }
    out
}

fn save_ppm(path: &str, fb: &[u32], w: usize, h: usize) {
    let p = std::path::Path::new(path);
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut f = match fs::File::create(p) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Warning: failed to create PPM file at {}: {}", path, e);
            return;
        }
    };
    let header = format!("P6\n{} {}\n255\n", w, h);
    f.write_all(header.as_bytes()).unwrap();
    let mut bytes = Vec::with_capacity(w * h * 3);
    for &px in fb {
        let r = ((px >> 16) & 0xFF) as u8;
        let g = ((px >> 8) & 0xFF) as u8;
        let b = (px & 0xFF) as u8;
        bytes.push(r);
        bytes.push(g);
        bytes.push(b);
    }
    f.write_all(&bytes).unwrap();
}

fn boot_castlevania_to_gameplay() -> J2meEmulator {
    let jar_path = "/Users/foolvan/Desktop/j2me_market/roms/Games/CastlevaniaAoS.jar";
    let jar_bytes = fs::read(jar_path).expect("Castlevania JAR must exist");
    let mut emu = J2meEmulator::new(240, 320, 44100);
    emu.load_jar(&jar_bytes)
        .expect("Castlevania JAR should load");

    // 1. Language Selection (Tick 60): Press SoftLeft (-6) for English
    for _ in 0..60 {
        emu.tick(16.66667);
    }
    emu.key_event(-6, true);
    for _ in 0..15 {
        emu.tick(16.66667);
    }
    emu.key_event(-6, false);

    // 2. Sound Prompt (Tick 140): Press SoftLeft (-6) for Sound YES
    for _ in 0..65 {
        emu.tick(16.66667);
    }
    emu.key_event(-6, true);
    for _ in 0..15 {
        emu.tick(16.66667);
    }
    emu.key_event(-6, false);

    // 3. Konami & Code Monkeys Splashes -> Title Screen (Tick 520)
    for _ in 0..365 {
        emu.tick(16.66667);
    }

    // 4. Title Screen: Press FIRE (-5) to open Main Menu
    emu.key_event(-5, true);
    for _ in 0..20 {
        emu.tick(16.66667);
    }
    emu.key_event(-5, false);

    // 5. Main Menu (Tick 620): Press FIRE (-5) to select Start Game
    for _ in 0..80 {
        emu.tick(16.66667);
    }
    emu.key_event(-5, true);
    for _ in 0..20 {
        emu.tick(16.66667);
    }
    emu.key_event(-5, false);

    // 6. Submenu (Tick 680): Press FIRE (-5) to select New Game
    for _ in 0..40 {
        emu.tick(16.66667);
    }
    emu.key_event(-5, true);
    for _ in 0..20 {
        emu.tick(16.66667);
    }
    emu.key_event(-5, false);

    // 7. Prologue Story Screen (Tick 980): Press SoftRight (-7) to SKIP
    for _ in 0..260 {
        emu.tick(16.66667);
    }
    emu.key_event(-7, true);
    for _ in 0..20 {
        emu.tick(16.66667);
    }
    emu.key_event(-7, false);

    // 8. Wait for Castle Courtyard Level 1 to load (Tick 1030)
    for _ in 0..30 {
        emu.tick(16.66667);
    }

    emu
}

#[test]
#[ignore]
fn test_castlevania_drive_to_gameplay() {
    let jar_path = "/Users/foolvan/Desktop/j2me_market/roms/Games/CastlevaniaAoS.jar";
    if !std::path::Path::new(jar_path).exists() {
        eprintln!("Castlevania JAR not found, skipping");
        return;
    }

    println!("Booting Castlevania: Aria of Sorrow to Level 1...");
    let mut emu_control = boot_castlevania_to_gameplay();
    let raw_control = unsafe {
        std::slice::from_raw_parts(emu_control.framebuffer_ptr(), emu_control.framebuffer_len())
    };
    let hash_level1_control = sha256_digest(raw_control);
    println!(
        "Castlevania Level 1 Initial Gameplay Frame SHA256: {}",
        hash_level1_control
    );
    save_ppm(
        "experiments/castlevania_level1_control.ppm",
        &emu_control.context.framebuffer,
        240,
        320,
    );

    // Run control for 70 ticks with no action
    for _ in 0..70 {
        emu_control.tick(16.66667);
    }
    let raw_no_action = unsafe {
        std::slice::from_raw_parts(emu_control.framebuffer_ptr(), emu_control.framebuffer_len())
    };
    let hash_no_action = sha256_digest(raw_no_action);
    println!(
        "Castlevania Level 1 (No-Action after 70 ticks) SHA256: {}",
        hash_no_action
    );

    // Now boot a second instance and apply RIGHT (-4) action (running up castle stairs)
    let mut emu_action = boot_castlevania_to_gameplay();
    emu_action.key_event(-4, true);
    for _ in 0..70 {
        emu_action.tick(16.66667);
    }
    emu_action.key_event(-4, false);

    let raw_action = unsafe {
        std::slice::from_raw_parts(emu_action.framebuffer_ptr(), emu_action.framebuffer_len())
    };
    let hash_action = sha256_digest(raw_action);
    println!(
        "Castlevania Level 1 (Action RIGHT after 70 ticks) SHA256: {}",
        hash_action
    );
    save_ppm(
        "experiments/castlevania_level1_action.ppm",
        &emu_action.context.framebuffer,
        240,
        320,
    );

    // Verify difference
    assert_ne!(
        hash_no_action, hash_action,
        "Castlevania must react to RIGHT movement input on Level 1!"
    );
}
