const VIDEO_RAM_SIZE: usize = 0x1FFF;
const VIEWPORT_PIXELS: usize = 256*256;

#[derive(Clone,Copy)]
pub struct Tile {
    data: [u8; 8*8],
}

impl Tile {
    pub fn new() -> Tile {
        Tile {
            data: [0; 8*8],
        }
    }
}

pub struct GPU {
    pub tile_set: [Tile; 384],
    pub video_ram: [u8; VIDEO_RAM_SIZE],
    pub canvas_buffer: [u8; VIEWPORT_PIXELS],
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            tile_set: [Tile::new(); 384],
            video_ram: [0; VIDEO_RAM_SIZE],
            canvas_buffer: [0; VIEWPORT_PIXELS],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.video_ram[address as usize - 0x8000],
            _ => panic!("bad address"),
        }
    }

    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0x8000..=0x9FFF => self.video_ram[address as usize - 0x8000] = val,
            _ => panic!("bad address"),
        }
    }

}
