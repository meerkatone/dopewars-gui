fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        follow_system_theme: true,
        default_theme: eframe::Theme::Dark,
        vsync: true,
        ..Default::default()
    };
    eframe::run_native(
        "DopeWars GUI",
        options,
        Box::new(|_cc| Box::new(DopeWarsApp::default())),
    )
}
use eframe::egui;
use egui::{Color32, RichText, Ui};
use rand::Rng;
use std::collections::HashMap;

const LOCATIONS: [&str; 6] = [
    "Bronx",
    "Brooklyn",
    "Manhattan",
    "Queens",
    "Staten Island",
    "Central Park",
];

const SUBSTANCES: [&str; 6] = ["Weed", "Cocaine", "Ludes", "Acid", "Heroin", "Speed"];

const MAX_DAYS: i32 = 30;
const STARTING_CASH: i32 = 2000;
const MAX_CARRYING_CAPACITY: i32 = 100;
const LOAN_SHARK_INTEREST: f32 = 0.1; // 10% interest per day

struct Player {
    cash: i32,
    debt: i32,
    inventory: HashMap<String, i32>,
    current_location: String,
    day: i32,
    health: i32,
}

impl Player {
    fn new() -> Self {
        let mut inventory = HashMap::new();
        for substance in SUBSTANCES.iter() {
            inventory.insert(substance.to_string(), 0);
        }

        Player {
            cash: STARTING_CASH,
            debt: 5000,
            inventory,
            current_location: LOCATIONS[0].to_string(),
            day: 1,
            health: 100,
        }
    }

    fn total_items(&self) -> i32 {
        self.inventory.values().sum()
    }

    fn space_available(&self) -> i32 {
        MAX_CARRYING_CAPACITY - self.total_items()
    }
}

struct Market {
    prices: HashMap<String, i32>,
    events: Vec<String>,
    price_history: HashMap<String, Vec<i32>>,
    max_history: usize,
}

impl Market {
    fn new() -> Self {
        let mut price_history = HashMap::new();
        // Initialize price history for each substance
        for substance in SUBSTANCES.iter() {
            price_history.insert(substance.to_string(), Vec::new());
        }

        Market {
            prices: HashMap::new(),
            events: Vec::new(),
            price_history,
            max_history: 10, // Keep track of the last 10 prices
        }
    }

    fn generate_prices(&mut self) {
        self.events.clear();
        let mut rng = rand::thread_rng();

        for substance in SUBSTANCES.iter() {
            let base_price = match *substance {
                "Weed" => rng.gen_range(10..100),
                "Cocaine" => rng.gen_range(100..1000),
                "Ludes" => rng.gen_range(20..200),
                "Acid" => rng.gen_range(50..400),
                "Heroin" => rng.gen_range(150..1500),
                "Speed" => rng.gen_range(50..700),
                _ => 0,
            };

            // Random event: sometimes a substance's price spikes or crashes
            let special_event = rng.gen_range(0..20);
            let price = match special_event {
                0 => {
                    let message = format!(
                        "📰 Breaking news! Police busted a {} shipment! Prices skyrocketing!",
                        substance
                    );
                    self.events.push(message);
                    base_price * 5
                }
                1 => {
                    let message =
                        format!("📰 Market flooded with {}! Prices have crashed!", substance);
                    self.events.push(message);
                    base_price / 5
                }
                _ => base_price,
            };

            self.prices.insert(substance.to_string(), price);

            // Update price history
            if let Some(history) = self.price_history.get_mut(&substance.to_string()) {
                history.push(price);
                // Keep only the most recent prices based on max_history
                if history.len() > self.max_history {
                    history.remove(0);
                }
            }
        }
    }
}

enum GameState {
    Running,
    GameOver(String),
}

enum Screen {
    Main,
    Buy,
    Sell,
    Travel,
    LoanShark,
    Hospital,
    Settings,
    PriceChart,
}

struct DopeWarsApp {
    player: Player,
    market: Market,
    game_state: GameState,
    current_screen: Screen,
    message_log: Vec<String>,
    selected_substance: Option<String>,
    amount_to_trade: String,
    amount_to_borrow_repay: String,
    ui_scale: f32,
    font_size: f32,
    selected_chart_substance: Option<String>,
    restart_requested: bool,
}

impl Default for DopeWarsApp {
    fn default() -> Self {
        let mut app = Self {
            player: Player::new(),
            market: Market::new(),
            game_state: GameState::Running,
            current_screen: Screen::Main,
            message_log: Vec::new(),
            selected_substance: None,
            amount_to_trade: "0".to_string(),
            amount_to_borrow_repay: "0".to_string(),
            ui_scale: 1.0,
            font_size: 14.0,
            selected_chart_substance: Some("Weed".to_string()), // Default to first substance
            restart_requested: false,
        };

        app.message_log
            .push("Welcome to DopeWars GUI Edition!".to_string());
        app.message_log
            .push("You have 30 days to make as much money as possible.".to_string());
        app.message_log
            .push("Buy low, sell high, and watch out for the cops!".to_string());

        // Generate initial market prices
        app.market.generate_prices();

        // Add market events to message log
        for event in &app.market.events {
            app.message_log.push(event.clone());
        }

        app
    }
}

impl eframe::App for DopeWarsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply scaling
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::proportional(self.font_size * 1.5),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::proportional(self.font_size),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::monospace(self.font_size),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::proportional(self.font_size),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::proportional(self.font_size * 0.8),
            ),
        ]
        .into();
        ctx.set_style(style);
        ctx.set_pixels_per_point(self.ui_scale);

        // Check if restart was requested
        if self.restart_requested {
            // Reset player
            self.player = Player::new();

            // Reset market
            self.market = Market::new();
            self.market.generate_prices();

            // Reset game state
            self.game_state = GameState::Running;
            self.current_screen = Screen::Main;

            // Reset interface state
            self.selected_substance = None;
            self.amount_to_trade = "0".to_string();
            self.amount_to_borrow_repay = "0".to_string();

            // Reset message log but keep intro messages
            self.message_log.clear();
            self.message_log
                .push("Welcome to DopeWars GUI Edition!".to_string());
            self.message_log
                .push("You have 30 days to make as much money as possible.".to_string());
            self.message_log
                .push("Buy low, sell high, and watch out for the cops!".to_string());
            self.message_log.push("Game restarted!".to_string());

            // Add market events to message log
            for event in &self.market.events {
                self.message_log.push(event.clone());
            }

            // Reset restart flag
            self.restart_requested = false;
        }

        match self.game_state {
            GameState::Running => {
                egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                    self.render_top_panel(ui);
                });

                egui::SidePanel::left("side_panel").show(ctx, |ui| {
                    self.render_side_panel(ui);
                });

                egui::CentralPanel::default().show(ctx, |ui| match self.current_screen {
                    Screen::Main => self.render_main_screen(ui),
                    Screen::Buy => self.render_buy_screen(ui),
                    Screen::Sell => self.render_sell_screen(ui),
                    Screen::Travel => self.render_travel_screen(ui),
                    Screen::LoanShark => self.render_loan_shark_screen(ui),
                    Screen::Hospital => self.render_hospital_screen(ui),
                    Screen::Settings => self.render_settings_screen(ui),
                    Screen::PriceChart => self.render_price_chart_screen(ui),
                });

                egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                    self.render_message_log(ui);
                });
            }
            GameState::GameOver(ref reason) => {
                // Create a local copy of the reason to avoid borrow issues
                let reason_copy = reason.clone();

                egui::CentralPanel::default().show(ctx, |ui| {
                    self.render_game_over(ui, &reason_copy);
                });
            }
        }

        // Check for game over conditions
        if let GameState::Running = self.game_state {
            if self.player.day > MAX_DAYS {
                self.game_state =
                    GameState::GameOver("Time's up! Your 30 days are over.".to_string());
            } else if self.player.health <= 0 {
                self.game_state = GameState::GameOver("You died from your injuries!".to_string());
            }
        }
    }
}

impl DopeWarsApp {
    fn render_top_panel(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("DopeWars GUI");
            ui.add_space(20.0);
            ui.label(format!("Day: {}/{}", self.player.day, MAX_DAYS));
            ui.add_space(10.0);
            ui.label(format!("Cash: ${}", self.player.cash));
            ui.add_space(10.0);
            ui.label(format!("Debt: ${}", self.player.debt));
            ui.add_space(10.0);
            ui.label(format!("Location: {}", self.player.current_location));
            ui.add_space(10.0);

            let health_text = if self.player.health < 30 {
                RichText::new(format!("Health: {}", self.player.health)).color(Color32::RED)
            } else if self.player.health < 70 {
                RichText::new(format!("Health: {}", self.player.health)).color(Color32::YELLOW)
            } else {
                RichText::new(format!("Health: {}", self.player.health)).color(Color32::GREEN)
            };

            ui.label(health_text);
        });
    }

    fn render_side_panel(&mut self, ui: &mut Ui) {
        ui.heading("Navigation");

        if ui.button("Main Menu").clicked() {
            self.current_screen = Screen::Main;
        }

        if ui.button("Buy").clicked() {
            self.current_screen = Screen::Buy;
            self.selected_substance = None;
            self.amount_to_trade = "0".to_string();
        }

        if ui.button("Sell").clicked() {
            self.current_screen = Screen::Sell;
            self.selected_substance = None;
            self.amount_to_trade = "0".to_string();
        }

        if ui.button("Travel").clicked() {
            self.current_screen = Screen::Travel;
        }

        if ui.button("Loan Shark").clicked() {
            self.current_screen = Screen::LoanShark;
            self.amount_to_borrow_repay = "0".to_string();
        }

        if ui.button("Hospital").clicked() {
            self.current_screen = Screen::Hospital;
        }

        if ui.button("Price Charts").clicked() {
            self.current_screen = Screen::PriceChart;
        }

        if ui.button("Settings").clicked() {
            self.current_screen = Screen::Settings;
        }

        ui.separator();

        ui.heading("Inventory");
        ui.label(format!(
            "Space: {}/{}",
            self.player.total_items(),
            MAX_CARRYING_CAPACITY
        ));

        for (substance, amount) in &self.player.inventory {
            if *amount > 0 {
                ui.label(format!("{}: {}", substance, amount));
            }
        }

        ui.separator();

        ui.heading("Market Prices");
        for substance in SUBSTANCES.iter() {
            let price = self.market.prices.get(&substance.to_string()).unwrap_or(&0);
            ui.label(format!("{}: ${}", substance, price));
        }
    }

    fn render_main_screen(&mut self, ui: &mut Ui) {
        ui.heading("DopeWars - Main Menu");
        ui.label("What do you want to do?");

        ui.separator();

        if ui.button("Buy substances").clicked() {
            self.current_screen = Screen::Buy;
            self.selected_substance = None;
            self.amount_to_trade = "0".to_string();
        }

        if ui.button("Sell substances").clicked() {
            self.current_screen = Screen::Sell;
            self.selected_substance = None;
            self.amount_to_trade = "0".to_string();
        }

        if ui.button("Travel to another location").clicked() {
            self.current_screen = Screen::Travel;
        }

        if ui.button("Visit loan shark").clicked() {
            self.current_screen = Screen::LoanShark;
            self.amount_to_borrow_repay = "0".to_string();
        }

        if ui.button("Go to hospital").clicked() {
            self.current_screen = Screen::Hospital;
        }

        if ui.button("View Price Charts").clicked() {
            self.current_screen = Screen::PriceChart;
        }

        if ui.button("Settings").clicked() {
            self.current_screen = Screen::Settings;
        }
    }

    fn render_buy_screen(&mut self, ui: &mut Ui) {
        ui.heading("Buy Substances");
        ui.label(format!("Cash: ${}", self.player.cash));
        ui.label(format!(
            "Space available: {}",
            self.player.space_available()
        ));

        ui.separator();

        for substance in SUBSTANCES.iter() {
            let substance_str = substance.to_string();
            let price = self.market.prices.get(&substance_str).unwrap_or(&0);

            let is_selected = match &self.selected_substance {
                Some(selected) => selected == &substance_str,
                None => false,
            };

            if ui
                .selectable_label(is_selected, format!("{} - ${} per unit", substance, price))
                .clicked()
            {
                self.selected_substance = Some(substance_str.clone());
                self.amount_to_trade = "0".to_string();
            }
        }

        ui.separator();

        if let Some(ref substance) = self.selected_substance {
            let price = self.market.prices.get(substance).unwrap_or(&0);

            ui.label(format!("Selected: {} - ${} per unit", substance, price));

            ui.horizontal(|ui| {
                ui.label("Amount to buy:");
                ui.text_edit_singleline(&mut self.amount_to_trade);

                if ui.button("Max").clicked() {
                    let max_affordable = self.player.cash / price;
                    let max_space = self.player.space_available();
                    let max_amount = std::cmp::min(max_affordable, max_space);
                    self.amount_to_trade = max_amount.to_string();
                }
            });

            let amount: i32 = match self.amount_to_trade.parse() {
                Ok(num) => num,
                Err(_) => 0,
            };

            let total_cost = amount * price;
            ui.label(format!("Total cost: ${}", total_cost));

            let can_buy = amount > 0
                && amount <= self.player.space_available()
                && total_cost <= self.player.cash;

            if !can_buy {
                let mut reason = "Cannot buy: ".to_string();
                if amount <= 0 {
                    reason.push_str("Amount must be positive. ");
                }
                if amount > self.player.space_available() {
                    reason.push_str("Not enough space. ");
                }
                if total_cost > self.player.cash {
                    reason.push_str("Not enough cash. ");
                }
                ui.label(RichText::new(reason).color(Color32::RED));
            }

            if ui.add_enabled(can_buy, egui::Button::new("Buy")).clicked() {
                self.player.cash -= total_cost;
                *self.player.inventory.entry(substance.clone()).or_insert(0) += amount;

                let message = format!(
                    "Bought {} units of {} for ${}",
                    amount, substance, total_cost
                );
                self.message_log.push(message);

                self.selected_substance = None;
                self.amount_to_trade = "0".to_string();
            }
        } else {
            ui.label("Select a substance to buy");
        }
    }

    fn render_sell_screen(&mut self, ui: &mut Ui) {
        ui.heading("Sell Substances");
        ui.label(format!("Cash: ${}", self.player.cash));

        ui.separator();

        let mut has_inventory = false;

        for substance in SUBSTANCES.iter() {
            let substance_str = substance.to_string();
            let price = self.market.prices.get(&substance_str).unwrap_or(&0);
            let amount = self.player.inventory.get(&substance_str).unwrap_or(&0);

            if *amount > 0 {
                has_inventory = true;

                let is_selected = match &self.selected_substance {
                    Some(selected) => selected == &substance_str,
                    None => false,
                };

                if ui
                    .selectable_label(
                        is_selected,
                        format!("{} - {} units - ${} per unit", substance, amount, price),
                    )
                    .clicked()
                {
                    self.selected_substance = Some(substance_str.clone());
                    self.amount_to_trade = "0".to_string();
                }
            }
        }

        if !has_inventory {
            ui.label("You don't have any substances to sell.");
        }

        ui.separator();

        if let Some(ref substance) = self.selected_substance {
            let price = self.market.prices.get(substance).unwrap_or(&0);
            let current_amount = self.player.inventory.get(substance).unwrap_or(&0);

            ui.label(format!("Selected: {} - ${} per unit", substance, price));

            ui.horizontal(|ui| {
                ui.label("Amount to sell:");
                ui.text_edit_singleline(&mut self.amount_to_trade);

                if ui.button("Max").clicked() {
                    self.amount_to_trade = current_amount.to_string();
                }
            });

            let amount: i32 = match self.amount_to_trade.parse() {
                Ok(num) => num,
                Err(_) => 0,
            };

            let total_earned = amount * price;
            ui.label(format!("Total earned: ${}", total_earned));

            let can_sell = amount > 0 && amount <= *current_amount;

            if !can_sell {
                let mut reason = "Cannot sell: ".to_string();
                if amount <= 0 {
                    reason.push_str("Amount must be positive. ");
                }
                if amount > *current_amount {
                    reason.push_str("Not enough inventory. ");
                }
                ui.label(RichText::new(reason).color(Color32::RED));
            }

            if ui
                .add_enabled(can_sell, egui::Button::new("Sell"))
                .clicked()
            {
                self.player.cash += total_earned;
                *self.player.inventory.get_mut(substance).unwrap() -= amount;

                let message = format!(
                    "Sold {} units of {} for ${}",
                    amount, substance, total_earned
                );
                self.message_log.push(message);

                self.selected_substance = None;
                self.amount_to_trade = "0".to_string();
            }
        } else if has_inventory {
            ui.label("Select a substance to sell");
        }
    }

    fn render_travel_screen(&mut self, ui: &mut Ui) {
        ui.heading("Travel");
        ui.label("Where do you want to go?");

        ui.separator();

        for location in LOCATIONS.iter() {
            if *location != self.player.current_location {
                if ui.button(*location).clicked() {
                    self.travel_to(location);
                }
            }
        }
    }

    fn travel_to(&mut self, new_location: &str) {
        let mut rng = rand::thread_rng();
        let random_event = rng.gen_range(0..10);

        self.message_log
            .push(format!("Traveling to {}...", new_location));

        match random_event {
            0 => {
                let message =
                    "🚨 You were stopped by cops! They confiscated some of your stuff!".to_string();
                self.message_log.push(message);

                let substance_to_confiscate =
                    SUBSTANCES[rng.gen_range(0..SUBSTANCES.len())].to_string();

                if let Some(amount) = self.player.inventory.get_mut(&substance_to_confiscate) {
                    if *amount > 0 {
                        let confiscated = rng.gen_range(1..=*amount);
                        *amount -= confiscated;
                        let message = format!(
                            "They took {} units of {}",
                            confiscated, substance_to_confiscate
                        );
                        self.message_log.push(message);
                    }
                }
            }
            1 => {
                let message = "🔫 You were mugged! You lost some cash.".to_string();
                self.message_log.push(message);

                let lost = rng.gen_range(100..500).min(self.player.cash);
                self.player.cash -= lost;

                let message = format!("You lost ${}", lost);
                self.message_log.push(message);
            }
            2 => {
                let message = "🏥 You got injured during travel!".to_string();
                self.message_log.push(message);

                let damage = rng.gen_range(5..20);
                self.player.health -= damage;

                let message = format!("You lost {} health points", damage);
                self.message_log.push(message);

                if self.player.health <= 0 {
                    self.player.health = 0;
                    self.message_log
                        .push("You're severely injured and need medical attention!".to_string());
                }
            }
            _ => {
                self.message_log.push("Journey was uneventful.".to_string());
            }
        }

        self.player.current_location = new_location.to_string();
        self.player.day += 1;
        self.player.debt = (self.player.debt as f32 * (1.0 + LOAN_SHARK_INTEREST)) as i32;

        let message = format!("You've arrived at {}.", new_location);
        self.message_log.push(message);

        let message = format!(
            "Your debt has increased to ${} due to interest.",
            self.player.debt
        );
        self.message_log.push(message);

        // Generate new market prices for the new location
        self.market.generate_prices();

        // Add market events to message log
        for event in &self.market.events {
            self.message_log.push(event.clone());
        }

        // Return to main screen
        self.current_screen = Screen::Main;
    }

    fn render_loan_shark_screen(&mut self, ui: &mut Ui) {
        ui.heading("Loan Shark");
        ui.label(format!("Current debt: ${}", self.player.debt));
        ui.label(format!("Current cash: ${}", self.player.cash));

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Amount:");
            ui.text_edit_singleline(&mut self.amount_to_borrow_repay);
        });

        let amount: i32 = match self.amount_to_borrow_repay.parse() {
            Ok(num) => num,
            Err(_) => 0,
        };

        let can_borrow = amount > 0;
        let can_repay = amount > 0 && amount <= self.player.cash && amount <= self.player.debt;

        ui.horizontal(|ui| {
            if ui
                .add_enabled(can_borrow, egui::Button::new("Borrow"))
                .clicked()
            {
                self.player.cash += amount;
                self.player.debt += amount;

                let message = format!(
                    "You borrowed ${}, your debt is now ${}",
                    amount, self.player.debt
                );
                self.message_log.push(message);

                self.amount_to_borrow_repay = "0".to_string();
            }

            if ui
                .add_enabled(can_repay, egui::Button::new("Repay"))
                .clicked()
            {
                self.player.cash -= amount;
                self.player.debt -= amount;

                let message = format!(
                    "You repaid ${}, your debt is now ${}",
                    amount, self.player.debt
                );
                self.message_log.push(message);

                self.amount_to_borrow_repay = "0".to_string();
            }
        });

        if !can_borrow {
            ui.label(RichText::new("To borrow, enter a positive amount.").color(Color32::RED));
        }

        if !can_repay {
            let mut reason = "Cannot repay: ".to_string();
            if amount <= 0 {
                reason.push_str("Amount must be positive. ");
            }
            if amount > self.player.cash {
                reason.push_str("Not enough cash. ");
            }
            if amount > self.player.debt {
                reason.push_str("Amount exceeds debt. ");
            }
            ui.label(RichText::new(reason).color(Color32::RED));
        }

        ui.separator();
        ui.label("Daily interest rate: 10%");
        ui.label("Your debt increases every time you travel to a new location.");
    }

    fn render_hospital_screen(&mut self, ui: &mut Ui) {
        ui.heading("Hospital");
        ui.label(format!("Your health: {}/100", self.player.health));

        if self.player.health >= 100 {
            ui.label("You're in perfect health!");
            return;
        }

        let healing_needed = 100 - self.player.health;
        let cost_per_point = 50;
        let total_cost = healing_needed * cost_per_point;

        ui.label(format!(
            "It will cost ${} to fully heal ({} health points at ${} each).",
            total_cost, healing_needed, cost_per_point
        ));

        let can_heal = self.player.cash >= total_cost;

        if !can_heal {
            ui.label(
                RichText::new("You don't have enough cash for treatment.").color(Color32::RED),
            );
        }

        if ui
            .add_enabled(can_heal, egui::Button::new("Get treatment"))
            .clicked()
        {
            self.player.cash -= total_cost;
            self.player.health = 100;

            let message = "You've been treated and are now at full health!".to_string();
            self.message_log.push(message);
        }
    }

    fn render_message_log(&self, ui: &mut Ui) {
        ui.heading("Message Log");

        egui::ScrollArea::vertical()
            .max_height(150.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in &self.message_log {
                    ui.label(message);
                }
            });
    }

    fn render_settings_screen(&mut self, ui: &mut Ui) {
        ui.heading("Settings");

        ui.add_space(20.0);

        ui.label("UI Scale");
        let mut scale = self.ui_scale;
        ui.add(egui::Slider::new(&mut scale, 0.5..=3.0).text("Scale factor"));
        if scale != self.ui_scale {
            self.ui_scale = scale;
        }

        ui.add_space(10.0);

        ui.label("Font Size");
        let mut font_size = self.font_size;
        ui.add(egui::Slider::new(&mut font_size, 8.0..=32.0).text("Size in points"));
        if font_size != self.font_size {
            self.font_size = font_size;
        }

        ui.add_space(20.0);

        // Quick presets
        ui.label("Presets:");
        ui.horizontal(|ui| {
            if ui.button("Small").clicked() {
                self.ui_scale = 0.8;
                self.font_size = 12.0;
            }

            if ui.button("Normal").clicked() {
                self.ui_scale = 1.0;
                self.font_size = 14.0;
            }

            if ui.button("Large").clicked() {
                self.ui_scale = 1.2;
                self.font_size = 18.0;
            }

            if ui.button("Extra Large").clicked() {
                self.ui_scale = 1.5;
                self.font_size = 24.0;
            }
        });

        ui.add_space(20.0);

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if ui.button("Return to Main Menu").clicked() {
                self.current_screen = Screen::Main;
            }
        });
    }

    fn render_price_chart_screen(&mut self, ui: &mut Ui) {
        ui.heading("Price Charts");

        ui.horizontal(|ui| {
            ui.label("Select substance:");

            for substance in SUBSTANCES.iter() {
                let substance_str = substance.to_string();
                let is_selected = match &self.selected_chart_substance {
                    Some(selected) => selected == &substance_str,
                    None => false,
                };

                if ui.selectable_label(is_selected, *substance).clicked() {
                    self.selected_chart_substance = Some(substance_str.clone());
                }
            }
        });

        if let Some(ref substance) = self.selected_chart_substance {
            let current_price = self.market.prices.get(substance).unwrap_or(&0);
            ui.label(format!(
                "Current price of {}: ${}",
                substance, current_price
            ));

            if let Some(price_history) = self.market.price_history.get(substance) {
                if price_history.is_empty() {
                    ui.label("No price history available yet. Travel to generate price data.");
                } else {
                    let chart_height = 200.0;
                    let text_height = 30.0;
                    let padding = 10.0;

                    // Find min and max values in history for scaling
                    let max_price = price_history
                        .iter()
                        .copied()
                        .max()
                        .unwrap_or(0)
                        .max(*current_price);
                    let min_price = price_history
                        .iter()
                        .copied()
                        .min()
                        .unwrap_or(0)
                        .min(*current_price);
                    let price_range = (max_price - min_price).max(1); // Avoid division by zero

                    let painter = ui.painter();
                    let available_rect = ui.available_rect_before_wrap();
                    let chart_rect = egui::Rect::from_min_size(
                        available_rect.min,
                        egui::vec2(
                            available_rect.width(),
                            chart_height + text_height + 2.0 * padding,
                        ),
                    );

                    let inner_rect = egui::Rect::from_min_size(
                        chart_rect.min + egui::vec2(padding, padding),
                        chart_rect.size() - egui::vec2(2.0 * padding, 2.0 * padding),
                    );

                    // Draw background
                    painter.rect_filled(chart_rect, 0.0, Color32::from_rgb(30, 30, 30));

                    // Draw borders
                    painter.rect_stroke(inner_rect, 0.0, egui::Stroke::new(1.0, Color32::GRAY));

                    // Draw horizontal gridlines (5 lines)
                    let grid_color = Color32::from_rgb(60, 60, 60);
                    for i in 0..=4 {
                        let y = inner_rect.min.y
                            + (i as f32) * (inner_rect.height() - text_height) / 4.0;
                        painter.line_segment(
                            [
                                egui::pos2(inner_rect.min.x, y),
                                egui::pos2(inner_rect.max.x, y),
                            ],
                            egui::Stroke::new(1.0, grid_color),
                        );

                        // Draw price labels
                        let price = max_price - (i as i32) * price_range / 4;
                        painter.text(
                            egui::pos2(inner_rect.min.x + 5.0, y - 10.0),
                            egui::Align2::LEFT_CENTER,
                            format!("${}", price),
                            egui::FontId::proportional(12.0),
                            Color32::LIGHT_GRAY,
                        );
                    }

                    // Combine current price with history for plotting
                    let all_prices: Vec<i32> = price_history
                        .iter()
                        .chain(std::iter::once(current_price))
                        .copied()
                        .collect();

                    // Need at least 2 points to draw a line
                    if all_prices.len() >= 2 {
                        let points_count = all_prices.len();
                        let x_step = inner_rect.width() / (points_count as f32 - 1.0);

                        // Draw day labels
                        for i in 0..points_count {
                            let x = inner_rect.min.x + (i as f32) * x_step;
                            let day_text = if i == points_count - 1 {
                                "Now".to_string()
                            } else {
                                format!("Day {}", self.player.day - (points_count - i - 1) as i32)
                            };

                            // Draw vertical gridlines
                            painter.line_segment(
                                [
                                    egui::pos2(x, inner_rect.min.y),
                                    egui::pos2(x, inner_rect.max.y - text_height),
                                ],
                                egui::Stroke::new(1.0, grid_color),
                            );

                            // Draw day labels
                            painter.text(
                                egui::pos2(x, inner_rect.max.y - text_height / 2.0),
                                egui::Align2::CENTER_CENTER,
                                day_text,
                                egui::FontId::proportional(10.0),
                                Color32::LIGHT_GRAY,
                            );
                        }

                        // Draw price line
                        let mut points = Vec::with_capacity(points_count);
                        for (i, &price) in all_prices.iter().enumerate() {
                            let x = inner_rect.min.x + (i as f32) * x_step;
                            // Normalize price to chart height
                            let normalized_price = if price_range > 0 {
                                (price - min_price) as f32 / price_range as f32
                            } else {
                                0.5 // Default to middle if there's no range
                            };

                            let y = inner_rect.max.y
                                - text_height
                                - normalized_price * (inner_rect.height() - text_height);
                            points.push(egui::pos2(x, y));
                        }

                        // Draw connecting lines
                        for i in 0..points.len() - 1 {
                            painter.line_segment(
                                [points[i], points[i + 1]],
                                egui::Stroke::new(2.0, Color32::from_rgb(100, 200, 100)),
                            );
                        }

                        // Draw points
                        for point in &points {
                            painter.circle_filled(*point, 4.0, Color32::from_rgb(100, 200, 100));
                        }

                        // Highlight current price
                        painter.circle_filled(
                            points[points.len() - 1],
                            6.0,
                            Color32::from_rgb(200, 100, 100),
                        );
                    }

                    // Reserve space for the chart
                    ui.allocate_rect(chart_rect, egui::Sense::hover());
                }
            }

            // Trend analysis if we have history
            if let Some(price_history) = self.market.price_history.get(substance) {
                if price_history.len() >= 2 {
                    ui.add_space(10.0);

                    // Calculate price trend
                    let first_price = price_history.first().unwrap_or(&0);
                    let last_recorded_price = price_history.last().unwrap_or(&0);

                    // Calculate percentage change
                    let trend_percentage = if *first_price > 0 {
                        ((last_recorded_price - first_price) as f32 / *first_price as f32) * 100.0
                    } else {
                        0.0
                    };

                    let trend_text = if trend_percentage > 15.0 {
                        RichText::new(format!("📈 Strong upward trend: {:.1}%", trend_percentage))
                            .color(Color32::from_rgb(0, 255, 0))
                    } else if trend_percentage > 5.0 {
                        RichText::new(format!("📈 Upward trend: {:.1}%", trend_percentage))
                            .color(Color32::from_rgb(100, 255, 100))
                    } else if trend_percentage < -15.0 {
                        RichText::new(format!(
                            "📉 Strong downward trend: {:.1}%",
                            trend_percentage
                        ))
                        .color(Color32::from_rgb(255, 0, 0))
                    } else if trend_percentage < -5.0 {
                        RichText::new(format!("📉 Downward trend: {:.1}%", trend_percentage))
                            .color(Color32::from_rgb(255, 100, 100))
                    } else {
                        RichText::new(format!("➡️ Stable price: {:.1}%", trend_percentage))
                            .color(Color32::LIGHT_GRAY)
                    };

                    ui.label("Market Analysis:");
                    ui.label(trend_text);

                    // Trading advice
                    ui.add_space(5.0);
                    ui.label("Trading Recommendation:");

                    let advice = if trend_percentage > 10.0 {
                        RichText::new("Consider selling - prices are high and may drop soon.")
                            .color(Color32::from_rgb(255, 200, 0))
                    } else if trend_percentage < -10.0 {
                        RichText::new("Good time to buy - prices are low and may rise soon.")
                            .color(Color32::from_rgb(0, 255, 200))
                    } else {
                        RichText::new("Market is stable - no strong buy/sell signals.")
                            .color(Color32::WHITE)
                    };

                    ui.label(advice);
                }
            }
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if ui.button("Return to Main Menu").clicked() {
                self.current_screen = Screen::Main;
            }
        });
    }

    fn render_game_over(&mut self, ui: &mut Ui, reason: &str) {
        ui.vertical_centered(|ui| {
            ui.heading("GAME OVER");
            ui.label(reason);

            ui.add_space(20.0);

            ui.label("Final Stats:");
            ui.label(format!("Days survived: {}", self.player.day));
            ui.label(format!("Final cash: ${}", self.player.cash));
            ui.label(format!("Final debt: ${}", self.player.debt));
            ui.label(format!(
                "Net worth: ${}",
                self.player.cash - self.player.debt
            ));

            ui.add_space(10.0);

            ui.label("Final inventory:");
            for (substance, amount) in &self.player.inventory {
                if *amount > 0 {
                    ui.label(format!("{} - {} units", substance, amount));
                }
            }

            ui.add_space(20.0);

            if ui.button("Play Again").clicked() {
                self.restart_requested = true;
            }
        });
    }
}
