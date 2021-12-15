use std::borrow::{Borrow, BorrowMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Add;
use eframe::{egui, epi};
use eframe::egui::{InnerResponse, Ui, Vec2};
use crate::app::MinTermState::{DontCare, One};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    table: Vec<HashMap<u32, Vec<MinTerm>>>,
}

#[derive(PartialEq, Copy, Clone)]
enum MinTermState {
    Zero,
    One,
    DontCare,
}

#[derive(PartialEq)]
struct MinTerm {
    original: Vec<i32>,
    digit: Vec<MinTermState>,
    is_primimplikant: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "".to_owned(),
            table: vec![HashMap::new()],
        }
    }
}

impl epi::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let mut fonts = egui::FontDefinitions::default();
        // Large button text:
        fonts.family_and_size.insert(
            egui::TextStyle::Body,
            (egui::FontFamily::Proportional, 22.0));
        ctx.set_fonts(fonts);
        let Self { label, table } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Min-Terme eingeben ");
                ui.text_edit_singleline(label);
            });
            if ui.button("Optimieren").clicked() {
                let mut parsed_table = parse_string(label);
                *table = find_all_primimplikante(parsed_table);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            for t in table {
                show_quine_table(t, ui);
            }
            egui::warn_if_debug_build(ui);
        });
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {}

    fn name(&self) -> &str {
        "Simplifizierung nach Quine McCluskey"
    }
}

fn show_quine_table(table: &HashMap<u32, Vec<MinTerm>>, ui: &mut Ui) {
    egui::Grid::new("unique ID").show(ui, |ui| {
        ui.label("Group");
        ui.label("MinTerm");
        if table.capacity() > 0 {
            let min_term = table.values().next().unwrap().get(0).unwrap();
            for k in (0..min_term.digit.len()).rev() {
                ui.label(format!("{}", (k + 'A' as usize) as u8 as char));
            }
        }
        ui.end_row();
        let max_index = find_max_index(table);
        for index in 0..max_index + 1 {
            if table.contains_key(&index) {
                for entry in table.get(&index) {
                    for min_term in entry.into_iter() {
                        ui.label(format!("{}", index));
                        ui.horizontal(|ui| {
                            for num in &min_term.original {
                                ui.label(format!("{},", num));
                            }
                        });
                        let term = &min_term.digit;
                        for k in (0..term.len()).rev() {
                            let show = match term[k] {
                                MinTermState::One => "1",
                                MinTermState::Zero => "0",
                                MinTermState::DontCare => "-"
                            };
                            ui.label(show);
                        }
                        ui.label(if min_term.is_primimplikant { "P" } else { "_" });
                        ui.end_row();
                    }
                }
            }
        }
    });
}

fn find_all_primimplikante(mut input: HashMap<u32, Vec<MinTerm>>) -> Vec<HashMap<u32, Vec<MinTerm>>> {
    let mut table: Vec<HashMap<u32, Vec<MinTerm>>> = vec![];
    table.push(input);
    let mut currentIndex = 0;
    let mut was_simplified = true;
    while was_simplified {
        let primImplikant: (HashMap<u32, Vec<MinTerm>>, bool) = find_primImplikante(table.get_mut(currentIndex).unwrap());
        was_simplified = primImplikant.1;
        if was_simplified {
            table.push(primImplikant.0);
            currentIndex += 1;
        }
    }
    table
}

fn find_primImplikante(input: &mut HashMap<u32, Vec<MinTerm>>) -> (HashMap<u32, Vec<MinTerm>>, bool) {
    let mut table: HashMap<u32, Vec<MinTerm>> = HashMap::new();
    let mut found_something = false;
    for index in 0..find_max_index(input) + 1 {
        if input.contains_key(&index) && input.contains_key(&(index + 1)) {
            let mut min_terms_small = input.remove(&index).unwrap();
            let mut min_terms_bigger = input.remove(&(index + 1)).unwrap();
            for lower in min_terms_small.iter_mut() {
                for higher in min_terms_bigger.iter_mut() {
                    if has_single_digit_difference(lower, higher) {
                        lower.is_primimplikant = false;
                        higher.is_primimplikant = false;
                        found_something = true;
                        let merged = merge(lower, higher);
                        let ones = count_ones(&merged);
                        if table.get(&ones).is_none() {
                            table.insert(ones, vec![]);
                        }
                        table.get_mut(&ones).unwrap().push(merged);
                    }
                }
            }
            input.insert(index, min_terms_small);
            input.insert(index + 1, min_terms_bigger);
        }
    }
    for index in 0..find_max_index(&table)+1{
        if table.contains_key(&index){
            let list=table.get_mut(&index).unwrap();
            list.dedup();
        }
    }
    (table, found_something)
}

fn find_max_index(table: &HashMap<u32, Vec<MinTerm>>) -> u32 {
    let mut maxIndex = 0;
    for key in table.keys() {
        if maxIndex < *key {
            maxIndex = *key;
        }
    }
    maxIndex
}

fn parse_string(input: &str) -> HashMap<u32, Vec<MinTerm>> {
    let min_terms: Vec<i32> = input.split(",").map(|s| s.parse::<i32>().unwrap()).collect();
    let mut max = 0;
    for i in &min_terms {
        if *i > max {
            max = *i;
        }
    }
    let bits = log_2(max);
    let mut table: HashMap<u32, Vec<MinTerm>> = HashMap::new();
    for term in min_terms {
        let mut ones = term.count_ones();
        let min_term = parse_min_term(term, &bits);
        if table.get(&ones).is_none() {
            table.insert(ones, vec![]);
        }
        table.get_mut(&ones).unwrap().push(min_term);
    }
    table
}

fn count_ones(term: &MinTerm) -> u32 {
    let mut count = 0;
    for d in &term.digit {
        if *d == One {
            count += 1;
        }
    }
    count
}

fn parse_min_term(input: i32, digits: &u32) -> MinTerm {
    let mut result: MinTerm = MinTerm { original: vec![input], digit: vec![], is_primimplikant: true };
    for i in 0..*digits - 1 {
        let digit = input >> i & 0x1;
        if digit == 0 {
            result.digit.push(MinTermState::Zero);
        } else {
            result.digit.push(MinTermState::One);
        }
    }
    result
}

fn has_single_digit_difference(min1: &MinTerm, min2: &MinTerm) -> bool {
    let mut count = 0;
    for i in 0..min1.digit.len() {
        if min1.digit[i] != min2.digit[i] {
            count += 1;
        }
    }
    count <= 1
}

fn merge(min1: &MinTerm, min2: &MinTerm) -> MinTerm {
    let mut merged = MinTerm { original: vec![], is_primimplikant: true, digit: vec![] };
    for o in &min1.original {
        merged.original.push(*o);
    }
    for o in &min2.original {
        merged.original.push(*o);
    }
    merged.original.sort();
    for i in 0..min1.digit.len() {
        if min1.digit[i] != min2.digit[i] {
            merged.digit.push(DontCare);
        } else {
            merged.digit.push(min1.digit[i]);
        }
    }
    merged
}

fn log_2(x: i32) -> u32 {
    assert!(x > 0);
    32 - x.leading_zeros() + 1
}