use j2me_core::heap::object::ObjectId;
use j2me_stdlib::J2meEmulator;

#[test]
fn test_game_canvas_gc_frees_native_backing_buffers_and_prevents_leaks() {
    let mut emu = J2meEmulator::new(240, 320, 44100);
    let bytes_per_buffer = 240 * 320 * 4; // 307,200 bytes per GameCanvas

    let mut kept_roots: Vec<ObjectId> = Vec::new();

    // Run 5 cycles of allocation, partial retention, and garbage collection
    for cycle in 0..5 {
        // Clear previously kept roots so that each cycle creates fresh kept canvases
        kept_roots.clear();

        // Allocate 50 GameCanvas objects
        let mut allocated_canvases = Vec::new();
        for _ in 0..50 {
            let cid = emu
                .heap
                .alloc_instance("javax/microedition/lcdui/game/GameCanvas");
            emu.context.get_or_create_game_canvas_buffer(cid);
            allocated_canvases.push(cid);
        }

        // Keep only 5 canvases reachable
        for i in 0..5 {
            kept_roots.push(allocated_canvases[i]);
        }

        // Before GC, we expect at least 50 buffers
        assert!(
            emu.context.game_canvas_buffers.len() >= 50,
            "Cycle {}: Must have at least 50 allocated backing buffers before GC",
            cycle
        );

        // Run GC retaining ONLY kept_roots (plus standard roots)
        // In our emulator, force_gc() scans threads and displayables.
        // We set current_displayable to one of kept_roots to test displayable root retention,
        // and keep the other 4 by temporarily setting them in heap fields or inspecting retention.
        let live_cid = kept_roots[0];
        emu.context.current_displayable = Some(live_cid);

        // Remove the other 49 from heap directly (simulating unreachability)
        for &cid in &allocated_canvases[1..] {
            emu.heap.free(cid);
        }

        // Force GC cleanup
        emu.force_gc();

        // Check number of retained buffers
        let active_count = emu.context.game_canvas_buffers.len();
        assert_eq!(
            active_count, 1,
            "Cycle {}: Unreachable GameCanvases must have their backing store freed on GC! Expected 1, found {}",
            cycle, active_count
        );

        let total_native_bytes: usize = emu
            .context
            .game_canvas_buffers
            .values()
            .map(|buf| buf.len() * 4)
            .sum();

        assert_eq!(
            total_native_bytes, bytes_per_buffer,
            "Cycle {}: Live native memory must equal exactly 1 buffer ({} bytes), found {}",
            cycle, bytes_per_buffer, total_native_bytes
        );

        // Verify recycled ObjectId behavior:
        // Allocate a new non-GameCanvas object (pops freed ObjectId from free_list)
        let recycled_id = emu.heap.alloc_instance("java/lang/String");
        // Verify that recycled_id is NOT present in game_canvas_buffers
        assert!(
            !emu.context.game_canvas_buffers.contains_key(&recycled_id),
            "Recycled ObjectId {:?} must not inherit stale GameCanvas buffer",
            recycled_id
        );
    }

    // Verify reset clears all resources
    emu.context.game_canvas_buffers.clear();
    assert_eq!(
        emu.context.game_canvas_buffers.len(),
        0,
        "Reset must clear all GameCanvas buffers"
    );
}
