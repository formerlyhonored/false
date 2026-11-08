// Prevent console window popup on Windows
#![windows_subsystem = "windows"]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let exe_stem = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    let is_setup = args
        .iter()
        .any(|a| a == "--installer" || a == "--setup" || a == "--uninstaller")
        || exe_stem.contains("setup")
        || exe_stem.contains("installer")
        || exe_stem.contains("uninstall");

    if is_setup {
        let temp_profile =
            std::env::temp_dir().join(format!("false_setup_wv2_{}", std::process::id()));
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", temp_profile);
    }

    // Hardware acceleration and render engine optimizations for Chromium / WebView2
    let existing_args = std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").unwrap_or_default();
    let performance_flags = "--enable-gpu-rasterization \
                             --enable-zero-copy \
                             --ignore-gpu-blocklist \
                             --enable-features=CanvasOopRasterization,ParallelDownloading \
                             --disable-background-timer-throttling \
                             --disable-backgrounding-occluded-windows \
                             --disable-renderer-backgrounding \
                             --enable-accelerated-2d-canvas \
                             --enable-accelerated-video-decode \
                             --num-raster-threads=4";
    if !existing_args.contains("enable-gpu-rasterization") {
        let combined = if existing_args.is_empty() {
            performance_flags.to_string()
        } else {
            format!("{} {}", existing_args, performance_flags)
        };
        std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", combined);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if args[1] == "--export-flse" {
            let version = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
            let out = args
                .get(3)
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(format!("false-v{}.flse", version)));
            match false_ui_lib::flse::create_package(&version, "False browser update", false, &out)
            {
                Ok(_) => println!("Successfully created {}", out.display()),
                Err(e) => eprintln!("Error: {}", e),
            }
            return;
        }

        // A .flse passed on the command line is only previewed. The UI asks for
        // consent (and re-verifies the signature) before applying anything.
        for arg in &args[1..] {
            let p = std::path::Path::new(arg);
            if p.exists() && (arg.ends_with(".flse") || arg.ends_with(".zip")) {
                false_ui_lib::set_pending_package(p.to_path_buf());
                break;
            }
        }
    }

    false_ui_lib::run();
}
