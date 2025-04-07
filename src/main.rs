use eframe::egui;
use egui::{Color32, Response, RichText, Ui};
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

const WEAPONS: [&str; 5] = ["Knife", "Pistol", "Shotgun", "Assault Rifle", "Grenade"];

const WEAPON_PRICES: [i32; 5] = [500, 1500, 3000, 6000, 4000];

const WEAPON_POWER: [i32; 5] = [20, 40, 65, 85, 95];

const MAX_DAYS: i32 = 30;
const STARTING_CASH: i32 = 2000;
const MAX_CARRYING_CAPACITY: i32 = 100;
const LOAN_SHARK_INTEREST: f32 = 0.1; // 10% interest per day
const STASH_HOUSE_CAPACITY: i32 = 200;
const STASH_HOUSE_BASE_PRICE: i32 = 10000;

// Define a struct for stash houses
struct StashHouse {
    location: String,
    inventory: HashMap<String, i32>,
    capacity: i32,
}

impl StashHouse {
    fn new(location: String) -> Self {
        let mut inventory = HashMap::new();
        for substance in SUBSTANCES.iter() {
            inventory.insert(substance.to_string(), 0);
        }

        StashHouse {
            location,
            inventory,
            capacity: STASH_HOUSE_CAPACITY,
        }
    }

    fn total_items(&self) -> i32 {
        self.inventory.values().sum()
    }

    fn space_available(&self) -> i32 {
        self.capacity - self.total_items()
    }
}

struct Player {
    cash: i32,
    debt: i32,
    inventory: HashMap<String, i32>,
    current_location: String,
    day: i32,
    health: i32,
    weapons: HashMap<String, i32>,
    active_weapon: Option<String>,
    stash_houses: HashMap<String, StashHouse>,
}

impl Player {
    fn new() -> Self {
        let mut inventory = HashMap::new();
        for substance in SUBSTANCES.iter() {
            inventory.insert(substance.to_string(), 0);
        }

        let mut weapons = HashMap::new();
        for weapon in WEAPONS.iter() {
            weapons.insert(weapon.to_string(), 0);
        }

        Player {
            cash: STARTING_CASH,
            debt: 5000,
            inventory,
            current_location: LOCATIONS[0].to_string(),
            day: 1,
            health: 100,
            weapons,
            active_weapon: None,
            stash_houses: HashMap::new(),
        }
    }

    fn total_items(&self) -> i32 {
        self.inventory.values().sum()
    }

    fn space_available(&self) -> i32 {
        MAX_CARRYING_CAPACITY - self.total_items()
    }

    fn has_weapon(&self) -> bool {
        self.weapons.values().sum::<i32>() > 0
    }

    fn get_active_weapon_power(&self) -> i32 {
        if let Some(weapon) = &self.active_weapon {
            // Find the weapon index
            for (i, w) in WEAPONS.iter().enumerate() {
                if *w == *weapon {
                    return WEAPON_POWER[i];
                }
            }
        }
        0 // No weapon or weapon not found
    }

    fn owns_stash_house_in_current_location(&self) -> bool {
        self.stash_houses.contains_key(&self.current_location)
    }

    fn get_stash_house_price(&self, location: &str) -> i32 {
        // Prices vary by location
        match location {
            "Manhattan" => STASH_HOUSE_BASE_PRICE * 2, // Most expensive
            "Bronx" | "Brooklyn" => STASH_HOUSE_BASE_PRICE,
            "Queens" => STASH_HOUSE_BASE_PRICE * 3 / 4, // 75% of base price
            "Staten Island" => STASH_HOUSE_BASE_PRICE / 2, // Half price
            "Central Park" => STASH_HOUSE_BASE_PRICE * 4, // Very expensive, high risk
            _ => STASH_HOUSE_BASE_PRICE,
        }
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

        // Chance for global market event
        let global_event = rng.gen_range(0..20);
        let global_market_modifier = match global_event {
            0 => {
                let message = "📰 GLOBAL EVENT: International drug enforcement operation! All prices increased!".to_string();
                self.events.push(message);
                1.5 // Prices up 50%
            }
            1 => {
                let message = "📰 GLOBAL EVENT: Major drug cartel busted! Supply chains disrupted!"
                    .to_string();
                self.events.push(message);
                2.0 // Prices doubled
            }
            2 => {
                let message = "📰 GLOBAL EVENT: New synthetic alternatives flooding market! All prices decreased!".to_string();
                self.events.push(message);
                0.5 // Prices halved
            }
            3 => {
                let message = "📰 GLOBAL EVENT: Economic recession! Drug demand down!".to_string();
                self.events.push(message);
                0.7 // Prices 30% down
            }
            _ => 1.0, // No global effect
        };

        // Generate prices for each substance
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
                2 => {
                    let message = format!(
                        "📰 New high-quality {} on the market! Premium prices!",
                        substance
                    );
                    self.events.push(message);
                    base_price * 3
                }
                3 => {
                    let message = format!(
                        "📰 Contaminated {} discovered! Demand plummeting!",
                        substance
                    );
                    self.events.push(message);
                    base_price / 3
                }
                _ => base_price,
            };

            // Apply global modifier
            let final_price = (price as f32 * global_market_modifier) as i32;

            // Ensure price is always at least 1
            let final_price = std::cmp::max(final_price, 1);

            self.prices.insert(substance.to_string(), final_price);

            // Update price history
            if let Some(history) = self.price_history.get_mut(&substance.to_string()) {
                history.push(final_price);
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

#[derive(Clone)]
enum Screen {
    Main,
    Buy,
    Sell,
    Travel,
    LoanShark,
    Hospital,
    Settings,
    PriceChart,
    Weapons,
    PoliceEncounter,
    StashHouses,
    StashDeposit,
    StashWithdraw,
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
    selected_weapon: Option<String>,
    bribe_amount: String,
    stash_selected_substance: Option<String>,
    stash_amount_to_transfer: String,
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
            selected_weapon: None,
            bribe_amount: "0".to_string(),
            stash_selected_substance: None,
            stash_amount_to_transfer: "0".to_string(),
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
            self.selected_weapon = None;
            self.bribe_amount = "0".to_string();
            self.stash_selected_substance = None;
            self.stash_amount_to_transfer = "0".to_string();

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
                    Screen::Weapons => self.render_weapons_screen(ui),
                    Screen::PoliceEncounter => self.render_police_encounter(ui),
                    Screen::StashHouses => self.render_stash_houses_screen(ui),
                    Screen::StashDeposit => self.render_stash_deposit_screen(ui),
                    Screen::StashWithdraw => self.render_stash_withdraw_screen(ui),
                });

                egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                    self.render_message_log(ui);
                });
            }
            GameState::GameOver(ref reason) => {
                let reason_string = reason.clone(); // Clone to avoid borrowing issues
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.render_game_over(ui, &reason_string);
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

        if ui.button("Weapons Shop").clicked() {
            self.current_screen = Screen::Weapons;
            self.selected_weapon = None;
        }

        if ui.button("Stash Houses").clicked() {
            self.current_screen = Screen::StashHouses;
            self.stash_selected_substance = None;
            self.stash_amount_to_transfer = "0".to_string();
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

        ui.heading("Weapons");
        let has_weapons = self.player.weapons.values().sum::<i32>() > 0;

        if has_weapons {
            for (weapon, amount) in &self.player.weapons {
                if *amount > 0 {
                    let is_active = match &self.player.active_weapon {
                        Some(active) => active == weapon,
                        None => false,
                    };

                    let text = if is_active {
                        RichText::new(format!("{}: {} (EQUIPPED)", weapon, amount))
                            .color(Color32::GREEN)
                    } else {
                        RichText::new(format!("{}: {}", weapon, amount))
                    };

                    ui.label(text);
                }
            }
        } else {
            ui.label("No weapons");
        }

        ui.separator();

        ui.heading("Market Prices");
        for substance in SUBSTANCES.iter() {
            let price = self.market.prices.get(&substance.to_string()).unwrap_or(&0);
            ui.label(format!("{}: ${}", substance, price));
        }

        // Add stash houses section to sidebar if player owns any
        if !self.player.stash_houses.is_empty() {
            ui.separator();
            ui.heading("Your Stash Houses");

            for (location, stash) in &self.player.stash_houses {
                let is_current = *location == self.player.current_location;
                let text = if is_current {
                    RichText::new(format!(
                        "{}: {}/{}",
                        location,
                        stash.total_items(),
                        stash.capacity
                    ))
                    .color(Color32::GREEN)
                } else {
                    RichText::new(format!(
                        "{}: {}/{}",
                        location,
                        stash.total_items(),
                        stash.capacity
                    ))
                };

                ui.label(text);
            }
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

        if ui.button("Buy weapons").clicked() {
            self.current_screen = Screen::Weapons;
            self.selected_weapon = None;
        }

        if ui.button("Manage stash houses").clicked() {
            self.current_screen = Screen::StashHouses;
            self.stash_selected_substance = None;
            self.stash_amount_to_transfer = "0".to_string();
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
        let random_event = rng.gen_range(0..12); // Increased range for more event types

        self.message_log
            .push(format!("Traveling to {}...", new_location));

        match random_event {
            0 => {
                // Trigger the police encounter screen
                self.message_log
                    .push("🚨 You've been stopped by the police!".to_string());
                self.current_screen = Screen::PoliceEncounter;
                return; // Exit early - don't continue with the travel until the encounter is resolved
            }
            1 => {
                let message = "🔫 You were mugged! You lost some cash.".to_string();
                self.message_log.push(message);

                // Check if player has a weapon for defense
                if self.player.has_weapon() && self.player.active_weapon.is_some() {
                    let weapon_power = self.player.get_active_weapon_power();
                    let defend_chance = 30 + (weapon_power / 2);

                    if rng.gen_range(0..100) < defend_chance {
                        self.message_log.push(format!(
                            "You used your {} to fight off the muggers!",
                            self.player.active_weapon.as_ref().unwrap()
                        ));

                        // Still might get hurt
                        let damage = rng.gen_range(0..10);
                        if damage > 0 {
                            self.player.health -= damage;
                            self.message_log
                                .push(format!("You got hurt in the fight (-{} health)", damage));
                        }

                        return;
                    }
                }

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
            3 => {
                // Substance price crash
                let substance = SUBSTANCES[rng.gen_range(0..SUBSTANCES.len())].to_string();
                let message = format!("📰 Market flooded with {}! Prices have crashed!", substance);
                self.message_log.push(message);

                if let Some(price) = self.market.prices.get_mut(&substance) {
                    *price = *price / 3;
                }
            }
            4 => {
                // Substance price spike
                let substance = SUBSTANCES[rng.gen_range(0..SUBSTANCES.len())].to_string();
                let message = format!(
                    "📰 Police busted a {} shipment! Prices skyrocketing!",
                    substance
                );
                self.message_log.push(message);

                if let Some(price) = self.market.prices.get_mut(&substance) {
                    *price = *price * 3;
                }
            }
            5 => {
                // Find some drugs
                let substance = SUBSTANCES[rng.gen_range(0..SUBSTANCES.len())].to_string();
                let amount = rng.gen_range(1..5);

                if self.player.space_available() >= amount {
                    *self.player.inventory.entry(substance.clone()).or_insert(0) += amount;
                    let message = format!(
                        "Lucky! You found {} units of {} on the ground!",
                        amount, substance
                    );
                    self.message_log.push(message);
                } else {
                    self.message_log
                        .push("You found some drugs but had no space to carry them.".to_string());
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

        // Check for possible stash house raids
        self.check_stash_house_raid();

        // Generate new market prices for the new location
        self.market.generate_prices();

        // Add market events to message log
        for event in &self.market.events {
            self.message_log.push(event.clone());
        }

        // Return to main screen
        self.current_screen = Screen::Main;
    }

    fn check_stash_house_raid(&mut self) {
        let mut rng = rand::thread_rng();

        // Only check if player has at least one stash house
        if self.player.stash_houses.is_empty() {
            return;
        }

        // 5% chance of a raid per travel if player has stash houses
        if rng.gen_range(0..100) < 5 {
            // Determine which stash house gets raided (if player has multiple)
            let stash_locations: Vec<String> = self.player.stash_houses.keys().cloned().collect();
            let raided_location = &stash_locations[rng.gen_range(0..stash_locations.len())];

            self.message_log.push(format!(
                "🚨 ALERT! Your stash house in {} was raided by police!",
                raided_location
            ));

            // Check if the stash has anything in it
            if let Some(stash) = self.player.stash_houses.get_mut(raided_location) {
                let has_substances = stash.inventory.values().sum::<i32>() > 0;

                if has_substances {
                    // Determine how much is confiscated (50-100%)
                    let confiscation_percentage = rng.gen_range(50..=100);

                    self.message_log.push(format!(
                        "Police confiscated {}% of your stash!",
                        confiscation_percentage
                    ));

                    // Apply confiscation to each substance
                    for (_, amount) in stash.inventory.iter_mut() {
                        if *amount > 0 {
                            let confiscated = (*amount * confiscation_percentage) / 100;
                            *amount -= confiscated;
                        }
                    }

                    // Potential fine
                    let fine = rng.gen_range(1000..5000).min(self.player.cash);
                    if fine > 0 {
                        self.player.cash -= fine;
                        self.message_log
                            .push(format!("You were fined ${} for the illegal stash!", fine));
                    }
                } else {
                    self.message_log.push(
                        "Luckily, your stash was empty so nothing was confiscated!".to_string(),
                    );
                }

                // Small chance (10%) the stash house is seized entirely
                if rng.gen_range(0..100) < 10 {
                    self.message_log.push(format!(
                        "Your stash house in {} was seized by authorities!",
                        raided_location
                    ));
                    self.player.stash_houses.remove(raided_location);
                }
            }
        }
    }

    fn handle_police_encounter(&mut self) -> bool {
        let mut rng = rand::thread_rng();

        // Check if the player has any drugs
        let has_drugs = self.player.inventory.values().sum::<i32>() > 0;
        if !has_drugs {
            self.message_log.push(
                "The cops let you go since you aren't carrying anything illegal.".to_string(),
            );
            return false; // No drugs, no problem
        }

        // Police encounter dialog
        self.message_log
            .push("🚨 POLICE! You've been stopped for a search!".to_string());

        // Let's avoid using clone() for Screen enum
        // Create a custom dialog for police encounter
        let mut encounter_resolved = false;

        // Fight outcome
        let weapon_power = self.player.get_active_weapon_power();
        let fight_chance = if weapon_power > 0 {
            // Base chance plus weapon bonus
            20 + (weapon_power as f32 * 0.7) as i32
        } else {
            // Base chance with no weapon
            10
        };

        // Use a Grenade if equipped (one-time use)
        let using_grenade = if let Some(active_weapon) = &self.player.active_weapon {
            active_weapon == "Grenade"
        } else {
            false
        };

        // Get the substance with the most quantity
        let mut max_substance = String::new();
        let mut max_amount = 0;

        for (substance, amount) in &self.player.inventory {
            if *amount > max_amount {
                max_amount = *amount;
                max_substance = substance.clone();
            }
        }

        if using_grenade {
            // Grenade is extremely effective but one-time use
            let success = rng.gen_range(0..100) < 95; // 95% chance to escape

            if success {
                self.message_log
                    .push("💥 You threw a grenade and escaped in the chaos!".to_string());

                // Remove the grenade
                if let Some(amount) = self.player.weapons.get_mut("Grenade") {
                    *amount -= 1;
                }

                // Unequip the grenade
                if self.player.active_weapon == Some("Grenade".to_string()) {
                    self.player.active_weapon = None;
                }

                return false; // Escaped successfully
            } else {
                self.message_log.push(
                    "💥 Your grenade didn't go off properly! The police arrested you!".to_string(),
                );
                self.confiscate_substances();

                // Remove the grenade
                if let Some(amount) = self.player.weapons.get_mut("Grenade") {
                    *amount -= 1;
                }

                // Unequip the grenade
                if self.player.active_weapon == Some("Grenade".to_string()) {
                    self.player.active_weapon = None;
                }

                return true; // Caught
            }
        }

        // Determine if the player fights, runs, or bribes
        let encounter_type = rng.gen_range(0..3);

        match encounter_type {
            0 => {
                // Fight
                let success = rng.gen_range(0..100) < fight_chance;

                if success {
                    if weapon_power > 0 {
                        self.message_log.push(format!(
                            "You used your {} to fight off the cops and escaped!",
                            self.player
                                .active_weapon
                                .as_ref()
                                .unwrap_or(&"weapon".to_string())
                        ));
                    } else {
                        self.message_log.push("You somehow managed to fight off the cops with your bare hands and escaped!".to_string());
                    }

                    // Take some damage
                    let damage = rng.gen_range(5..20);
                    self.player.health -= damage;
                    self.message_log
                        .push(format!("You got hurt in the fight (-{} health)", damage));

                    if self.player.health <= 0 {
                        self.player.health = 1; // Don't let player die from police fight
                        self.message_log.push(
                            "You're critically injured and need medical attention immediately!"
                                .to_string(),
                        );
                    }

                    return false; // Escaped
                } else {
                    if weapon_power > 0 {
                        self.message_log.push(format!("Even with your {}, you couldn't fight off the cops. You've been arrested!",
                            self.player.active_weapon.as_ref().unwrap_or(&"weapon".to_string())));
                    } else {
                        self.message_log.push("You tried to fight the cops with your bare hands and failed miserably! You've been arrested!".to_string());
                    }

                    self.confiscate_substances();

                    // Take more damage for failing
                    let damage = rng.gen_range(15..40);
                    self.player.health -= damage;
                    self.message_log
                        .push(format!("The cops roughed you up (-{} health)", damage));

                    if self.player.health <= 0 {
                        self.player.health = 1; // Don't let player die from police fight
                        self.message_log.push(
                            "You're critically injured and need medical attention immediately!"
                                .to_string(),
                        );
                    }

                    return true; // Caught
                }
            }
            1 => {
                // Run
                let run_chance = 30 + (self.player.health / 4); // Health affects running ability
                let success = rng.gen_range(0..100) < run_chance;

                if success {
                    self.message_log
                        .push("You outran the cops and got away!".to_string());
                    return false; // Escaped
                } else {
                    self.message_log.push(
                        "You tried to run but the cops caught you! You've been arrested!"
                            .to_string(),
                    );
                    self.confiscate_substances();
                    return true; // Caught
                }
            }
            _ => {
                // Bribe opportunity
                self.message_log.push(
                    "The cop hints that this could all go away for the right price...".to_string(),
                );

                // Let's determine a reasonable bribe amount based on what the player is carrying
                let total_value = self.calculate_inventory_value();
                let min_bribe = total_value / 4;
                let ideal_bribe = total_value / 2;

                // Bribe success depends on the ratio of offered bribe to ideal bribe
                let bribe_amount = rng.gen_range(min_bribe..=ideal_bribe);
                let bribe_success = rng.gen_range(0..100) < 70; // 70% chance of success

                if self.player.cash >= bribe_amount && bribe_success {
                    self.player.cash -= bribe_amount;
                    self.message_log.push(format!(
                        "You slipped the cop ${} and they let you go.",
                        bribe_amount
                    ));
                    return false; // Bribe worked
                } else {
                    if self.player.cash < bribe_amount {
                        self.message_log.push(format!(
                            "The cop wanted ${} but you couldn't afford it!",
                            bribe_amount
                        ));
                    } else {
                        self.message_log.push(format!(
                            "You offered ${} but the cop refused the bribe!",
                            bribe_amount
                        ));
                    }

                    self.message_log
                        .push("You've been arrested and your substances confiscated!".to_string());
                    self.confiscate_substances();
                    return true; // Caught
                }
            }
        }
    }

    fn confiscate_substances(&mut self) {
        // Confiscate all substances
        for (_, amount) in self.player.inventory.iter_mut() {
            *amount = 0;
        }

        self.message_log
            .push("All your substances have been confiscated!".to_string());
    }

    fn calculate_inventory_value(&self) -> i32 {
        let mut total = 0;

        for (substance, amount) in &self.player.inventory {
            if let Some(price) = self.market.prices.get(substance) {
                total += price * amount;
            }
        }

        total
    }

    fn render_police_encounter(&mut self, ui: &mut Ui) {
        ui.add_space(20.0);

        let mut rng = rand::thread_rng();

        ui.vertical_centered(|ui| {
            ui.heading("🚨 POLICE ENCOUNTER 🚨");
            ui.label("You've been stopped by the police!");

            let total_drugs = self.player.inventory.values().sum::<i32>();

            if total_drugs == 0 {
                ui.label("You aren't carrying any illegal substances.");
                ui.label("The officer lets you go with a warning.");

                if ui.button("Continue").clicked() {
                    self.current_screen = Screen::Main;
                }
                return;
            }

            ui.add_space(10.0);
            ui.label("The officer wants to search you. You're carrying:");

            for (substance, amount) in &self.player.inventory {
                if *amount > 0 {
                    ui.label(format!("- {} units of {}", amount, substance));
                }
            }

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                let fight_outcome = if ui.button("Fight").clicked() {
                    let weapon_power = self.player.get_active_weapon_power();
                    let fight_chance = if weapon_power > 0 {
                        20 + (weapon_power as f32 * 0.7) as i32
                    } else {
                        10
                    };

                    let success = rng.gen_range(0..100) < fight_chance;

                    if success {
                        if weapon_power > 0 {
                            self.message_log.push(format!("You used your {} to fight off the cops and escaped!",
                                self.player.active_weapon.as_ref().unwrap_or(&"weapon".to_string())));
                        } else {
                            self.message_log.push("You somehow managed to fight off the cops with your bare hands and escaped!".to_string());
                        }

                        // Take some damage
                        let damage = rng.gen_range(5..20);
                        self.player.health -= damage;
                        self.message_log.push(format!("You got hurt in the fight (-{} health)", damage));

                        if self.player.health <= 0 {
                            self.player.health = 1; // Don't let player die from police fight
                            self.message_log.push("You're critically injured and need medical attention immediately!".to_string());
                        }

                        self.current_screen = Screen::Main;
                        true
                    } else {
                        if weapon_power > 0 {
                            self.message_log.push(format!("Even with your {}, you couldn't fight off the cops. You've been arrested!",
                                self.player.active_weapon.as_ref().unwrap_or(&"weapon".to_string())));
                        } else {
                            self.message_log.push("You tried to fight the cops with your bare hands and failed miserably! You've been arrested!".to_string());
                        }

                        self.confiscate_substances();

                        // Take more damage for failing
                        let damage = rng.gen_range(15..40);
                        self.player.health -= damage;
                        self.message_log.push(format!("The cops roughed you up (-{} health)", damage));

                        if self.player.health <= 0 {
                            self.player.health = 1; // Don't let player die from police fight
                            self.message_log.push("You're critically injured and need medical attention immediately!".to_string());
                        }

                        self.current_screen = Screen::Main;
                        false
                    }
                } else {
                    false
                };

                let run_outcome = if ui.button("Run").clicked() {
                    let run_chance = 30 + (self.player.health / 4); // Health affects running ability
                    let success = rng.gen_range(0..100) < run_chance;

                    if success {
                        self.message_log.push("You outran the cops and got away!".to_string());
                        self.current_screen = Screen::Main;
                        true
                    } else {
                        self.message_log.push("You tried to run but the cops caught you! You've been arrested!".to_string());
                        self.confiscate_substances();
                        self.current_screen = Screen::Main;
                        false
                    }
                } else {
                    false
                };

                let bribe_clicked = ui.button("Offer Bribe").clicked();
                if bribe_clicked {
                    // Handle the bribe logic
                    let bribe_successful = self.offer_bribe(ui);
                    if bribe_successful {
                        self.current_screen = Screen::Main;
                    } else {
                        self.current_screen = Screen::Main;
                    }
                }

                let surrender_outcome = if ui.button("Surrender").clicked() {
                    self.message_log.push("You surrender to the police.".to_string());
                    self.confiscate_substances();

                    // Sometimes you get a fine
                    let fine_chance = rng.gen_bool(0.7); // 70% chance of fine
                    if fine_chance {
                        let fine = rng.gen_range(500..2000).min(self.player.cash);
                        if fine > 0 {
                            self.player.cash -= fine;
                            self.message_log.push(format!("You were fined ${} for possession!", fine));
                        }
                    }

                    self.current_screen = Screen::Main;
                    false
                } else {
                    false
                };
            });
        });
    }

    fn offer_bribe(&mut self, ui: &mut Ui) -> bool {
        let mut rng = rand::thread_rng();

        ui.heading("🚨 POLICE ENCOUNTER");
        ui.label("The officer seems open to a bribe...");

        let total_value = self.calculate_inventory_value();
        let suggested_bribe = total_value / 3;

        ui.label(format!(
            "You're carrying inventory worth approximately ${}",
            total_value
        ));
        ui.label(format!("Suggested bribe: ${}", suggested_bribe));

        ui.horizontal(|ui| {
            ui.label("Bribe amount: $");
            ui.text_edit_singleline(&mut self.bribe_amount);
        });

        let bribe: i32 = match self.bribe_amount.parse() {
            Ok(num) => num,
            Err(_) => 0,
        };

        let can_bribe = bribe > 0 && bribe <= self.player.cash;

        if !can_bribe {
            if bribe <= 0 {
                ui.label(RichText::new("Enter a positive amount").color(Color32::RED));
            } else if bribe > self.player.cash {
                ui.label(RichText::new("You don't have that much cash").color(Color32::RED));
            }
        }

        let bribe_offered = ui
            .add_enabled(can_bribe, egui::Button::new("Offer Bribe"))
            .clicked();
        let refuse = ui.button("Refuse to Bribe").clicked();

        if bribe_offered {
            // Calculate success chance based on bribe amount relative to inventory value
            let ratio = bribe as f32 / total_value as f32;
            let success_chance = (ratio * 100.0).min(95.0).max(10.0) as i32;

            let success = rng.gen_range(0..100) < success_chance;

            self.player.cash -= bribe;

            if success {
                self.message_log.push(format!(
                    "You offered ${} as a bribe and the officer accepted!",
                    bribe
                ));
                self.message_log
                    .push("The officer lets you go with a warning.".to_string());
                return true; // Successful bribe
            } else {
                self.message_log.push(format!(
                    "You offered ${} as a bribe but the officer refused!",
                    bribe
                ));
                self.message_log
                    .push("The officer seems offended and decides to arrest you.".to_string());
                self.confiscate_substances();

                // Additional penalty for failed bribe
                let fine = rng.gen_range(500..2000).min(self.player.cash);
                if fine > 0 {
                    self.player.cash -= fine;
                    self.message_log.push(format!(
                        "You were fined an additional ${} for attempted bribery!",
                        fine
                    ));
                }

                return false; // Failed bribe
            }
        }

        if refuse {
            self.message_log
                .push("You refused to offer a bribe.".to_string());

            // 50% chance of getting arrested anyway
            if rng.gen_bool(0.5) {
                self.message_log.push(
                    "The officer decides to search you anyway and finds your stash!".to_string(),
                );
                self.confiscate_substances();
                return false;
            } else {
                self.message_log
                    .push("The officer lets you go with a warning.".to_string());
                return true;
            }
        }

        // If we get here, the player hasn't made a decision yet
        false
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

    fn render_weapons_screen(&mut self, ui: &mut Ui) {
        ui.heading("Weapons Shop");
        ui.label(format!("Cash: ${}", self.player.cash));

        ui.separator();

        ui.label("Buy weapons to protect yourself from cops and thugs!");
        ui.label("A better weapon increases your chance of successfully defending yourself.");

        ui.separator();

        let mut equip_clicked = false;
        let mut equipped_weapon = None;

        // First, show the player's current weapons
        ui.heading("Your Weapons");
        let has_weapons = self.player.weapons.values().sum::<i32>() > 0;

        if has_weapons {
            for (weapon, amount) in &self.player.weapons {
                if *amount > 0 {
                    let is_active = match &self.player.active_weapon {
                        Some(active) => active == weapon,
                        None => false,
                    };

                    ui.horizontal(|ui| {
                        ui.label(format!("{}: {}", weapon, amount));

                        if !is_active {
                            if ui.button("Equip").clicked() {
                                equip_clicked = true;
                                equipped_weapon = Some(weapon.clone());
                            }
                        } else {
                            ui.label(RichText::new(" [EQUIPPED]").color(Color32::GREEN));
                        }
                    });
                }
            }
        } else {
            ui.label("You don't have any weapons yet.");
        }

        if equip_clicked && equipped_weapon.is_some() {
            self.player.active_weapon = equipped_weapon;
            self.message_log.push(format!(
                "You equipped a {}.",
                self.player.active_weapon.as_ref().unwrap()
            ));
        }

        ui.separator();

        // Show weapons for sale
        ui.heading("Weapons For Sale");

        for (i, weapon) in WEAPONS.iter().enumerate() {
            let price = WEAPON_PRICES[i];
            let power = WEAPON_POWER[i];

            let is_selected = match &self.selected_weapon {
                Some(selected) => selected == weapon,
                None => false,
            };

            if ui
                .selectable_label(
                    is_selected,
                    format!("{} - ${} (Power: {})", weapon, price, power),
                )
                .clicked()
            {
                self.selected_weapon = Some(weapon.to_string());
            }
        }

        ui.separator();

        if let Some(weapon) = &self.selected_weapon {
            // Find the weapon index
            let mut weapon_index = 0;
            for (i, w) in WEAPONS.iter().enumerate() {
                if *w == *weapon {
                    weapon_index = i;
                    break;
                }
            }

            let price = WEAPON_PRICES[weapon_index];
            let power = WEAPON_POWER[weapon_index];

            ui.label(format!("Selected: {} - ${}", weapon, price));
            ui.label(format!("Power Rating: {}/100", power));

            match weapon.as_str() {
                "Knife" => {
                    ui.label("A basic knife. Better than nothing in a fight.");
                }
                "Pistol" => {
                    ui.label("Standard handgun. Decent stopping power.");
                }
                "Shotgun" => {
                    ui.label("Good for close encounters. Very intimidating.");
                }
                "Assault Rifle" => {
                    ui.label("Military grade. Highly effective but expensive.");
                }
                "Grenade" => {
                    ui.label("Dangerous but extremely effective. One-time use only.");
                }
                _ => {}
            }

            let can_buy = self.player.cash >= price;

            if !can_buy {
                ui.label(RichText::new("You can't afford this weapon.").color(Color32::RED));
            }

            if ui
                .add_enabled(can_buy, egui::Button::new("Buy Weapon"))
                .clicked()
            {
                self.player.cash -= price;
                *self.player.weapons.entry(weapon.clone()).or_insert(0) += 1;

                let message = format!("Bought a {} for ${}", weapon, price);
                self.message_log.push(message);

                // If this is the player's first weapon, automatically equip it
                if self.player.active_weapon.is_none() {
                    self.player.active_weapon = Some(weapon.clone());
                    self.message_log
                        .push(format!("You equipped your new {}.", weapon));
                }

                // Clear selection
                self.selected_weapon = None;
            }
        } else {
            ui.label("Select a weapon to purchase");
        }
    }

    fn render_stash_houses_screen(&mut self, ui: &mut Ui) {
        ui.heading("Stash Houses");

        ui.separator();

        ui.label("Stash houses let you store extra inventory in each location.");
        ui.label("This allows you to buy low and sell high across different boroughs.");

        ui.separator();

        // First check if player has a stash house in the current location
        let has_stash_in_current_location = self.player.owns_stash_house_in_current_location();

        // Show list of owned stash houses
        ui.heading("Your Stash Houses");

        if self.player.stash_houses.is_empty() {
            ui.label("You don't own any stash houses yet.");
        } else {
            for (location, stash) in &self.player.stash_houses {
                let is_current = *location == self.player.current_location;
                let text = if is_current {
                    RichText::new(format!(
                        "{}: {}/{} capacity (CURRENT LOCATION)",
                        location,
                        stash.total_items(),
                        stash.capacity
                    ))
                    .color(Color32::GREEN)
                } else {
                    RichText::new(format!(
                        "{}: {}/{} capacity",
                        location,
                        stash.total_items(),
                        stash.capacity
                    ))
                };

                ui.label(text);
            }
        }

        ui.separator();

        // Options for the current location
        ui.heading(format!(
            "Current Location: {}",
            self.player.current_location
        ));

        if has_stash_in_current_location {
            ui.horizontal(|ui| {
                if ui.button("Deposit Items").clicked() {
                    self.current_screen = Screen::StashDeposit;
                    self.stash_selected_substance = None;
                    self.stash_amount_to_transfer = "0".to_string();
                }

                if ui.button("Withdraw Items").clicked() {
                    self.current_screen = Screen::StashWithdraw;
                    self.stash_selected_substance = None;
                    self.stash_amount_to_transfer = "0".to_string();
                }
            });
        } else {
            // Show purchase option
            let stash_price = self
                .player
                .get_stash_house_price(&self.player.current_location);
            ui.label(format!(
                "Cost to buy a stash house in {}: ${}",
                self.player.current_location, stash_price
            ));

            let can_afford = self.player.cash >= stash_price;

            if !can_afford {
                ui.label(RichText::new("You can't afford a stash house here.").color(Color32::RED));
            }

            if ui
                .add_enabled(can_afford, egui::Button::new("Buy Stash House"))
                .clicked()
            {
                self.player.cash -= stash_price;

                // Create new stash house
                let stash = StashHouse::new(self.player.current_location.clone());
                self.player
                    .stash_houses
                    .insert(self.player.current_location.clone(), stash);

                let message = format!(
                    "Purchased a stash house in {} for ${}",
                    self.player.current_location, stash_price
                );
                self.message_log.push(message);
            }
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if ui.button("Return to Main Menu").clicked() {
                self.current_screen = Screen::Main;
            }
        });
    }

    fn render_stash_deposit_screen(&mut self, ui: &mut Ui) {
        ui.heading("Deposit Items to Stash House");

        if !self.player.owns_stash_house_in_current_location() {
            ui.label("You don't own a stash house in this location.");

            if ui.button("Back").clicked() {
                self.current_screen = Screen::StashHouses;
            }

            return;
        }

        // Copy necessary data to avoid borrow conflicts
        let current_location = self.player.current_location.clone();
        let player_total = self.player.total_items();
        let player_space = self.player.space_available();

        // Get the stash house for the current location
        if let Some(stash) = self.player.stash_houses.get_mut(&current_location) {
            let stash_total = stash.total_items();
            let stash_capacity = stash.capacity;
            let stash_space = stash.space_available();

            ui.label(format!(
                "Your personal inventory: {}/{} units",
                player_total, MAX_CARRYING_CAPACITY
            ));
            ui.label(format!(
                "Stash house inventory: {}/{} units",
                stash_total, stash_capacity
            ));

            ui.separator();

            // Check if player has anything to deposit
            let has_inventory = self.player.inventory.values().sum::<i32>() > 0;

            if !has_inventory {
                ui.label("You don't have any substances to deposit.");
            } else {
                for substance in SUBSTANCES.iter() {
                    let substance_str = substance.to_string();
                    let amount = self.player.inventory.get(&substance_str).unwrap_or(&0);

                    if *amount > 0 {
                        let is_selected = match &self.stash_selected_substance {
                            Some(selected) => selected == &substance_str,
                            None => false,
                        };

                        if ui
                            .selectable_label(
                                is_selected,
                                format!("{} - {} units", substance, amount),
                            )
                            .clicked()
                        {
                            self.stash_selected_substance = Some(substance_str.clone());
                            self.stash_amount_to_transfer = "0".to_string();
                        }
                    }
                }

                ui.separator();

                if let Some(ref substance) = self.stash_selected_substance {
                    let current_amount = self.player.inventory.get(substance).unwrap_or(&0);

                    ui.label(format!(
                        "Selected: {} - {} units",
                        substance, current_amount
                    ));
                    ui.label(format!("Space available in stash: {}", stash_space));

                    ui.horizontal(|ui| {
                        ui.label("Amount to deposit:");
                        ui.text_edit_singleline(&mut self.stash_amount_to_transfer);

                        if ui.button("Max").clicked() {
                            let max_amount = std::cmp::min(*current_amount, stash_space);
                            self.stash_amount_to_transfer = max_amount.to_string();
                        }
                    });

                    let amount: i32 = match self.stash_amount_to_transfer.parse() {
                        Ok(num) => num,
                        Err(_) => 0,
                    };

                    let can_deposit =
                        amount > 0 && amount <= *current_amount && amount <= stash_space;

                    if !can_deposit {
                        let mut reason = "Cannot deposit: ".to_string();
                        if amount <= 0 {
                            reason.push_str("Amount must be positive. ");
                        }
                        if amount > *current_amount {
                            reason.push_str("Not enough in your inventory. ");
                        }
                        if amount > stash_space {
                            reason.push_str("Not enough space in stash house. ");
                        }
                        ui.label(RichText::new(reason).color(Color32::RED));
                    }

                    if ui
                        .add_enabled(can_deposit, egui::Button::new("Deposit"))
                        .clicked()
                    {
                        // Remove from player inventory
                        *self.player.inventory.get_mut(substance).unwrap() -= amount;

                        // Add to stash inventory
                        *stash.inventory.entry(substance.clone()).or_insert(0) += amount;

                        let message =
                            format!("Deposited {} units of {} to your stash", amount, substance);
                        self.message_log.push(message);

                        self.stash_selected_substance = None;
                        self.stash_amount_to_transfer = "0".to_string();
                    }
                } else if has_inventory {
                    ui.label("Select a substance to deposit");
                }
            }
        }

        ui.separator();

        if ui.button("Back to Stash Menu").clicked() {
            self.current_screen = Screen::StashHouses;
        }
    }

    fn render_stash_withdraw_screen(&mut self, ui: &mut Ui) {
        ui.heading("Withdraw Items from Stash House");

        if !self.player.owns_stash_house_in_current_location() {
            ui.label("You don't own a stash house in this location.");

            if ui.button("Back").clicked() {
                self.current_screen = Screen::StashHouses;
            }

            return;
        }

        // Copy necessary data to avoid borrow conflicts
        let current_location = self.player.current_location.clone();
        let player_total = self.player.total_items();
        let player_space = self.player.space_available();

        // Get the stash house for the current location
        if let Some(stash) = self.player.stash_houses.get_mut(&current_location) {
            let stash_total = stash.total_items();
            let stash_capacity = stash.capacity;

            ui.label(format!(
                "Your personal inventory: {}/{} units",
                player_total, MAX_CARRYING_CAPACITY
            ));
            ui.label(format!(
                "Stash house inventory: {}/{} units",
                stash_total, stash_capacity
            ));

            ui.separator();

            // Check if stash has anything to withdraw
            let has_stash_inventory = stash.inventory.values().sum::<i32>() > 0;

            if !has_stash_inventory {
                ui.label("Your stash house is empty.");
            } else {
                for substance in SUBSTANCES.iter() {
                    let substance_str = substance.to_string();
                    let amount = stash.inventory.get(&substance_str).unwrap_or(&0);

                    if *amount > 0 {
                        let is_selected = match &self.stash_selected_substance {
                            Some(selected) => selected == &substance_str,
                            None => false,
                        };

                        if ui
                            .selectable_label(
                                is_selected,
                                format!("{} - {} units", substance, amount),
                            )
                            .clicked()
                        {
                            self.stash_selected_substance = Some(substance_str.clone());
                            self.stash_amount_to_transfer = "0".to_string();
                        }
                    }
                }

                ui.separator();

                if let Some(ref substance) = self.stash_selected_substance {
                    let stash_amount = stash.inventory.get(substance).unwrap_or(&0);

                    ui.label(format!("Selected: {} - {} units", substance, stash_amount));
                    ui.label(format!(
                        "Space available in your inventory: {}",
                        player_space
                    ));

                    ui.horizontal(|ui| {
                        ui.label("Amount to withdraw:");
                        ui.text_edit_singleline(&mut self.stash_amount_to_transfer);

                        if ui.button("Max").clicked() {
                            let max_amount = std::cmp::min(*stash_amount, player_space);
                            self.stash_amount_to_transfer = max_amount.to_string();
                        }
                    });

                    let amount: i32 = match self.stash_amount_to_transfer.parse() {
                        Ok(num) => num,
                        Err(_) => 0,
                    };

                    let can_withdraw =
                        amount > 0 && amount <= *stash_amount && amount <= player_space;

                    if !can_withdraw {
                        let mut reason = "Cannot withdraw: ".to_string();
                        if amount <= 0 {
                            reason.push_str("Amount must be positive. ");
                        }
                        if amount > *stash_amount {
                            reason.push_str("Not enough in stash. ");
                        }
                        if amount > player_space {
                            reason.push_str("Not enough space in your inventory. ");
                        }
                        ui.label(RichText::new(reason).color(Color32::RED));
                    }

                    if ui
                        .add_enabled(can_withdraw, egui::Button::new("Withdraw"))
                        .clicked()
                    {
                        // Remove from stash inventory
                        *stash.inventory.get_mut(substance).unwrap() -= amount;

                        // Add to player inventory
                        *self.player.inventory.entry(substance.clone()).or_insert(0) += amount;

                        let message =
                            format!("Withdrew {} units of {} from your stash", amount, substance);
                        self.message_log.push(message);

                        self.stash_selected_substance = None;
                        self.stash_amount_to_transfer = "0".to_string();
                    }
                } else if has_stash_inventory {
                    ui.label("Select a substance to withdraw");
                }
            }
        }

        ui.separator();

        if ui.button("Back to Stash Menu").clicked() {
            self.current_screen = Screen::StashHouses;
        }
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

            ui.add_space(10.0);

            if !self.player.stash_houses.is_empty() {
                ui.label("Stash houses:");
                for (location, stash) in &self.player.stash_houses {
                    let total_items = stash.total_items();
                    if total_items > 0 {
                        ui.label(format!("{}: {} units", location, total_items));

                        for (substance, amount) in &stash.inventory {
                            if *amount > 0 {
                                ui.label(format!("  {}: {}", substance, amount));
                            }
                        }
                    } else {
                        ui.label(format!("{}: empty", location));
                    }
                }

                ui.add_space(10.0);
            }

            ui.add_space(20.0);

            if ui.button("Play Again").clicked() {
                self.restart_requested = true;
            }
        });
    }
}

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
