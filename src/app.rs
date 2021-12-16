use std::collections::{HashMap};
use eframe::{egui, epi};
use eframe::egui::{Ui};
use crate::app::MinTermState::{DontCare, One, Zero};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    table: Vec<HashMap<u32, Vec<MinTerm>>>,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum MinTermState {
    Zero,
    One,
    DontCare,
}

#[derive(PartialEq, Debug)]
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
                let parsed_table = parse_string(label);
                *table = find_all_primimplikante(parsed_table);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                let mut prim = list_primimplikants(&table);
                if prim.len() > 0 {
                    for t in table.iter_mut() {
                        show_quine_table(t, ui);
                    }
                    simplify_table(&mut prim);
                    ui.horizontal(|ui| {
                        ui.label("F = ");
                        for i in 0..prim.len() {
                            let term = &prim[i];
                            for k in 0..term.digit.len() {
                                if term.digit[k] != DontCare {
                                    let letter = (k + 'A' as usize) as u8 as char;
                                    let negated = if term.digit[k] == Zero { "'" } else { "" };
                                    ui.label(format!("{}{}", letter, negated));
                                }
                            }
                            if i < prim.len() - 1 {
                                ui.label(" + ");
                            }
                        }
                    });
                }
            });
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
    egui::Grid::new("unique ID").min_col_width(200f32).show(ui, |ui| {
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
    ui.horizontal(|ui| ui.separator());
}

fn simplify_table(input: &mut Vec<MinTerm>) {
    let mut changed = true;
    while changed {
        changed = false;
        changed = eliminate_vertical(input) || changed;
        changed = eleminate_horizontal(input) || changed;
    }
}

//      0,1,4,8,5,6,9,7,11,15
fn eliminate_vertical(input: &mut Vec<MinTerm>) -> bool {
    let mut essential: Vec<i32> = vec![];
    let mut found = true;
    let mut operations = 0;
    while found {
        found = false;
        let mut deletion_term: MinTerm = MinTerm { original: vec![], is_primimplikant: true, digit: vec![] };
        for term in input.iter() {
            for min in &term.original {
                if !essential.contains(min) && contained_by_only_one(*min, input) {
                    essential.push(*min);
                    found = true;
                    deletion_term = copy_min_term(term);
                    break;
                }
            }
            if found {
                break;
            }
        }
        if found {
            for term in input.iter_mut() {
                for to_remove in &*deletion_term.original {
                    if term.original.contains(&to_remove) && !essential.contains(&to_remove) {
                        term.original.remove(term.original.iter().position(|&r| r == *to_remove).unwrap());
                        operations += 1;
                    }
                }
            }
        }
    }
    operations > 0
}

fn eleminate_horizontal(input: &mut Vec<MinTerm>) -> bool {
    let mut found = true;
    let mut operations = 0;
    while found {
        found = false;
        //remove empty lines
        for index in 0..input.len() {
            if input[index].original.len() == 0 {
                found = true;
                input.remove(index);
                break;
            }
        }
        //remove dominated lines
        if !found {
            for dominator_index in 0..input.len() {
                for recessive_index in dominator_index + 1..input.len() {
                    if minterm_dominates(&input[dominator_index], &input[recessive_index]) {
                        found = true;
                        input.remove(recessive_index);
                        operations += 1;
                        break;
                    }
                }
                if found {
                    break;
                }
            }
        }
    }
    operations > 0
}

fn minterm_dominates(dominator: &MinTerm, rezesiv: &MinTerm) -> bool {
    for min in &rezesiv.original {
        if !dominator.original.contains(min) {
            return false;
        }
    }
    true
}

fn contained_by_only_one(num: i32, input: &Vec<MinTerm>) -> bool {
    let mut count = 0;
    for t in input {
        for i in &t.original {
            if num == *i {
                count += 1;
            }
        }
    }
    count == 1
}

fn list_primimplikants(input: &Vec<HashMap<u32, Vec<MinTerm>>>) -> Vec<MinTerm> {
    let mut primimplikants: Vec<MinTerm> = vec![];
    for t in input {
        for terms_list in t.values() {
            for term in terms_list {
                if term.is_primimplikant {
                    primimplikants.push(copy_min_term(term));
                }
            }
        }
    }
    primimplikants
}

fn find_all_primimplikante(input: HashMap<u32, Vec<MinTerm>>) -> Vec<HashMap<u32, Vec<MinTerm>>> {
    let mut table: Vec<HashMap<u32, Vec<MinTerm>>> = vec![];
    table.push(input);
    let mut current_index = 0;
    let mut was_simplified = true;
    while was_simplified {
        let primimplikant: (HashMap<u32, Vec<MinTerm>>, bool) = find_prim_implikante(table.get_mut(current_index).unwrap());
        was_simplified = primimplikant.1;
        if was_simplified {
            table.push(primimplikant.0);
            current_index += 1;
        }
    }
    table
}

fn find_prim_implikante(input: &mut HashMap<u32, Vec<MinTerm>>) -> (HashMap<u32, Vec<MinTerm>>, bool) {
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
    for index in 0..find_max_index(&table) + 1 {
        if table.contains_key(&index) {
            let list = table.get_mut(&index).unwrap();
            //manual dedup
            let mut dedup_found = true;
            while dedup_found {
                dedup_found = false;
                for index in 0..list.len() {
                    for control in index + 1..list.len() {
                        if list[index] == list[control] {
                            list.remove(control);
                            dedup_found = true;
                            break;
                        }
                    }
                    if dedup_found {
                        break;
                    }
                }
            }
            if list.len() == 0 {
                table.remove(&index);
            }
        }
    }
    (table, found_something)
}

fn find_max_index(table: &HashMap<u32, Vec<MinTerm>>) -> u32 {
    let mut max_index = 0;
    for key in table.keys() {
        if max_index < *key {
            max_index = *key;
        }
    }
    max_index
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
        let ones = term.count_ones();
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

fn copy_min_term(term: &MinTerm) -> MinTerm {
    let mut merged = MinTerm { original: vec![], is_primimplikant: term.is_primimplikant, digit: vec![] };
    for o in &term.original {
        merged.original.push(*o);
    }
    for i in 0..term.digit.len() {
        merged.digit.push(term.digit[i]);
    }
    merged
}


fn log_2(x: i32) -> u32 {
    assert!(x > 0);
    32 - x.leading_zeros() + 1
}