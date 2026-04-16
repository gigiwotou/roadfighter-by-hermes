use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rand::Rng;
use std::env;
use std::process::Command;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 600;
const ROAD_WIDTH: f32 = 360.0;
const LANE_COUNT: usize = 4;

fn main() {
    // 强制使用 X11 后端以支持窗口定位 (Wayland 下 set_position 无效)
    env::set_var("WAYLAND_DISPLAY", "");
    
    // 先获取屏幕尺寸
    let (screen_w, screen_h) = get_screen_size();
    let win_x = ((screen_w - SCREEN_WIDTH as u32) / 2) as isize;
    let win_y = ((screen_h - SCREEN_HEIGHT as u32) / 2) as isize;

    let mut window = Window::new(
        "Road Fighter",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions {
            resize: false,
            scale: minifb::Scale::X1,
            scale_mode: minifb::ScaleMode::Center,
            topmost: false,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("Cannot create window: {}", e);
    });

    // 使用 minifb 内置的位置设置
    window.set_position(win_x, win_y);
    window.set_target_fps(60);

    let mut game = Game::new();

 while window.is_open() && !window.is_key_down(Key::Escape) {
 game.update(&window);
 
 let mut buffer = vec![DARK_GRAY; SCREEN_WIDTH * SCREEN_HEIGHT];
 game.draw(&mut buffer);
 
 // 安全更新缓冲区
 if let Err(e) = window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT) {
     eprintln!("Buffer update error: {}", e);
     break;
 }
 }
}


fn get_screen_size() -> (u32, u32) {
    // 优先使用 xrandr 获取实际显示器尺寸
    if let Ok(output) = Command::new("xrandr").output() {
        let s = String::from_utf8_lossy(&output.stdout);
        for line in s.lines() {
            // 查找已连接的显示器，格式如: "Virtual-1 connected primary 4096x2560+0+0"
            if line.contains(" connected") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    // 查找 分辨率部分 "4096x2560+0+0"
                    if part.contains('x') && part.contains('+') {
                        // 提取 4096x2560 部分
                        let res_part = part.split('+').next().unwrap_or("");
                        let dims: Vec<&str> = res_part.split('x').collect();
                        if dims.len() == 2 {
                            if let (Ok(w), Ok(h)) = (dims[0].parse::<u32>(), dims[1].parse::<u32>()) {
                                return (w, h);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 备用: 使用 xdpyinfo
    if let Ok(output) = Command::new("xdpyinfo").output() {
        let s = String::from_utf8_lossy(&output.stdout);
        for line in s.lines() {
            if line.contains("dimensions:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let dims: Vec<&str> = parts[1].split('x').collect();
                    if dims.len() == 2 {
                        if let (Ok(w), Ok(h)) = (dims[0].parse::<u32>(), dims[1].parse::<u32>()) {
                            return (w, h);
                        }
                    }
                }
            }
        }
    }
 (1920, 1080)
}

// 颜色定义
const BLACK: u32 = 0x000000;
const WHITE: u32 = 0xFFFFFF;
const RED: u32 = 0xFF0000;
const GREEN: u32 = 0x00FF00;
const BLUE: u32 = 0x0000FF;
const YELLOW: u32 = 0xFFFF00;
const GRAY: u32 = 0x808080;
const DARK_GRAY: u32 = 0x404040;
const GRASS_GREEN: u32 = 0x228B22;
const ORANGE: u32 = 0xFFA500;
const PURPLE: u32 = 0x800080;
const GOLD: u32 = 0xFFD700;
const SKY_BLUE: u32 = 0x87CEEB;



#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Title,
    Playing,
    GameOver,
}

struct Player {
    x: f32,
    y: f32,
    fuel: f32,
    lives: i32,
    score: i32,
}

struct EnemyCar {
    x: f32,
    y: f32,
    speed: f32,
    color: u32,
}

struct Fuel {
    x: f32,
    y: f32,
}

struct RoadLine {
    y: f32,
}

struct Game {
    state: GameState,
    player: Player,
    enemies: Vec<EnemyCar>,
    fuels: Vec<Fuel>,
    road_lines: Vec<RoadLine>,
    scroll_speed: f32,
    road_left: f32,
    road_right: f32,
    spawn_timer: f32,
    fuel_timer: f32,
    fuel_consumption_timer: f32,
    high_score: i32,
    level: i32,
    frame_count: u32,
}

impl Game {
    fn new() -> Self {
        let road_left = (SCREEN_WIDTH as f32 - ROAD_WIDTH) / 2.0;
        let road_right = road_left + ROAD_WIDTH;

        let mut road_lines = Vec::new();
        for i in 0..12 {
            road_lines.push(RoadLine { y: i as f32 * 60.0 });
        }

        Game {
            state: GameState::Title,
            player: Player {
                x: SCREEN_WIDTH as f32 / 2.0 - 18.0,
                y: SCREEN_HEIGHT as f32 - 100.0,
                fuel: 100.0,
                lives: 3,
                score: 0,
            },
            enemies: Vec::new(),
            fuels: Vec::new(),
            road_lines,
            scroll_speed: 3.0,
            road_left,
            road_right,
            spawn_timer: 0.0,
            fuel_timer: 0.0,
            fuel_consumption_timer: 0.0,
            high_score: 0,
            level: 1,
            frame_count: 0,
        }
    }

    fn reset(&mut self) {
        self.player.x = SCREEN_WIDTH as f32 / 2.0 - 18.0;
        self.player.y = SCREEN_HEIGHT as f32 - 100.0;
        self.player.fuel = 100.0;
        self.player.lives = 3;
        self.player.score = 0;
        self.enemies.clear();
        self.fuels.clear();
        self.scroll_speed = 3.0;
        self.level = 1;
    }

    fn update(&mut self, window: &Window) {
        self.frame_count += 1;

        match self.state {
            GameState::Title => {
                if window.is_key_pressed(Key::Space, KeyRepeat::No) {
                    self.state = GameState::Playing;
                    self.reset();
                }
            }
            GameState::Playing => {
                self.update_playing(window);
            }
            GameState::GameOver => {
                if window.is_key_pressed(Key::Space, KeyRepeat::No) {
                    self.state = GameState::Title;
                }
            }
        }
    }

    fn update_playing(&mut self, window: &Window) {
        let move_speed = 4.0;

        if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
            self.player.x -= move_speed;
        }
        if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
            self.player.x += move_speed;
        }
        if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {
            self.player.y -= move_speed * 0.5;
        }
        if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
            self.player.y += move_speed * 0.5;
        }

        self.player.x = self.player.x.max(self.road_left + 5.0).min(self.road_right - 36.0 - 5.0);
        self.player.y = self.player.y.max(50.0).min(SCREEN_HEIGHT as f32 - 60.0);

        for line in &mut self.road_lines {
            line.y += self.scroll_speed;
            if line.y > SCREEN_HEIGHT as f32 {
                line.y = -20.0;
            }
        }

        self.spawn_timer += 1.0 / 60.0;
        if self.spawn_timer > 1.5 - (self.level as f32 * 0.1).min(0.8) {
            self.spawn_enemy();
            self.spawn_timer = 0.0;
        }

        self.fuel_timer += 1.0 / 60.0;
        if self.fuel_timer > 4.0 {
            self.spawn_fuel();
            self.fuel_timer = 0.0;
        }

        for enemy in &mut self.enemies {
            enemy.y += enemy.speed;
        }

        for fuel in &mut self.fuels {
            fuel.y += self.scroll_speed;
        }

        self.enemies.retain(|e| e.y < SCREEN_HEIGHT as f32 + 100.0);
        self.fuels.retain(|f| f.y < SCREEN_HEIGHT as f32 + 50.0);

        let player_left = self.player.x;
        let player_right = self.player.x + 36.0;
        let player_top = self.player.y;
        let player_bottom = self.player.y + 60.0;

        for enemy in &self.enemies {
            let enemy_left = enemy.x;
            let enemy_right = enemy.x + 32.0;
            let enemy_top = enemy.y;
            let enemy_bottom = enemy.y + 52.0;

            if player_left < enemy_right && player_right > enemy_left
                && player_top < enemy_bottom && player_bottom > enemy_top
            {
                self.player.lives -= 1;
                if self.player.lives <= 0 {
                    if self.player.score > self.high_score {
                        self.high_score = self.player.score;
                    }
                    self.state = GameState::GameOver;
                } else {
                    self.player.x = SCREEN_WIDTH as f32 / 2.0 - 18.0;
                    self.player.y = SCREEN_HEIGHT as f32 - 100.0;
                }
                return;
            }
        }

        self.fuels.retain(|fuel| {
            let fuel_left = fuel.x;
            let fuel_right = fuel.x + 28.0;
            let fuel_top = fuel.y;
            let fuel_bottom = fuel.y + 18.0;

            if player_left < fuel_right && player_right > fuel_left
                && player_top < fuel_bottom && player_bottom > fuel_top
            {
                self.player.fuel = (self.player.fuel + 30.0).min(100.0);
                self.player.score += 50;
                false
            } else {
                true
            }
        });

        self.fuel_consumption_timer += 1.0 / 60.0;
        if self.fuel_consumption_timer > 0.1 {
            self.player.fuel -= 0.3 * (1.0 + self.level as f32 * 0.1);
            self.fuel_consumption_timer = 0.0;
        }

 if self.player.fuel <= 0.0 {
 self.player.fuel = 0.0; // 确保 fuel 不为负
 self.player.lives -= 1;
 if self.player.lives <= 0 {
     if self.player.score > self.high_score {
         self.high_score = self.player.score;
     }
     self.state = GameState::GameOver;
 } else {
     self.player.fuel = 50.0;
     self.player.x = SCREEN_WIDTH as f32 / 2.0 - 18.0;
 }
 }

        self.player.score += 1;

        if self.player.score > self.level * 500 {
            self.level += 1;
            self.scroll_speed += 0.5;
        }
    }

    fn spawn_enemy(&mut self) {
        let mut rng = rand::thread_rng();
        let lane_width = ROAD_WIDTH / LANE_COUNT as f32;
        let lane = rng.gen_range(0..LANE_COUNT);
        let x = self.road_left + lane as f32 * lane_width + (lane_width - 32.0) / 2.0;

        let colors = [RED, YELLOW, PURPLE, ORANGE];
        let color = colors[rng.gen_range(0..colors.len())];

        self.enemies.push(EnemyCar {
            x,
            y: -52.0,
            speed: self.scroll_speed + rng.gen_range(0.0..2.0),
            color,
        });
    }

    fn spawn_fuel(&mut self) {
        let mut rng = rand::thread_rng();
        let lane_width = ROAD_WIDTH / LANE_COUNT as f32;
        let lane = rng.gen_range(0..LANE_COUNT);
        let x = self.road_left + lane as f32 * lane_width + (lane_width - 28.0) / 2.0;

        self.fuels.push(Fuel { x, y: -18.0 });
    }

 fn draw(&self, buffer: &mut Vec<u32>) {
 // 安全检查
 if buffer.len() != SCREEN_WIDTH * SCREEN_HEIGHT {
     return;
 }

 // 清空
 for pixel in buffer.iter_mut() {
     *pixel = DARK_GRAY;
 }

 let road_left_int = self.road_left.max(0.0).min(SCREEN_WIDTH as f32) as usize;
 let road_right_int = self.road_right.max(0.0).min(SCREEN_WIDTH as f32) as usize;

        // 草地
        for y in 0..SCREEN_HEIGHT {
            for x in 0..road_left_int {
                buffer[y * SCREEN_WIDTH + x] = GRASS_GREEN;
            }
            for x in road_right_int..SCREEN_WIDTH {
                buffer[y * SCREEN_WIDTH + x] = GRASS_GREEN;
            }
        }

        // 道路
        for y in 0..SCREEN_HEIGHT {
            for x in road_left_int..road_right_int {
                buffer[y * SCREEN_WIDTH + x] = GRAY;
            }
        }

        // 道路边缘
        draw_rect(buffer, self.road_left, 0.0, 8.0, SCREEN_HEIGHT as f32, WHITE);
        draw_rect(buffer, self.road_right - 8.0, 0.0, 8.0, SCREEN_HEIGHT as f32, WHITE);

        // 车道分隔线
        let lane_width = ROAD_WIDTH / LANE_COUNT as f32;
        for i in 1..LANE_COUNT {
            for line in &self.road_lines {
                let x = self.road_left + i as f32 * lane_width;
                draw_rect(buffer, x - 2.0, line.y, 4.0, 30.0, WHITE);
            }
        }

        match self.state {
            GameState::Title => self.draw_title(buffer),
            GameState::Playing => self.draw_game(buffer),
            GameState::GameOver => self.draw_game_over(buffer),
        }
    }

    fn draw_title(&self, buffer: &mut Vec<u32>) {
        // 背景
        draw_rect(buffer, 150.0, 80.0, 500.0, 440.0, 0x1A1A2E);
        draw_rect(buffer, 160.0, 90.0, 480.0, 420.0, 0x0F0F1A);

        // 标题
        draw_text(buffer, "ROAD FIGHTER", 220, 130, 3, RED);
        draw_text(buffer, "FC Racing Game", 280, 200, 2, YELLOW);

        // 示例赛车
        draw_car(buffer, SCREEN_WIDTH as f32 / 2.0 - 18.0, 260.0, BLUE, true);

        // 操作说明
        draw_text(buffer, "Controls:", 340, 360, 1, WHITE);
        draw_text(buffer, "Arrow Keys / WASD - Move", 250, 400, 1, 0xAAAAAA);
        draw_text(buffer, "Collect Green Fuel", 290, 430, 1, GREEN);

        // 开始提示
        if (self.frame_count / 30) % 2 == 0 {
            draw_text(buffer, "Press SPACE to Start", 270, 480, 2, GREEN);
        }
    }

    fn draw_game(&self, buffer: &mut Vec<u32>) {
        // 燃料补给
        for fuel in &self.fuels {
            draw_rect(buffer, fuel.x, fuel.y, 28.0, 18.0, GREEN);
            draw_rect(buffer, fuel.x + 4.0, fuel.y + 4.0, 20.0, 10.0, 0x00AA00);
        }

        // 敌方车辆
        for enemy in &self.enemies {
            draw_car(buffer, enemy.x, enemy.y, enemy.color, false);
        }

        // 玩家
        draw_car(buffer, self.player.x, self.player.y, BLUE, true);

        // UI
        self.draw_ui(buffer);
    }

    fn draw_game_over(&self, buffer: &mut Vec<u32>) {
        self.draw_game(buffer);

        // 半透明覆盖
        draw_rect(buffer, 0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32, 0x000000AA);
        
        // 框
        draw_rect(buffer, 150.0, 150.0, 500.0, 300.0, 0x1A1A2E);
        draw_rect(buffer, 160.0, 160.0, 480.0, 280.0, BLACK);

        draw_text(buffer, "GAME OVER", 280, 190, 3, RED);
        draw_text(buffer, &format!("Score: {}", self.player.score), 300, 280, 2, YELLOW);
        draw_text(buffer, &format!("High Score: {}", self.high_score), 280, 320, 2, GOLD);
        draw_text(buffer, &format!("Level: {}", self.level), 330, 360, 1, WHITE);

        if (self.frame_count / 30) % 2 == 0 {
            draw_text(buffer, "Press SPACE", 310, 410, 2, GREEN);
        }
    }

    fn draw_ui(&self, buffer: &mut Vec<u32>) {
        // 燃料条
        draw_rect(buffer, 10.0, 10.0, 124.0, 24.0, BLACK);
        draw_rect(buffer, 12.0, 12.0, 120.0, 20.0, DARK_GRAY);
        
        let fuel_width = (self.player.fuel.max(0.0) / 100.0) * 116.0;
        let fuel_color = if self.player.fuel > 30.0 { GREEN } else { RED };
        draw_rect(buffer, 14.0, 14.0, fuel_width, 16.0, fuel_color);

        draw_text(buffer, "FUEL", 140, 24, 1, WHITE);
        
        draw_text(buffer, &format!("SCORE:{}", self.player.score), 10, 50, 1, YELLOW);
        draw_text(buffer, &format!("LV:{}", self.level), 10, 74, 1, WHITE);
        draw_text(buffer, &format!("HIGH:{}", self.high_score), SCREEN_WIDTH as i32 - 130, 24, 1, GOLD);

        // 生命
        draw_text(buffer, "LIFE:", 10, 98, 1, WHITE);
        for i in 0..self.player.lives {
            draw_mini_car(buffer, 70 + i * 30, 85, BLUE);
        }
    }
}

fn draw_rect(buffer: &mut Vec<u32>, x: f32, y: f32, w: f32, h: f32, color: u32) {
    let x_start = x.max(0.0) as usize;
    let x_end = (x + w).min(SCREEN_WIDTH as f32) as usize;
    let y_start = y.max(0.0) as usize;
    let y_end = (y + h).min(SCREEN_HEIGHT as f32) as usize;

    if x_end <= x_start || y_end <= y_start {
        return;
    }

    for py in y_start..y_end {
        for px in x_start..x_end {
            buffer[py * SCREEN_WIDTH + px] = color;
        }
    }
}

fn draw_car(buffer: &mut Vec<u32>, x: f32, y: f32, color: u32, is_player: bool) {
    draw_rect(buffer, x, y + 10.0, 36.0, 40.0, color);
    
    let darker = darken_color(color, 0.7);
    draw_rect(buffer, x + 4.0, y, 28.0, 10.0, darker);
    draw_rect(buffer, x + 4.0, y + 50.0, 28.0, 10.0, darker);
    
    draw_rect(buffer, x + 8.0, y + 15.0, 20.0, 8.0, SKY_BLUE);
    draw_rect(buffer, x + 8.0, y + 35.0, 20.0, 6.0, SKY_BLUE);
    
    draw_rect(buffer, x - 3.0, y + 5.0, 6.0, 12.0, BLACK);
    draw_rect(buffer, x - 3.0, y + 43.0, 6.0, 12.0, BLACK);
    draw_rect(buffer, x + 33.0, y + 5.0, 6.0, 12.0, BLACK);
    draw_rect(buffer, x + 33.0, y + 43.0, 6.0, 12.0, BLACK);
    
    if is_player {
        draw_rect(buffer, x + 15.0, y + 10.0, 6.0, 40.0, WHITE);
    }
}

fn draw_mini_car(buffer: &mut Vec<u32>, x: i32, y: i32, color: u32) {
    draw_rect(buffer, x as f32, y as f32, 24.0, 40.0, color);
    draw_rect(buffer, x as f32 + 4.0, (y + 4) as f32, 16.0, 5.0, SKY_BLUE);
}

fn darken_color(color: u32, factor: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32 * factor;
    let g = ((color >> 8) & 0xFF) as f32 * factor;
    let b = (color & 0xFF) as f32 * factor;
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

// 5x7 位图字体数据 (每个字符5x7像素)
fn get_font_data() -> Vec<Vec<Vec<bool>>> {
    // 0-9, A-Z 的位图数据
    let chars = [
        // 0
        vec![" ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### "],
        // 1
        vec!["  #  ", " ##  ", "  #  ", "  #  ", "  #  ", "  #  ", " ### "],
        // 2
        vec![" ### ", "#   #", "    #", " ### ", "#    ", "#    ", "#####"],
        // 3
        vec![" ### ", "#   #", "    #", "  ## ", "    #", "#   #", " ### "],
        // 4
        vec!["   # ", "  ## ", " # # ", "#  # ", "#####", "   # ", "   # "],
        // 5
        vec!["#####", "#    ", "#    "," ### ", "    #", "#   #", " ### "],
        // 6
        vec![" ### ", "#    ", "#    ", "#### ", "#   #", "#   #", " ### "],
        // 7
        vec!["#####", "    #", "   # ", "  #  ", " #   ", " #   ", " #   "],
        // 8
        vec![" ### ", "#   #", "#   #", " ### ", "#   #", "#   #", " ### "],
        // 9
        vec![" ### ", "#   #", "#   #", " ####", "    #", "    #", " ### "],
        // A
        vec![" ### ", "#   #", "#   #", "#####", "#   #", "#   #", "#   #"],
        // B
        vec!["#### ", "#   #", "#   #", "#### ", "#   #", "#   #", "#### "],
        // C
        vec![" ### ", "#   #", "#    ", "#    ", "#    ", "#   #", " ### "],
        // D
        vec!["#### ", "#   #", "#   #", "#   #", "#   #", "#   #", "#### "],
        // E
        vec!["#####", "#    ", "#    ", "###  ", "#    ", "#    ", "#####"],
        // F
        vec!["#####", "#    ", "#    ", "###  ", "#    ", "#    ", "#    "],
        // G
        vec![" ### ", "#    ", "#    ", "#  ##", "#   #", "#   #", " ### "],
        // H
        vec!["#   #", "#   #", "#   #", "#####", "#   #", "#   #", "#   #"],
        // I
        vec![" ### ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", " ### "],
        // J
        vec!["  ###", "   # ", "   # ", "   # ", "   # ", "#  # ", " ##  "],
        // K
        vec!["#   #", "#  # ", "##   ", "#    ", "##   ", "#  # ", "#   #"],
        // L
        vec!["#    ", "#    ", "#    ", "#    ", "#    ", "#    ", "#####"],
        // M
        vec!["#   #", "## ##", "# # #", "#   #", "#   #", "#   #", "#   #"],
        // N
        vec!["#   #", "##  #", "# # #", "#  ##", "#   #", "#   #", "#   #"],
        // O
        vec![" ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### "],
        // P
        vec!["#### ", "#   #", "#   #", "#### ", "#    ", "#    ", "#    "],
        // Q
        vec![" ### ", "#   #", "#   #", "#   #", "# # #", "#  # ", " ## #"],
        // R
        vec!["#### ", "#   #", "#   #", "#### ", "#  # ", "#   #", "#   #"],
        // S
        vec![" ####", "#    ", "#    ", " ### ", "    #", "    #", "#### "],
        // T
        vec!["#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  "],
        // U
        vec!["#   #", "#   #", "#   #", "#   #", "#   #", "#   #", " ### "],
        // V
        vec!["#   #", "#   #", "#   #", "#   #", " # # ", " # # ", "  #  "],
        // W
        vec!["#   #", "#   #", "#   #", "# # #", "# # #", "## ##", "#   #"],
        // X
        vec!["#   #", " # # ", "  #  ", "  #  ", "  #  ", " # # ", "#   #"],
        // Y
        vec!["#   #", " # # ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  "],
        // Z
        vec!["#####", "   # ", "  #  ", " #   ", "#    ", "#    ", "#####"],
    ];
    
    chars.iter().map(|c| {
        c.iter().map(|row| {
            row.chars().map(|ch| ch == '#').collect()
        }).collect()
    }).collect()
}

fn draw_text(buffer: &mut Vec<u32>, text: &str, x: i32, y: i32, scale: i32, color: u32) {
    let font = get_font_data();
    let pixel_size = scale;
    let spacing = 6 * scale;
    
    for (i, ch) in text.chars().enumerate() {
        let char_index = if ch >= '0' && ch <= '9' {
            (ch as usize) - ('0' as usize)
        } else if ch >= 'A' && ch <= 'Z' {
            (ch as usize) - ('A' as usize) + 10
        } else if ch >= 'a' && ch <= 'z' {
            (ch as usize) - ('a' as usize) + 10
        } else if ch == ' ' {
            continue;
        } else if ch == ':' {
            // 冒号特殊处理
            let cx = x + (i as i32) * spacing;
            draw_rect(buffer, cx as f32 + pixel_size as f32, (y + pixel_size) as f32, pixel_size as f32, pixel_size as f32, color);
            draw_rect(buffer, cx as f32 + pixel_size as f32, (y + 3 * pixel_size) as f32, pixel_size as f32, pixel_size as f32, color);
            continue;
        } else {
            continue;
        };

        if char_index < font.len() {
            let char_data = &font[char_index];
            let cx = x + (i as i32) * spacing;
            
            for (row_idx, row) in char_data.iter().enumerate() {
                for (col_idx, &is_set) in row.iter().enumerate() {
                    if is_set {
                        let px = cx + (col_idx as i32) * pixel_size;
                        let py = y + (row_idx as i32) * pixel_size;
                        draw_rect(buffer, px as f32, py as f32, pixel_size as f32, pixel_size as f32, color);
                    }
                }
            }
        }
    }
}
