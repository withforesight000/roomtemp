// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod controller;
mod domain;
mod infrastructure;
mod presentation;
mod repository;
mod usecase;

fn main() {
    app_lib::run();
}
