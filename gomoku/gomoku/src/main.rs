use eframe::{
    egui::{self, Frame, Margin, Ui, RichText},
    epaint::{pos2, Color32, Pos2},
};

mod audio;
use audio::AudioManager;

// 游戏模式枚举
#[derive(PartialEq)]
enum GameMode {
    MainMenu,
    PlayerVsPlayer,
    PlayerVsAI,
}

struct AppUI {
    // 游戏模式状态
    game_mode: GameMode,
    
    // 一个 15 * 15 的棋盘，黑子用 1 表示，白子用 2 表示，空位用 0 表示
    board_data: [[u8; 15]; 15],

    // 棋盘起始点，棋盘左上角距离画布左上角的距离
    start_point: Pos2,

    // 是否该黑子落子了
    is_black: bool,

    // 是否已经产生了赢家
    is_winner: bool,

    // AI模式相关
    player_is_black: bool,  // 玩家是否为黑子
    ai_thinking: bool,      // AI是否正在思考
    color_selected: bool,   // 是否已选择颜色
    ai_delay_timer: f32,    // AI延迟计时器
    ai_pending_move: Option<(usize, usize)>, // AI待执行的移动

    // 音频系统
    audio_manager: AudioManager,

    frame: egui::Frame,
}

impl Default for AppUI {
    fn default() -> Self {
        Self {
            game_mode: GameMode::MainMenu,
            frame: Frame {
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                fill: egui::Color32::LIGHT_YELLOW,
                ..Default::default()
            },
            board_data: [[0; 15]; 15],
            // 棋盘左上角距离画布左上角的距离
            start_point: pos2(15.0, 15.0),
            is_black: true,
            is_winner: false,
            player_is_black: true,  // 默认玩家为黑子
            ai_thinking: false,
            color_selected: false,
            ai_delay_timer: 0.0,
            ai_pending_move: None,
            audio_manager: AudioManager::new().unwrap_or_else(|_| {
                // 如果音频初始化失败，程序仍然可以运行，只是没有音效
                panic!("Failed to initialize audio system");
            }),
        }
    }
}

impl AppUI {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// 渲染颜色选择界面
    fn render_color_selection(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading(RichText::new("Choose Your Color").size(32.0).color(egui::Color32::DARK_BLUE));
            ui.add_space(40.0);
            
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                // 黑子按钮
                if ui.add_sized([180.0, 60.0], egui::Button::new(RichText::new("Black (First Move)").size(18.0))).clicked() {
                    self.player_is_black = true;
                    self.color_selected = true;
                    self.is_black = true; // 玩家先手
                }
                
                ui.add_space(20.0);
                
                // 白子按钮
                if ui.add_sized([180.0, 60.0], egui::Button::new(RichText::new("White (Second Move)").size(18.0))).clicked() {
                    self.player_is_black = false;
                    self.color_selected = true;
                    self.is_black = true; // AI先手
                    // AI第一步下在中央
                    self.board_data[7][7] = 1; // 黑子下在中央
                    self.audio_manager.play_black_move(); // 播放AI落子音效
                    self.is_black = false; // 轮到白子
                }
                
                ui.add_space(30.0);
                
                // 说明文字
                ui.label(RichText::new("Black always goes first").size(14.0).color(egui::Color32::GRAY));
            });
        });
    }

    /// 渲染主菜单界面
    fn render_main_menu(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            // 标题
            ui.add_space(50.0);
            ui.heading(RichText::new("Gomoku Game").size(36.0).color(egui::Color32::DARK_BLUE));
            ui.add_space(30.0);
            
            // 模式选择按钮
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                // 双人对战按钮
                if ui.add_sized([200.0, 50.0], egui::Button::new(RichText::new("Player vs Player").size(20.0))).clicked() {
                    self.game_mode = GameMode::PlayerVsPlayer;
                    self.restart();
                }
                
                ui.add_space(15.0);
                
                // 人机对战按钮
                if ui.add_sized([200.0, 50.0], egui::Button::new(RichText::new("Player vs AI").size(20.0))).clicked() {
                    self.game_mode = GameMode::PlayerVsAI;
                    self.restart();
                    self.color_selected = false; // 重置颜色选择状态
                }
                
                ui.add_space(20.0);
                
                // 说明文字
                ui.label(RichText::new("Choose your game mode").size(14.0).color(egui::Color32::GRAY));
            });
        });
    }

    /// 绘制棋盘
    fn render_board(&self, ui: &Ui) {
        let stroke = egui::Stroke::new(1.0, egui::Color32::DARK_GRAY);

        // 先画横线
        for i in 0..15 {
            let start = self.start_point + egui::Vec2::new(0.0, i as f32 * 30.0);
            let end = start + egui::Vec2::new(420.0, 0.0);
            ui.painter().line_segment([start, end], stroke);
        }
        // 再画竖线
        for i in 0..15 {
            let start = self.start_point + egui::Vec2::new(i as f32 * 30.0, 0.0);
            let end = start + egui::Vec2::new(0.0, 420.0);
            ui.painter().line_segment([start, end], stroke);
        }
    }

    /// 画圆
    fn render_circle(&self, ui: &Ui, center: egui::Pos2, color: Color32, stroke_color: Color32) {
        let stroke = egui::Stroke::new(1.0, stroke_color);
        ui.painter().circle(center, 14.0, color, stroke)
    }

    /// 画白子
    fn render_white(&self, ui: &Ui, center: egui::Pos2) {
        self.render_circle(ui, center, Color32::WHITE, Color32::GRAY)
    }

    /// 画黑子
    fn render_black(&self, ui: &Ui, center: egui::Pos2) {
        self.render_circle(ui, center, Color32::BLACK, Color32::BLACK)
    }

    /// 绘制棋子
    fn render_piece(&self, ui: &Ui) {
        // 遍历棋子数组数据
        for (i, x) in self.board_data.iter().enumerate() {
            for (j, y) in x.iter().enumerate() {
                match y {
                    1 => self.render_black(ui, self.get_position(i, j)),
                    2 => self.render_white(ui, self.get_position(i, j)),
                    _ => {}
                }
            }
        }
    }

    fn get_position(&self, x: usize, y: usize) -> Pos2 {
        // start + ( 30 * x, 30 * y )
        let x = x as f32;
        let y = y as f32;
        self.start_point + egui::Vec2::new(30.0 * x, 30.0 * y)
    }

    /// 处理鼠标点击事件
    fn handle_click(&mut self, pos: Pos2) {
        // 在AI模式下，只有玩家的回合才能点击
        if self.game_mode == GameMode::PlayerVsAI {
            let ai_piece = if self.player_is_black { 2 } else { 1 };
            let current_piece = if self.is_black { 1 } else { 2 };
            if current_piece == ai_piece {
                return; // AI的回合，不允许玩家点击
            }
        }

        // 首先 xy 都减去 15，然后除以 30，然后四舍五入
        let x = ((pos.x - 15.0) / 30.0).round() as usize;
        let y = ((pos.y - 15.0) / 30.0).round() as usize;
        // 如果点击了棋盘以外的空间，或者该点位已有棋子，什么事都不做
        if x > 14 || y > 14 || self.board_data[x][y] != 0 {
            return;
        }
        let piece_type = if self.is_black { 1 } else { 2 };
        self.board_data[x][y] = piece_type;
        
        // 播放相应的音效
        if piece_type == 1 {
            self.audio_manager.play_black_move();
        } else {
            self.audio_manager.play_white_move();
        }
        
        if self.check_winner(x, y) {
            self.is_winner = true;
            return;
        };
        self.is_black = !self.is_black;
    }

    /// 检查是否有获胜者
    fn check_winner(&self, x: usize, y: usize) -> bool {
        // 从最后一次的落点开始检查
        let current = self.board_data[x][y];
        let mut count = 1;

        // 先往左数，再往右数，累加，检查是否大于等于 5
        for i in 1..5 {
            if x < i || self.board_data[x - i][y] != current {
                break;
            }
            count += 1;
        }
        for i in 1..5 {
            if x + i > 14 || self.board_data[x + i][y] != current {
                break;
            }
            count += 1;
        }
        if count >= 5 {
            return true;
        } else {
            count = 1;
        }

        // 先往上数，再往下数，累加，检查是否大于等于 5
        for i in 1..5 {
            if y < i || self.board_data[x][y - i] != current {
                break;
            }
            count += 1;
        }
        for i in 1..5 {
            if y + i > 14 || self.board_data[x][y + i] != current {
                break;
            }
            count += 1;
        }
        if count >= 5 {
            return true;
        } else {
            count = 1;
        }

        // 先往左上数，再往右下数，累加，检查是否大于等于 5
        for i in 1..5 {
            if x < i || y < i || self.board_data[x - i][y - i] != current {
                break;
            }
            count += 1;
        }
        for i in 1..5 {
            if x + i > 14 || y + i > 14 || self.board_data[x + i][y + i] != current {
                break;
            }
            count += 1;
        }
        if count >= 5 {
            return true;
        } else {
            count = 1;
        }

        // 先往左下数，再往右上数，累加，检查是否大于等于 5
        // 往左下是 x- y+
        for i in 1..5 {
            if x < i || y + i > 14 || self.board_data[x - i][y + i] != current {
                break;
            }
            count += 1;
        }
        // 往右上是 x+ y-
        for i in 1..5 {
            if x + i > 14 || y < i || self.board_data[x + i][y - i] != current {
                break;
            }
            count += 1;
        }
        if count >= 5 {
            return true;
        }

        false
    }

    fn restart(&mut self) {
        self.board_data = [[0; 15]; 15];
        self.is_black = true;
        self.is_winner = false;
        self.player_is_black = true;  // 重置为玩家黑子先手
        self.ai_thinking = false;
        self.ai_delay_timer = 0.0;
        self.ai_pending_move = None;
    }

    /// AI落子逻辑
    fn ai_move(&mut self, delta_time: f32) {
        if self.game_mode != GameMode::PlayerVsAI || self.is_winner {
            return;
        }

        // 检查是否轮到AI
        let ai_piece = if self.player_is_black { 2 } else { 1 }; // AI为白子或黑子
        let current_piece = if self.is_black { 1 } else { 2 };
        
        if current_piece != ai_piece {
            return; // 不是AI的回合
        }

        // 如果有待执行的移动，检查延迟时间
        if let Some((x, y)) = self.ai_pending_move {
            self.ai_delay_timer += delta_time;
            if self.ai_delay_timer >= 0.5 {
                // 执行AI移动
                self.board_data[x][y] = ai_piece;
                
                // 播放AI落子音效
                if ai_piece == 1 {
                    self.audio_manager.play_black_move();
                } else {
                    self.audio_manager.play_white_move();
                }
                
                if self.check_winner(x, y) {
                    self.is_winner = true;
                    self.ai_pending_move = None;
                    self.ai_thinking = false;
                    return;
                }
                self.is_black = !self.is_black;
                
                // 重置状态
                self.ai_pending_move = None;
                self.ai_thinking = false;
                self.ai_delay_timer = 0.0;
            }
        } else {
            // 计算AI移动并设置延迟
            self.ai_thinking = true;
            let (best_x, best_y) = self.find_best_move();
            self.ai_pending_move = Some((best_x, best_y));
            self.ai_delay_timer = 0.0;
        }
    }

    /// 寻找最佳落子位置
    fn find_best_move(&self) -> (usize, usize) {
        let ai_piece = if self.player_is_black { 2 } else { 1 };
        let player_piece = if self.player_is_black { 1 } else { 2 };
        
        let mut best_score = -1000;
        let mut best_move = (7, 7); // 默认中心位置
        
        // 遍历所有空位
        for x in 0..15 {
            for y in 0..15 {
                if self.board_data[x][y] == 0 {
                    let score = self.evaluate_position(x, y, ai_piece, player_piece);
                    if score > best_score {
                        best_score = score;
                        best_move = (x, y);
                    }
                }
            }
        }
        
        best_move
    }

    /// 评估位置的价值
    fn evaluate_position(&self, x: usize, y: usize, ai_piece: u8, player_piece: u8) -> i32 {
        let mut score = 0;
        
        // 检查四个方向
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)]; // 水平、垂直、对角线
        
        for (dx, dy) in directions {
            // 评估AI在该方向的得分
            score += self.evaluate_direction(x, y, dx, dy, ai_piece) * 10;
            // 评估玩家在该方向的得分（防守）
            score += self.evaluate_direction(x, y, dx, dy, player_piece) * 8;
        }
        
        // 中心位置加分
        let center_distance = ((x as i32 - 7).abs() + (y as i32 - 7).abs()) as i32;
        score += (14 - center_distance) * 2;
        
        score
    }

    /// 评估某个方向的得分
    fn evaluate_direction(&self, x: usize, y: usize, dx: i32, dy: i32, piece: u8) -> i32 {
        let mut count = 0;
        let mut blocked = 0;
        
        // 向一个方向计数
        for i in 1..5 {
            let nx = (x as i32 + dx * i) as usize;
            let ny = (y as i32 + dy * i) as usize;
            
            if nx >= 15 || ny >= 15 {
                blocked += 1;
                break;
            }
            
            if self.board_data[nx][ny] == piece {
                count += 1;
            } else if self.board_data[nx][ny] == 0 {
                break;
            } else {
                blocked += 1;
                break;
            }
        }
        
        // 向另一个方向计数
        for i in 1..5 {
            let nx = (x as i32 - dx * i) as usize;
            let ny = (y as i32 - dy * i) as usize;
            
            if nx >= 15 || ny >= 15 {
                blocked += 1;
                break;
            }
            
            if self.board_data[nx][ny] == piece {
                count += 1;
            } else if self.board_data[nx][ny] == 0 {
                break;
            } else {
                blocked += 1;
                break;
            }
        }
        
        // 根据连子数和阻塞情况给分
        match count {
            4 => 10000,  // 五连
            3 => if blocked == 0 { 1000 } else { 100 },
            2 => if blocked == 0 { 100 } else { 10 },
            1 => if blocked == 0 { 10 } else { 1 },
            _ => 0,
        }
    }
}

impl eframe::App for AppUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 获取时间增量
        let delta_time = ctx.input(|i| i.unstable_dt);
        
        match self.game_mode {
            GameMode::MainMenu => {
                egui::CentralPanel::default()
                    .frame(self.frame)
                    .show(ctx, |ui| {
                        self.render_main_menu(ui);
                    });
            }
            GameMode::PlayerVsAI if !self.color_selected => {
                egui::CentralPanel::default()
                    .frame(self.frame)
                    .show(ctx, |ui| {
                        self.render_color_selection(ui);
                    });
            }
            GameMode::PlayerVsPlayer | GameMode::PlayerVsAI => {
                egui::CentralPanel::default()
                    .frame(self.frame)
                    .show(ctx, |ui| {
                        // 添加返回主菜单按钮和游戏信息
                        ui.horizontal(|ui| {
                            if ui.button("Back to Menu").clicked() {
                                self.game_mode = GameMode::MainMenu;
                                return;
                            }
                            
                            // 显示当前回合信息
                            if self.game_mode == GameMode::PlayerVsAI {
                                let current_player = if self.is_black {
                                    if self.player_is_black { "Player (Black)" } else { "AI (Black)" }
                                } else {
                                    if self.player_is_black { "AI (White)" } else { "Player (White)" }
                                };
                                
                                ui.label(format!("Current Turn: {}", current_player));
                                
                                if self.ai_thinking || self.ai_pending_move.is_some() {
                                    ui.label("AI is thinking...");
                                }
                            } else {
                                let current_player = if self.is_black { "Black" } else { "White" };
                                ui.label(format!("Current Turn: {}", current_player));
                            }
                        });
                        
                        self.render_board(ui);
                        self.render_piece(ui);

                        if self.is_winner {
                            let text = if self.game_mode == GameMode::PlayerVsAI {
                                if self.is_black {
                                    if self.player_is_black { "Player Wins!" } else { "AI Wins!" }
                                } else {
                                    if self.player_is_black { "AI Wins!" } else { "Player Wins!" }
                                }
                            } else {
                                if self.is_black { "Black Wins!" } else { "White Wins!" }
                            };
                            egui::Window::new(text)
                                .collapsible(false)
                                .resizable(false)
                                .show(ctx, |ui| {
                                    ui.vertical_centered(|ui| {
                                        if ui.button("Restart").clicked() {
                                            self.restart();
                                        }
                                        if ui.button("Back to Menu").clicked() {
                                            self.game_mode = GameMode::MainMenu;
                                        }
                                    });
                                });
                            return;
                        }

                        // 监听点击事件
                        if let Some(pos) = ctx.input(|i| i.pointer.press_origin()) {
                            self.handle_click(pos);
                        }
                    });
                
                // 在AI模式下，玩家落子后调用AI逻辑
                if self.game_mode == GameMode::PlayerVsAI && !self.is_winner {
                    self.ai_move(delta_time);
                }
            }
        }
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(450.0, 450.0)),
        resizable: false,
        ..Default::default()
    };
    eframe::run_native("Gomoku", options, Box::new(|cc| Box::new(AppUI::new(cc)))).unwrap();
}
