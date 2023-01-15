const VIDEO_RAM_SIZE: usize = 0x1FFF;
const OAM_SIZE: usize = 0x9F;
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

#[derive(Debug,Clone,Copy)]
pub struct Obj {
    y: i16,
    x: i16,
    tile: u8,
    attributes: ObjAttributes,
}

#[derive(Debug,Clone,Copy)]
pub struct ObjAttributes {
    bg_win_over_obj: bool,
    y_flip: bool,
    x_flip: bool,
    palette: bool,

    // GBC fields
    // tile_vram_bank: bool,
    // palette_number: 2*bool,
}

impl Obj {
    pub fn new() -> Obj {
        Obj {
            y: 0,
            x: 0,
            tile: 0,
            attributes: ObjAttributes{
                bg_win_over_obj: false,
                y_flip: false,
                x_flip: false,
                palette: false,
            },
        }
    }
}

pub struct GPU {
    pub tile_set: [Tile; 384],
    pub video_ram: [u8; VIDEO_RAM_SIZE + 1],
    pub oam: [u8; OAM_SIZE + 1],
    pub obj_set: [Obj; 40],
    pub canvas_buffer: [u32; VIEWPORT_PIXELS + 1],

    pub mode_clock: u32,

    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    lcdc: LCDC,
    bg_palette: Palette,
    obj0_palette: Palette,
    obj1_palette: Palette,

    lcd_status: LCDStatus,
}

#[derive(Debug,Clone,Copy)]
pub struct LCDStatus {
    lyc_equals_ly_int: bool,
    oam_int: bool,
    vblank_int: bool,
    hblank_int: bool,

    lyc_equals_ly: bool,
    mode: GPUMode,
}

impl From<u8> for LCDStatus {
    // Sets lycEqualsLy and mode to 0 since they're read-only
    fn from(value: u8) -> LCDStatus {
        LCDStatus {
            lyc_equals_ly_int: (value >> 6) & 0x01 == 1,
            oam_int: (value >> 5) & 0x01 == 1,
            vblank_int: (value >> 4) & 0x01 == 1,
            hblank_int: (value >> 3) & 0x01 == 1,
            lyc_equals_ly: false,
            mode: GPUMode::HBlank,
        }
    }
}

impl From<LCDStatus> for u8 {
    fn from(value: LCDStatus) -> u8 {
        let ret = (if value.lyc_equals_ly_int {1} else {0} << 6) |
        (if value.oam_int {1} else {0} << 5) |
        (if value.vblank_int {1} else {0} << 4) |
        (if value.hblank_int {1} else {0} << 3) |
        (if value.lyc_equals_ly {1} else {0} << 2);

        match value.mode {
            GPUMode::HBlank => { ret },
            GPUMode::VBlank => { ret | 0x1 },
            GPUMode::OAMRead => { ret | 0x2 },
            GPUMode::VRAMRead => { ret | 0x3 },
        }
    }
}

#[derive(Clone,Copy,PartialEq)]
pub enum GPUInterrupts {
    None,
    VBlank,
    LCDStat,
    Both,
}

impl GPUInterrupts {
    pub fn add(&mut self, new_request: GPUInterrupts) {
        match self {
            GPUInterrupts::None => *self = new_request,
            GPUInterrupts::VBlank if new_request == GPUInterrupts::LCDStat => {
                *self = GPUInterrupts::Both
            },
            GPUInterrupts::LCDStat if new_request == GPUInterrupts::VBlank => {
                *self = GPUInterrupts::Both
            },
            _ => {},
        }
    }
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

#[derive(PartialEq,Debug,Clone,Copy)]
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
pub struct Palette(Color, Color, Color, Color);

impl Palette {
    fn new() -> Palette {
        Palette(
            Color::White,
            Color::LightGray,
            Color::DarkGray,
            Color::Black,
        )
    }
}

impl From<u8> for Palette {
     fn from(value: u8) -> Self {
        Palette(
            (value & 0b11).into(),
            ((value >> 2) & 0b11).into(),
            ((value >> 4) & 0b11).into(),
            (value >> 6).into(),
        )
     }
}

impl From<Palette> for u8 {
    // TODO fix this ugliness
    fn from(value: Palette) -> u8 {
        let v3 = match value.3 {
            Color::White => 0,
            Color::LightGray => 1,
            Color::DarkGray => 2,
            Color::Black => 3,
        };

        let v2 = match value.2 {
            Color::White => 0,
            Color::LightGray => 1,
            Color::DarkGray => 2,
            Color::Black => 3,
        };

        let v1 = match value.1 {
            Color::White => 0,
            Color::LightGray => 1,
            Color::DarkGray => 2,
            Color::Black => 3,
        };

        let v0 = match value.0 {
            Color::White => 0,
            Color::LightGray => 1,
            Color::DarkGray => 2,
            Color::Black => 3,
        };

        (v3 << 6) |
        (v2 << 4) |
        (v1 << 2) |
        v0
    }
}

#[derive(Debug,Clone,Copy)]
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
            obj_set: [Obj::new(); 40],
            video_ram: [0; VIDEO_RAM_SIZE+1],
            oam: [0; OAM_SIZE+1],
            canvas_buffer: [0; VIEWPORT_PIXELS+1],

            mode_clock: 0,

            scx: 0,
            scy: 0,
            ly: 0,
            lyc: 0,
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
            lcd_status: LCDStatus{
                lyc_equals_ly_int: false,
                oam_int: false,
                vblank_int: false,
                hblank_int: false,

                lyc_equals_ly: false,
                mode: GPUMode::HBlank,
            },
            bg_palette: Palette::new(),
            obj0_palette: Palette::new(),
            obj1_palette: Palette::new(),
        }
    }

    pub fn step(&mut self, cycles: u32) -> GPUInterrupts {
        let mut interrupts_requested = GPUInterrupts::None;
        if !self.lcdc.lcd_enable {
            return interrupts_requested;
        }

        self.mode_clock += cycles;

        match self.lcd_status.mode {
            GPUMode::OAMRead => {
                if self.mode_clock >= 80 {
                    self.mode_clock = 0;
                    self.lcd_status.mode = GPUMode::VRAMRead;
                }
            },
            GPUMode::VRAMRead => {
                if self.mode_clock >= 172 {
                    self.mode_clock = 0;
                    self.lcd_status.mode = GPUMode::HBlank;

                    if self.lcd_status.hblank_int {
                        interrupts_requested.add(GPUInterrupts::LCDStat);
                    }

                    if self.lcdc.lcd_enable {
                        self.render_scan();
                    }
                }
            },
            GPUMode::HBlank => {
                if self.mode_clock >= 204 {
                    self.mode_clock = 0;
                    self.ly += 1;

                    if self.ly == 143 {
                        self.lcd_status.mode = GPUMode::VBlank;
                        interrupts_requested.add(GPUInterrupts::VBlank);
                        if self.lcd_status.vblank_int {
                            interrupts_requested.add(GPUInterrupts::LCDStat);
                        }
                    } else {
                        self.lcd_status.mode = GPUMode::OAMRead;
                    }
                }
            },
            GPUMode::VBlank => {
                if self.mode_clock >= 456 {
                    self.mode_clock = 0;
                    self.ly += 1;

                    if self.ly > 153 {
                        self.lcd_status.mode = GPUMode::OAMRead;
                        self.ly = 0;
                    }
                }
            }
        }

        if self.ly == self.lyc {
            if self.lcd_status.lyc_equals_ly_int && !self.lcd_status.lyc_equals_ly {
                interrupts_requested.add(GPUInterrupts::LCDStat);
            }
            self.lcd_status.lyc_equals_ly = true;
        } else {
            if self.lcd_status.lyc_equals_ly_int && self.lcd_status.lyc_equals_ly {
                interrupts_requested.add(GPUInterrupts::LCDStat);
            }
            self.lcd_status.lyc_equals_ly = false;
        }

        interrupts_requested
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

            self.tile_set[tile_idx].data[row_idx][col_idx] = (msb << 1) | lsb;
        }
    }

    pub fn update_object(&mut self, address: usize, val: u8) {
        let address = address & 0x1FF;
        // objects take 4 bytes
        let obj_idx = address >> 2;

        if obj_idx < 40 {
            let obj_byte = address & 0x3;

            match obj_byte {
                0 => self.obj_set[obj_idx].y = (val as i16) - 16,
                1 => self.obj_set[obj_idx].x = (val as i16) - 8,
                2 => self.obj_set[obj_idx].tile = val,
                3 => {
                    self.obj_set[obj_idx].attributes.bg_win_over_obj = (val & 0x80) != 0;
                    self.obj_set[obj_idx].attributes.y_flip = (val & 0x40) != 0;
                    self.obj_set[obj_idx].attributes.x_flip = (val & 0x20) != 0;
                    self.obj_set[obj_idx].attributes.palette = (val & 0x10) != 0;
                },
                _ => { },
            }
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

    fn get_obj_color(&self, val: u8, palette: bool) -> Color {
        match val {
            0 => if palette { self.obj1_palette.0 } else { self.obj0_palette.0 }
            1 => if palette { self.obj1_palette.1 } else { self.obj0_palette.1 }
            2 => if palette { self.obj1_palette.2 } else { self.obj0_palette.2 }
            3 => if palette { self.obj1_palette.3 } else { self.obj0_palette.3 }
            _ => panic!("Cannot convert {} to color", val),
        }
    }


    fn render_scan(&mut self) {
        let base_address: u16 = if self.lcdc.bg_tilemap { 0x1C00 } else { 0x1800 };
        let scy = self.scy as u16;
        let scx = self.scx as u16;
        let ly = self.ly as u16;
        let tile_map_y = ((scy + ly) & 0xff) >> 3;
        let mut pixel = [0; 160];

        let visible_offset = base_address + (tile_map_y * 32);
        let mut line_offset = scx >> 3;

        let mut x = scx & 7;
        let y = (ly + scy) & 7;

        let mut canvas_offset : usize = (ly * 160).into();

        let mut tile = self.video_ram[(visible_offset + line_offset) as usize] as i16;

        if !self.lcdc.bg_window_addressing_mode && tile < 0 {
            tile += 256;
        }

        for i in 0..160 {
            pixel[i] = self.tile_set[tile as usize].data[y as usize][x as usize];
            let color = self.get_color(pixel[i]);

            self.canvas_buffer[canvas_offset] = color.to_rgb();
            canvas_offset += 1;

            x += 1;
            if x == 8 {
                x = 0;
                line_offset = (line_offset + 1) & 0x1F;

                tile = self.video_ram[(visible_offset + line_offset) as usize] as i16;

                if !self.lcdc.bg_window_addressing_mode && tile < 0 {
                    tile += 256;
                }
            }
        }

        // Render OAM
        if self.lcdc.obj_enable {
            let mut rendered_objects = 0;
            for obj in self.obj_set.iter() {
                // only 10 objects rendered per scan-line
                if rendered_objects > 10 {
                    break;
                }

                if obj.y <= (self.ly as i16) && (self.ly as i16) < (obj.y + 8) {
                    rendered_objects += 1;

                    let mut canvas_offset: i16 = ly as i16 * 160 + obj.x;

                    let tile_y = if !obj.attributes.y_flip {
                        self.ly as i16 - obj.y
                    } else {
                        7 - (self.ly as i16 - obj.y)
                    };

                    for x in 0..8 {
                        let tile_x = if !obj.attributes.x_flip {
                            x
                        } else {
                            7 - x
                        };

                        let color_idx = self.tile_set[obj.tile as usize].data[tile_y as usize][tile_x as usize];
                        if color_idx == 0 {
                            // color 0 is transparent, don't render anything
                            canvas_offset += 1;
                            continue;
                        }

                        let color = self.get_obj_color(color_idx, obj.attributes.palette);

                        if ((obj.x + x) >= 0)
                            && ((obj.x + x) < 160)
                            && (obj.attributes.bg_win_over_obj || !pixel[(obj.x + x) as usize] != 0) {
                            self.canvas_buffer[canvas_offset as usize] = color.to_rgb();
                            canvas_offset += 1;
                        }
                    }
                }
            }
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.video_ram[address as usize - 0x8000],
            0xFE00..=0xFE9F => {
                self.oam[address as usize - 0xFE00]
            }
            0xFF40 => self.lcdc.into(),
            0xFF41 => self.lcd_status.into(),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bg_palette.into(),
            0xFF48 => self.obj0_palette.into(),
            0xFF49 => self.obj1_palette.into(),
            _ => { 0 /* TODO */ },
        }
    }

    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0x8000..=0x9FFF => {
                self.video_ram[address as usize - 0x8000] = val;
                self.update_tile(address as usize);
            },
            0xFE00..=0xFE9F => {
                self.oam[address as usize - 0xFE00] = val;
                self.update_object(address as usize, val);
            }
            0xFF40 => {
                self.lcdc = LCDC::from(val);
            },
            0xFF41 => {
                let mut new_status = LCDStatus::from(val);
                new_status.lyc_equals_ly =  self.lcd_status.lyc_equals_ly;
                new_status.mode =  self.lcd_status.mode;
                self.lcd_status = new_status;
            },
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => { },
            0xFF45 => { self.lyc = val; },
            0xFF47 => {
                self.bg_palette = Palette::from(val);
            },
            0xFF48 => {
                self.obj0_palette = Palette::from(val);
            }
            0xFF49 => {
                self.obj1_palette = Palette::from(val);
            }
            _ => { /* TODO */ },
        }
    }
}
