use image::GenericImageView;

fn main() {
    let path = std::path::PathBuf::from(std::env::args().nth(1).expect("file path required"));
    let image = image::open(&path).expect("Failed to open font table image");

    let (char_w, char_h) = (8, 12);
    let h_packed_chars = image.width() / char_w;
    let font_bitmap_table = (0..256)
        .flat_map(|n| {
            let (xo, yo) = ((n % h_packed_chars) * char_w, (n / h_packed_chars) * char_h);
            let image = &image;

            (0..char_h).map(move |y| {
                (0..char_w)
                    .map(|x| ((image.get_pixel(xo + x, yo + y).0[0] == 0) as u32) << x)
                    .fold(0, |a, b| a | b) as u8
            })
        })
        .collect::<Vec<_>>();

    std::fs::write(path.parent().unwrap().join("font.bin"), font_bitmap_table).unwrap();
}
