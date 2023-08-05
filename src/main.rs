#![windows_subsystem = "windows"]   // 隐藏windows console

use std::path::Path;
use flashing::erase_all;
use iced::executor;
use iced::subscription;
use iced::widget::{button, container, text, Column, combo_box};
use iced::window;
use iced::Event;
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
    Theme,
};
use probe_rs::{Probe, flashing, Permissions};
use probe_rs::flashing::BinOptions;

pub fn main() -> iced::Result {
    DapDownloader::run(Settings {
        exit_on_close_request: true,
        ..Settings::default()
    })
}

#[derive(Debug)]
struct DapDownloader {
    targets: combo_box::State<TargetMCU>,
    selected_target: Option<TargetMCU>,
    target_mcu: String,
    target_selected_flag: bool,
    file_path: String,
    file_format: String,
    text: String,
}

#[derive(Debug, Clone)]
enum Message {
    DropFile(Event),
    TargetSelected(TargetMCU),
    EraseTarget,
    FlashTarget,
    ResetTarget,
}


impl Application for DapDownloader {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (DapDownloader, Command<Message>) {
        (Self {
            targets: combo_box::State::new(TargetMCU::ALL.to_vec()),
            selected_target: None,
            target_mcu: String::new(),
            target_selected_flag: false,
            file_path: String::new(),
            file_format: String::new(),
            text: String::from("Drag and drop a file here"),
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("DAP Downloader")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DropFile(event) => {
                match self.target_selected_flag {
                    false => {
                        self.text = String::from("Please select a target MCU first");
                        Command::none()
                    }
                    true => {
                        let Event::Window(window::Event::FileDropped(path)) = event else { return Command::none() };
                        self.file_path = path.to_str().unwrap().to_string();
                        // self.text = self.file_path.clone();
                        match Path::new(&self.file_path).extension() {
                            Some(ext) => {
                                let ext = ext.to_str().unwrap();
                                match ext {
                                    "bin" => {
                                        self.file_format = String::from("bin");
                                    }
                                    "hex" => {
                                        self.file_format = String::from("hex");
                                    }
                                    "elf" => {
                                        self.file_format = String::from("elf");
                                    }
                                    _ => {
                                        self.text = String::from("unsupported file format");
                                    }
                                }
                            }
                            None => {
                                self.file_format = String::from("elf");
                            }
                        }
                        flash_target(self.target_mcu.clone(), self.file_path.clone(), self.file_format.clone());
                        self.text = String::from("Flash target MCU successfully");
                        Command::none()
                    }
                }
            }
            Message::TargetSelected(target) => {
                self.selected_target = Some(target);
                self.targets.unfocus();
                self.target_mcu = target.to_string();
                self.target_selected_flag = true;
                Command::none()
            }
            Message::EraseTarget => {
                match self.target_selected_flag {
                    false => {
                        // TODO: Popup a window to remind user to select a target MCU first
                        Command::none()
                    }
                    true => {
                        erase_target(self.target_mcu.clone());
                        self.text = String::from("Erase target MCU successfully");
                        Command::none()
                    }
                }
            }
            Message::FlashTarget => {
                match self.target_selected_flag {
                    false => {
                        // TODO: Popup a window to remind user to select a target MCU first
                        Command::none()
                    }
                    true => {
                        match self.file_path.is_empty() {
                            true => {
                                self.text = String::from("Please drop a file");
                                Command::none()
                            }
                            false => {
                                flash_target(self.target_mcu.clone(), self.file_path.clone(), self.file_format.clone());
                                self.text = String::from("Flash target MCU successfully");
                                Command::none()
                            }
                        }
                    }
                }
            }
            Message::ResetTarget => {
                match self.target_selected_flag {
                    false => {
                        // TODO: Popup a window to remind user to select a target MCU first
                        Command::none()
                    }
                    true => {
                        reset_target(self.target_mcu.clone());
                        self.text = String::from("Reset target MCU successfully");
                        Command::none()
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {

        let log = text(self.text.clone());
        let file_path = text(self.file_path.clone());

        let combo_box = combo_box(
            &self.targets,
            "Select a MCU...",
            self.selected_target.as_ref(),
            Message::TargetSelected,
        )
            .width(250);

        let btn_erase = button(
            text("Erase")
        )
            .width(100)
            .padding(10)
            .on_press(Message::EraseTarget);

        let btn_flash = button(
            text("Flash")
        )
            .width(100)
            .padding(10)
            .on_press(Message::FlashTarget);

        let btn_reset = button(
            text("Reset")
        )
            .width(100)
            .padding(10)
            .on_press(Message::ResetTarget);

        let content = Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(log)
            .push(file_path)
            .push(combo_box)
            .push(btn_erase)
            .push(btn_flash)
            .push(btn_reset);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        return subscription::events().map(Message::DropFile)
    }
}


fn flash_target(target: String, path: String, format: String) {
    let probes = Probe::list_all();
    let probe = probes[0].open().unwrap();
    let mut session = probe.attach(target, Permissions::default()).unwrap();
    erase_all(&mut session, None).unwrap();

    let _res = match format.as_str() {
        "bin" => flashing::download_file(&mut session, path, flashing::Format::Bin(BinOptions { base_address: None, skip: 0 })),
        "hex" => flashing::download_file(&mut session, path, flashing::Format::Hex),
        "elf" => flashing::download_file(&mut session, path, flashing::Format::Elf),
        _ => Ok(()),
    }.expect("TODO: panic message");

    let mut core = session.core(0).unwrap();
    core.reset().unwrap();
}


fn reset_target(target: String) {
    let probes = Probe::list_all();
    let probe = probes[0].open().unwrap();
    let mut session = probe.attach(target, Permissions::default()).unwrap();
    let mut core = session.core(0).unwrap();
    core.reset().unwrap();
}


fn erase_target(target: String) {
    let probes = Probe::list_all();
    let probe = probes[0].open().unwrap();
    let mut session = probe.attach(target, Permissions::default()).unwrap();
    erase_all(&mut session, None).unwrap();
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetMCU {
    #[default]
    STM32F103C8,
    STM32F103CB,
}

impl TargetMCU {
    const ALL: [TargetMCU; 2] = [
        TargetMCU::STM32F103C8,
        TargetMCU::STM32F103CB,
    ];

    fn to_string(&self) -> String {
        match self {
            TargetMCU::STM32F103C8 => "stm32f103c8".to_string(),
            TargetMCU::STM32F103CB => "stm32f103cb".to_string(),
        }
    }
}


impl std::fmt::Display for TargetMCU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TargetMCU::STM32F103C8 => "stm32f103c8",
                TargetMCU::STM32F103CB => "stm32f103cb",
            }
        )
    }
}