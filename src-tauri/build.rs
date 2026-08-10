fn main() {
    ensure_windows_icon();
    tauri_build::build()
}

fn ensure_windows_icon() {
    use std::{fs, path::Path};

    let path = Path::new("icons/icon.ico");
    if path.exists() {
        return;
    }
    fs::create_dir_all(path.parent().expect("icon directory")).expect("create icon directory");

    const SIZE: usize = 32;
    const PIXEL_BYTES: usize = SIZE * SIZE * 4;
    const MASK_BYTES: usize = SIZE * 4;
    let image_bytes = 40 + PIXEL_BYTES + MASK_BYTES;
    let mut icon = Vec::with_capacity(22 + image_bytes);

    // ICONDIR and ICONDIRENTRY.
    icon.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
    icon.extend_from_slice(&[SIZE as u8, SIZE as u8, 0, 0]);
    icon.extend_from_slice(&1u16.to_le_bytes());
    icon.extend_from_slice(&32u16.to_le_bytes());
    icon.extend_from_slice(&(image_bytes as u32).to_le_bytes());
    icon.extend_from_slice(&22u32.to_le_bytes());

    // BITMAPINFOHEADER. ICO stores XOR + AND masks, hence doubled height.
    icon.extend_from_slice(&40u32.to_le_bytes());
    icon.extend_from_slice(&(SIZE as i32).to_le_bytes());
    icon.extend_from_slice(&((SIZE * 2) as i32).to_le_bytes());
    icon.extend_from_slice(&1u16.to_le_bytes());
    icon.extend_from_slice(&32u16.to_le_bytes());
    icon.extend_from_slice(&0u32.to_le_bytes());
    icon.extend_from_slice(&(PIXEL_BYTES as u32).to_le_bytes());
    icon.extend_from_slice(&0i32.to_le_bytes());
    icon.extend_from_slice(&0i32.to_le_bytes());
    icon.extend_from_slice(&0u32.to_le_bytes());
    icon.extend_from_slice(&0u32.to_le_bytes());

    for stored_y in 0..SIZE {
        let y = SIZE - 1 - stored_y;
        for x in 0..SIZE {
            let left = 6 + (y.saturating_sub(5) * 10 / 21);
            let right = 26usize.saturating_sub(y.saturating_sub(5) * 10 / 21);
            let on_vertex =
                (5..=27).contains(&y) && (x.abs_diff(left) <= 1 || x.abs_diff(right) <= 1);
            let (r, g, b) = if on_vertex {
                (42u8, 168u8, 255u8)
            } else if x == 1 || x == SIZE - 2 || y == 1 || y == SIZE - 2 {
                (18u8, 73u8, 111u8)
            } else {
                (8u8, 15u8, 23u8)
            };
            icon.extend_from_slice(&[b, g, r, 255]);
        }
    }
    icon.resize(22 + image_bytes, 0);
    fs::write(path, icon).expect("write generated Windows icon");
}
