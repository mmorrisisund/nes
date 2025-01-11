pub struct PpuBus {
    chr_rom: Vec<u8>,
    vram: [u8; 2048],
    oam_data: [u8; 256],
}
