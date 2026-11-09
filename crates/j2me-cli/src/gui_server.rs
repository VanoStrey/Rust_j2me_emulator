//! Embedded Web GUI Runner for Rust J2ME Emulator.
//!
//! Provides a zero-dependency, local HTTP server serving the retro phone web player
//! (`web/index.html` + `web/j2me_wasm.wasm`), automatically binding to an available port,
//! serving the specified JAR / JAD ROM, and launching the user's default browser.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub struct GuiServerOptions {
    pub jar_path: PathBuf,
    pub jad_path: Option<PathBuf>,
    pub port: u16,
    pub open_browser: bool,
}

/// Find the `web/` assets directory.
fn locate_web_dir() -> Result<PathBuf, String> {
    let candidates = [
        PathBuf::from("web"),
        PathBuf::from("./web"),
        PathBuf::from("../web"),
        PathBuf::from("/Users/foolvan/Desktop/Rust_j2me_emulator/web"),
    ];

    for candidate in &candidates {
        if candidate.join("index.html").is_file() {
            return Ok(candidate.clone());
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let p1 = parent.join("web");
            if p1.join("index.html").is_file() {
                return Ok(p1);
            }
            if let Some(grandparent) = parent.parent() {
                let p2 = grandparent.join("web");
                if p2.join("index.html").is_file() {
                    return Ok(p2);
                }
                if let Some(great) = grandparent.parent() {
                    let p3 = great.join("web");
                    if p3.join("index.html").is_file() {
                        return Ok(p3);
                    }
                }
            }
        }
    }

    Err("Could not locate 'web/index.html' assets directory.".to_string())
}

fn get_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "jar" => "application/java-archive",
        "jad" | "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn handle_connection(
    mut stream: TcpStream,
    web_dir: &Path,
    jar_path: &Path,
    jad_path: Option<&Path>,
) {
    let mut buffer = [0u8; 4096];
    let n = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req_str = String::from_utf8_lossy(&buffer[..n]);
    let first_line = req_str.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    if method != "GET" && method != "HEAD" {
        let resp = "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    let req_path = path.split('?').next().unwrap_or("/");

    // 1. Specific API endpoints for active ROM
    if req_path == "/api/game.jar" {
        serve_file(&mut stream, jar_path, "application/java-archive", method == "HEAD");
        return;
    }

    if req_path == "/api/game.jad" {
        if let Some(jad) = jad_path {
            if jad.is_file() {
                serve_file(&mut stream, jad, "text/plain; charset=utf-8", method == "HEAD");
                return;
            }
        }
        send_404(&mut stream);
        return;
    }

    // 2. Static web files
    let rel_path = if req_path == "/" || req_path == "/index.html" {
        "index.html"
    } else {
        req_path.trim_start_matches('/')
    };

    let target_file = web_dir.join(rel_path);

    if target_file.is_file() {
        let mime = get_mime_type(&target_file);
        serve_file(&mut stream, &target_file, mime, method == "HEAD");
    } else {
        send_404(&mut stream);
    }
}

fn serve_file(stream: &mut TcpStream, file_path: &Path, mime: &str, head_only: bool) {
    match fs::read(file_path) {
        Ok(bytes) => {
            let header = format!(
                "HTTP/1.1 200 OK\r\n\
                 Content-Type: {}\r\n\
                 Content-Length: {}\r\n\
                 Access-Control-Allow-Origin: *\r\n\
                 Cross-Origin-Opener-Policy: same-origin\r\n\
                 Cross-Origin-Embedder-Policy: require-corp\r\n\
                 Cache-Control: no-cache\r\n\
                 Connection: close\r\n\r\n",
                mime,
                bytes.len()
            );

            let _ = stream.write_all(header.as_bytes());
            if !head_only {
                let _ = stream.write_all(&bytes);
            }
        }
        Err(_) => send_404(stream),
    }
}

fn send_404(stream: &mut TcpStream) {
    let resp = "HTTP/1.1 404 Not Found\r\n\
                Content-Type: text/plain; charset=utf-8\r\n\
                Content-Length: 13\r\n\
                Connection: close\r\n\r\n\
                404 Not Found";
    let _ = stream.write_all(resp.as_bytes());
}

/// Launch the interactive GUI Retro Phone Server.
pub fn launch_gui(options: GuiServerOptions) -> Result<(), String> {
    if !options.jar_path.is_file() {
        return Err(format!("JAR file not found: {:?}", options.jar_path));
    }

    let web_dir = locate_web_dir()?;

    // Bind to available port
    let mut port = options.port;
    let mut listener: Option<TcpListener> = None;
    for offset in 0..15 {
        let try_port = port + offset;
        if let Ok(l) = TcpListener::bind(("127.0.0.1", try_port)) {
            port = try_port;
            listener = Some(l);
            break;
        }
    }

    let listener = match listener {
        Some(l) => l,
        None => return Err(format!("Failed to bind to 127.0.0.1:{}..{}", options.port, options.port + 15)),
    };

    let game_name = options
        .jar_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("game");

    let url = format!("http://localhost:{}/?game=/api/game.jar", port);

    println!("========================================================================");
    println!(" 📱 [RUST J2ME RETRO PHONE GUI] Interactive Handset Emulator Running!");
    println!("========================================================================");
    println!("  🎮 Game:   {}", game_name);
    println!("  📦 JAR:    {:?}", options.jar_path);
    if let Some(ref jad) = options.jad_path {
        println!("  📄 JAD:    {:?}", jad);
    }
    println!("  🌐 URL:    {}", url);
    println!();
    println!("  ⌨️  DESKTOP CONTROLS:");
    println!("    • D-Pad / Directional: Arrow Keys (Up, Down, Left, Right) or [W, A, S, D]");
    println!("    • Fire / Action / OK:  [Enter], [Space], [F] or [5]");
    println!("    • Left SoftKey:        [Q], [F1], or on-screen [Left Soft] button");
    println!("    • Right SoftKey:       [E], [F2], or on-screen [Right Soft] button");
    println!("    • Numeric Keypad:      Number keys [0]-[9] or on-screen keypad");
    println!("    • Symbols:             [*] / [P], [#] / [H]");
    println!("    • Mouse / Touch:       Click any button directly on the retro phone chassis!");
    println!();
    println!("  ⚡ [Engine] Zero-Copy WebAssembly Core running at authentic 60 Hz");
    println!("  🔊 [Audio]  Live WebAudio synthesis for PCM waveforms and sound effects");
    println!("========================================================================");
    println!("  🚀 Opening phone GUI in your browser...");
    println!("  (Press Ctrl+C in this terminal when finished to stop the server)");
    println!("========================================================================");

    if options.open_browser {
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&url).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(&["/C", "start", &url]).spawn();
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Spawn server accept loop
    let web_dir_arc = Arc::new(web_dir);
    let jar_path_arc = Arc::new(options.jar_path);
    let jad_path_arc = Arc::new(options.jad_path);

    while r.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let w = Arc::clone(&web_dir_arc);
                let j = Arc::clone(&jar_path_arc);
                let d = Arc::clone(&jad_path_arc);
                thread::spawn(move || {
                    handle_connection(stream, &w, &j, d.as_deref());
                });
            }
            Err(e) => {
                eprintln!("[WARN] TCP accept error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
