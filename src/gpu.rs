const VIDEO_RAM_SIZE: usize = 0x1FFF;
const VIEWPORT_PIXELS: usize = 256*256;
const PIXELS: usize = 160*144;

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
    pub pixel_buffer: [u8; PIXELS],

    pub mode_clock: u32,
    pub mode: GPUMode,
    pub line: u16,
}

pub enum GPUMode {
    HBlank,
    VBlank,
    OAMRead,
    VRAMRead,
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            tile_set: [Tile::new(); 384],
            video_ram: [0; VIDEO_RAM_SIZE],
            canvas_buffer: [0; VIEWPORT_PIXELS],
            pixel_buffer: [255; PIXELS],

            mode_clock: 0,
            mode: GPUMode::OAMRead,
            line: 0,
        }
    }

    pub fn run(&mut self, cycles: u32) {
        self.mode_clock += cycles;
        println!("mode_clock = {}", self.mode_clock);
        println!("line = {}", self.line);

        match self.mode {
            GPUMode::OAMRead => {
                println!("[GPU] entering OAM read");
                if self.mode_clock >= 80 {
                    self.mode_clock = 0;
                    self.mode = GPUMode::VRAMRead;
                }
            },
            GPUMode::VRAMRead => {
                println!("[GPU] entering VRAM read");
                if self.mode_clock >= 172 {
                    self.mode_clock = 0;
                    self.mode = GPUMode::HBlank;

                    self.render_scan();
                }
            },
            GPUMode::HBlank => {
                println!("[GPU] entering HBlank");
                if self.mode_clock >= 204 {
                    self.mode_clock = 0;
                    self.line += 1;

                    if self.line == 143 {
                        self.mode = GPUMode::VBlank;
                        self.write_pixels();
                    } else {
                        self.mode = GPUMode::OAMRead;
                    }
                }
            },
            GPUMode::VBlank => {
                println!("[GPU] entering VBlank");
                if self.mode_clock >= 456 {
                    self.mode_clock = 0;
                    self.line += 1;

                    if self.line > 153 {
                        self.mode = GPUMode::OAMRead;
                        self.line = 0;
                    }
                }
            }
        }
    }

    fn render_scan(&self) { }
    fn write_pixels(&self) { }

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
