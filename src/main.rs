#![windows_subsystem = "windows"] // 隐藏windows console

mod probe;

slint::include_modules!();
use std::fs;
use std::rc::Rc;
use rfd::FileDialog;
use slint::{ModelRc, SharedString, VecModel};
use std::fs::File;
use std::io::prelude::*;
use yaml_rust::YamlLoader;

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    let probe_list : Rc<VecModel<SharedString>> =
            Rc::new(VecModel::from(string_to_shared_string(probe::list_probe())));
    let probe_list = ModelRc::from(probe_list.clone());
    ui.set_probe_list(probe_list);

    let mcu_list = load_file("./mcu");

    let ui_handle = ui.as_weak();
    ui.on_mcu_completion(move |name_start| {
        let ui = ui_handle.unwrap();
        let mcu_completion_list = mcu_completion(name_start.to_string(), mcu_list.clone());
        let mcu_completion_list : Rc<VecModel<SharedString>> =
            Rc::new(VecModel::from(string_to_shared_string(mcu_completion_list)));
        let mcu_completion_list = ModelRc::from(mcu_completion_list.clone());
        ui.set_mcu_completion_list(mcu_completion_list);
    });

    let ui_handle = ui.as_weak();
    ui.on_open_file_select_dialog(move || {
        let ui = ui_handle.unwrap();
        ui.set_file_path(file_select_dialog().into());
    });

    let ui_handle = ui.as_weak();
    ui.on_mcu_erase(move || {
        let ui = ui_handle.unwrap();
        let probe = probe::probe_open(0).unwrap();
        let session = probe::probe_attach(probe, ui.get_mcu_selected().to_string()).unwrap();
        let _ = probe::erase_target(session).unwrap();
        ui.set_erase_log("擦除完成".to_string().into());
    });

    let ui_handle = ui.as_weak();
    ui.on_mcu_flash(move || {
        let ui = ui_handle.unwrap();

        println!("probe: {}", ui.get_probe_selected());
        println!("mcu: {}", ui.get_mcu_selected());
        println!("path: {}", ui.get_file_path());

        let probe = probe::probe_open(0).unwrap();
        let session = probe::probe_attach(probe, ui.get_mcu_selected().to_string()).unwrap();
        let _ = probe::flash_target(session, ui.get_file_path().to_string(), "elf".to_string()).unwrap();
        ui.set_flash_log("烧录完成".to_string().into());
    });

    ui.run()
}

fn file_select_dialog() -> String {
    let files = FileDialog::new()
        .add_filter("file", &["bin", "hex", "elf"])
        .pick_file();
    String::from(files.unwrap().to_str().unwrap())
}

fn load_file(path: &str) -> Vec<String> {

    let mut mcu_list = Vec::new();

    for yaml_file in fs::read_dir(path).unwrap() {
        let yaml_file = yaml_file.unwrap().path();
        let yaml_file = yaml_file.to_str().unwrap();
        if yaml_file.ends_with(".yaml") {
            let mut yaml_file = File::open(yaml_file).expect("Unable to open file");
            let mut contents = String::new();
            yaml_file.read_to_string(&mut contents).expect("Unable to read file");

            let doc = YamlLoader::load_from_str(&contents).unwrap();
            let doc = &doc[0];
            for i in 0..doc["variants"].as_vec().unwrap().len() {
                let variant = &doc["variants"][i];
                let name = variant["name"].as_str().unwrap();
                mcu_list.push(name.to_string());
            }
        }
    }
    mcu_list
}

fn string_to_shared_string(string_list: Vec<String>) -> Vec<SharedString> {
    let mut shared_string_list = Vec::new();
    for i in 0..string_list.len() {
        shared_string_list.push(string_list[i].to_string().into());
    }
    shared_string_list
}

fn mcu_completion(name: String, mcu_list: Vec<String>) -> Vec<String> {
    let mut mcu_completion_list = Vec::new();
    for i in 0..mcu_list.len() {
        if mcu_list[i].to_lowercase().contains(&name.to_lowercase()) {
            mcu_completion_list.push(mcu_list[i].to_string());
        }
    }
    mcu_completion_list
}