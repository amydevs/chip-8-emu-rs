pub fn render_texture_to_target(dispmem: &[u8; 2048], frame: &mut [u8], fg: &crate::options::RGB, bg: &crate::options::RGB) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        if dispmem[i] == 1 {
            pixel.copy_from_slice(&[fg.r, fg.g, fg.b, 0xff]);
        }
        else {
            pixel.copy_from_slice(&[bg.r, bg.g, bg.b, 0xff]);
        }
    }
}