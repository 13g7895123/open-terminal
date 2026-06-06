#[path = "src/branding_core.rs"]
mod branding_core;

use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/branding_core.rs");

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR not set"));
    let sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let mut pngs = Vec::new();

    for size in sizes {
        let path = out_dir.join(format!("app-icon-{size}.png"));
        write_png(&path, size, &branding_core::render_icon_rgba(size));
        pngs.push(path);
    }

    write_ico(&out_dir.join("app-icon.ico"), &pngs);

    #[cfg(target_os = "windows")]
    {
        let icon_path = out_dir.join("app-icon.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon(icon_path.to_str().expect("icon path must be valid UTF-8"));
        res.compile().expect("failed to compile Windows resources");
    }
}

fn write_png(path: &PathBuf, size: u32, rgba: &[u8]) {
    let file = fs::File::create(path).expect("failed to create png");
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, size, size);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().expect("failed to write png header");
    png_writer
        .write_image_data(rgba)
        .expect("failed to write png data");
}

fn write_ico(path: &PathBuf, pngs: &[PathBuf]) {
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for png_path in pngs {
        let file = fs::File::open(png_path).expect("failed to open generated png");
        let image = ico::IconImage::read_png(file).expect("failed to decode generated png");
        let entry = ico::IconDirEntry::encode_as_png(&image).expect("failed to encode ico entry");
        icon_dir.add_entry(entry);
    }
    let file = fs::File::create(path).expect("failed to create ico");
    icon_dir.write(file).expect("failed to write ico");
}
