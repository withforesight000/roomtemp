// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod domain;
mod usecase;
mod repository;
mod controller;
mod presentation;
mod infrastructure;

fn main() {
  app_lib::run();
}
