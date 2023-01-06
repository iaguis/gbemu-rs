const VIDEO_RAM_SIZE: usize = 0x1FFF;
const VIEWPORT_PIXELS: usize = 160*144;

#[derive(Clone,Copy)]
pub struct Tile {
    data: [[u8; 8];8],
}

impl Tile {
    pub fn new() -> Tile {
        Tile {
            data: [[0; 8];8],
        }
    }
}

pub struct GPU {
    pub tile_set: [Tile; 384],
    pub video_ram: [u8; VIDEO_RAM_SIZE + 1],
    pub canvas_buffer: [u32; VIEWPORT_PIXELS + 1],

    pub mode_clock: u32,
    pub mode: GPUMode,

    scy: u8,
    scx: u8,
    ly: u8,
    lcdc: LCDC,
    bg_palette: BackgroundPalette,
}

#[derive(Clone,Copy)]
pub struct LCDC {
    lcd_enable: bool,
    window_tilemap: bool,
    window_enable: bool,
    bg_window_addressing_mode: bool,
    bg_tilemap: bool,
    obj_size: bool,
    obj_enable: bool,
    bg_window_priority: bool,
}

#[derive(Clone,Copy)]
pub enum Color {
    White = 255,
    LightGray = 192,
    DarkGray = 96,
    Black = 0,
}

impl Color {
    pub fn to_rgb(&self) -> u32 {
        match self {
            Color::White => 0xffffffff,
            Color::LightGray => 0xffc0c0c0,
            Color::DarkGray => 0xff606060,
            Color::Black => 0xff000000,
        }
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0 => Color::White,
            1 => Color::LightGray,
            2 => Color::DarkGray,
            3 => Color::Black,
            _ => panic!("Cannot convert {} to color", value),
        }
    }
}

impl From<Color> for u8 {
    fn from(value: Color) -> u8 {
        match value {
            Color::White => 0,
            Color::LightGray => 1,
            Color::DarkGray => 2,
            Color::Black => 3,
        }
    }
}

#[derive(Clone,Copy)]
pub struct BackgroundPalette(Color, Color, Color, Color);

impl BackgroundPalette {
    fn new() -> BackgroundPalette {
        BackgroundPalette(
            Color::White,
            Color::LightGray,
            Color::DarkGray,
            Color::Black,
        )
    }
}

impl From<u8> for BackgroundPalette {
     fn from(value: u8) -> Self {
        BackgroundPalette(
            (value & 0b11).into(),
            ((value >> 2) & 0b11).into(),
            ((value >> 4) & 0b11).into(),
            (value >> 6).into(),
        )
     }
}

impl From<BackgroundPalette> for u8 {
     fn from(value: BackgroundPalette) -> u8 {
         value.3.into()
     }
}


impl From<u8> for LCDC {
    fn from(value: u8) -> Self {
        LCDC {
            lcd_enable: (value & (1 << 7)) != 0,
            window_tilemap: (value & (1 << 6)) != 0,
            window_enable: (value & (1 << 5)) != 0,
            bg_window_addressing_mode: (value & (1 << 4)) != 0,
            bg_tilemap: (value & (1 << 3)) != 0,
            obj_size: (value & (1 << 2)) != 0,
            obj_enable: (value & (1 << 1)) != 0,
            bg_window_priority: (value & 1) != 0,
        }
    }
}

impl From<LCDC> for u8 {
    fn from(value: LCDC) -> u8 {
        (if value.lcd_enable {1} else {0} << 7) |
        (if value.window_tilemap {1} else {0} << 6) |
        (if value.window_enable {1} else {0} << 5) |
        (if value.bg_window_addressing_mode {1} else {0} << 4) |
        (if value.bg_tilemap {1} else {0} << 3) |
        (if value.obj_size {1} else {0} << 2) |
        (if value.obj_enable {1} else {0} << 1) |
        (if value.bg_window_priority {1} else {0})
    }
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
            video_ram: [0; VIDEO_RAM_SIZE+1],
            canvas_buffer: [0; VIEWPORT_PIXELS+1],

            mode_clock: 0,
            mode: GPUMode::OAMRead,

            scx: 0,
            scy: 0,
            ly: 0,
            lcdc: LCDC{
                lcd_enable: false,
                window_tilemap: false,
                window_enable: false,
                bg_window_addressing_mode: false,
                bg_tilemap: false,
                obj_size: false,
                obj_enable: false,
                bg_window_priority: false,
            },
            bg_palette: BackgroundPalette::new(),
        }
    }

    pub fn run(&mut self, cycles: u32) {
        self.mode_clock += cycles;

        match self.mode {
            GPUMode::OAMRead => {
                if self.mode_clock >= 80 {
                    self.mode_clock = 0;
                    self.mode = GPUMode::VRAMRead;
                }
            },
            GPUMode::VRAMRead => {
                if self.mode_clock >= 172 {
                    self.mode_clock = 0;
                    self.mode = GPUMode::HBlank;

                    self.render_scan();
                }
            },
            GPUMode::HBlank => {
                if self.mode_clock >= 204 {
                    self.mode_clock = 0;
                    self.ly += 1;

                    if self.ly == 143 {
                        self.mode = GPUMode::VBlank;
                        self.write_pixels();
                    } else {
                        self.mode = GPUMode::OAMRead;
                    }
                }
            },
            GPUMode::VBlank => {
                if self.mode_clock >= 456 {
                    self.mode_clock = 0;
                    self.ly += 1;

                    if self.ly > 153 {
                        self.mode = GPUMode::OAMRead;
                        self.ly = 0;
                    }
                }
            }
        }
    }

    // GB has a weird way to store pixels. Each row has 2 bytes, and to get the tile pixel color
    // (2 bits), the msb is from the second byte and the lsb is from the first byte.
    //
    // address: 0x8000 - 0x9FFF
    fn update_tile(&mut self, address: usize) {
        // make sure address is the first byte of each row
        let address = address & 0x1FFE;
        // address / 16 = tile index
        let tile_idx = address >> 4;

        // not a tile set address
        if tile_idx >= self.tile_set.len() {
            return
        }

        let row_idx = address >> 1 & 0x7;

        for col_idx in 0..8 {
            let bit_index = 1 << (7 - col_idx);

            let msb = if (self.video_ram[address+1] & bit_index) != 0 {1} else {0};
            let lsb = if (self.video_ram[address] & bit_index) != 0 {1} else {0};

            self.tile_set[tile_idx].data[row_idx][col_idx] = (msb << 1) | lsb
        }
    }

    fn get_color(&self, val: u8) -> Color {
        match val {
            0 => self.bg_palette.0,
            1 => self.bg_palette.1,
            2 => self.bg_palette.2,
            3 => self.bg_palette.3,
            _ => panic!("Cannot convert {} to color", val),
        }
    }

    fn render_scan(&mut self) {
        let base_address: u16 = if !self.lcdc.bg_tilemap { 0x1C00 } else { 0x1800 };
        let visible_offset = base_address + (((self.scy + self.ly) & 0xFF) as u16) >> 3;
        let mut line_offset = (self.scx as u16) >> 3;

        let mut x = self.scx & 7;
        let y = (self.ly + self.scy) & 7;

        let mut canvas_offset : usize = (self.ly as u16 * 160).into();

        let mut tile = self.video_ram[(visible_offset + line_offset) as usize] as u16;

        if !self.lcdc.bg_tilemap && tile < 128 {
            tile += 256;
        }

        for _ in 0..160 {
            let color = self.get_color(self.tile_set[tile as usize].data[y as usize][x as usize]);

            self.canvas_buffer[canvas_offset] = color.to_rgb();
            canvas_offset += 1;

            x += 1;
            if x == 8 {
                x = 0;
                line_offset = (line_offset + 1) & 31;
                tile = self.video_ram[(visible_offset + line_offset) as usize] as u16;

                if !self.lcdc.bg_tilemap && tile < 128 {
                    tile += 256;
                }
            }
        }
    }

    fn write_pixels(&self) { }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.video_ram[address as usize - 0x8000],
            0xFF40 => self.lcdc.into(),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF47 => self.bg_palette.into(),
            _ => panic!("bad address"),
        }
    }

    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0x8000..=0x9FFF => {
                self.video_ram[address as usize - 0x8000] = val;
                self.update_tile(address as usize);
            },
            0xFF40 => {
                self.lcdc = LCDC::from(val);
            },
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => self.ly = val,
            0xFF47 => {
                self.bg_palette = BackgroundPalette::from(val);
            },
            _ => panic!("bad address"),
        }
    }
}
