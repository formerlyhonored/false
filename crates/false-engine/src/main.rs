use false_engine::Engine;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("============================================================");
    println!("   ✦ FALSE BROWSER ENGINE (NATIVE RUST CORE)");
    println!("============================================================");

    let args: Vec<String> = env::args().collect();
    let (html_content, source_name) = if args.len() > 1 {
        let path_or_html = &args[1];
        if Path::new(path_or_html).exists() {
            (
                fs::read_to_string(path_or_html).unwrap_or_default(),
                path_or_html.clone(),
            )
        } else {
            (path_or_html.clone(), "inline argument".to_string())
        }
    } else {
        let sample = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {
                        background-color: #16171d;
                        color: #f3f4f6;
                        padding: 24px;
                    }
                    .hero {
                        background-color: #20222a;
                        border-width: 2px;
                        border-color: #333744;
                        padding: 20px;
                        margin-bottom: 20px;
                    }
                    h1 {
                        color: #e06c75;
                        font-size: 24px;
                        margin-bottom: 12px;
                    }
                    p {
                        color: #9ca3af;
                        font-size: 14px;
                    }
                    .badge {
                        background-color: #528bff;
                        color: #ffffff;
                        padding: 6px 12px;
                    }
                    .card {
                        background-color: #282c34;
                        border-width: 1px;
                        border-color: #4b5263;
                        padding: 16px;
                    }
                </style>
            </head>
            <body>
                <div class="hero">
                    <h1>False Native Browser Engine</h1>
                    <p>Rendered from scratch using pure Rust layout, DOM, CSS cascade and software rasterizer.</p>
                </div>
                <div class="card">
                    <p>Engine status: 100% Native Rust Pipeline Active</p>
                    <span class="badge">Blazingly Fast</span>
                </div>
            </body>
            </html>
        "#;
        (sample.to_string(), "Built-in False Engine Demo".to_string())
    };

    println!("Rendering: {}", source_name);
    let width = 800;
    let height = 600;

    let start = std::time::Instant::now();
    let result = Engine::render(&html_content, width, height);
    let elapsed = start.elapsed();

    println!("\n[1] DOM Tree Hierarchy:");
    println!("{}", result.dom_tree);

    println!(
        "[2] Display List Commands: {} items",
        result.display_list.len()
    );
    for (i, cmd) in result.display_list.iter().take(8).enumerate() {
        println!("  #{} {:?}", i + 1, cmd);
    }
    if result.display_list.len() > 8 {
        println!("  ... and {} more commands", result.display_list.len() - 8);
    }

    // Save rendered frame to BMP
    let out_bmp_path = "false_rendered_frame.bmp";
    let bmp_bytes = result.to_bmp();
    fs::write(out_bmp_path, &bmp_bytes).expect("Failed to write output BMP");

    println!("\n[3] Rasterization Performance:");
    println!("  Viewport: {}x{} pixels", width, height);
    println!("  Frame rendered in: {:?}", elapsed);
    println!("  Output saved to: {}", out_bmp_path);
    println!("  BMP file size: {} KB", bmp_bytes.len() / 1024);
    println!("\n✔ Native Browser Engine pipeline executed successfully!");
}
