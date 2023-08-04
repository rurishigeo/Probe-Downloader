#![windows_subsystem = "windows"]   // 隐藏windows console

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
    file_path: String,
    file_format: String,
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
            file_path: String::from("Drag and drop a file here"),
            file_format: String::new(),
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("DAP Downloader")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DropFile(event) => {
                if let Event::Window(window::Event::FileDropped(path)) = event {
                    // dbg!(path.clone());
                    self.file_path = path.to_str().unwrap().to_string();
                    // 取file_path的后缀名
                    let path = std::path::Path::new(&self.file_path);
                    match path.extension() {
                        Some(ext) => {
                            let ext = ext.to_str().unwrap().to_string();
                            if ext == "bin" {
                                self.file_format = String::from("bin");
                            } else if ext == "hex" {
                                self.file_format = String::from("hex");
                            } else if ext == "elf" {
                                self.file_format = String::from("elf");
                            } else {
                                self.file_path = String::from("unsupported file format");
                            }
                        }
                        None => {
                            self.file_format = String::from("elf");
                        }
                    }
                    flash_target(self.target_mcu.clone(), self.file_path.clone(), self.file_format.clone());
                }
                Command::none()
            }
            Message::TargetSelected(target) => {
                self.selected_target = Some(target);
                self.targets.unfocus();
                self.target_mcu = target.to_string();
                Command::none()
            }
            Message::EraseTarget => {
                erase_target(self.target_mcu.clone());
                Command::none()
            }
            Message::FlashTarget => {
                flash_target(self.target_mcu.clone(), self.file_path.clone(), self.file_format.clone());
                Command::none()
            }
            Message::ResetTarget => {
                reset_target(self.target_mcu.clone());
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {

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
    flashing::erase_all(&mut session, Option::None).unwrap();
    
    match format.as_str() {
        "bin" => flashing::download_file(&mut session, path, flashing::Format::Bin(BinOptions { base_address: None, skip: 0 })).unwrap(),
        "hex" => flashing::download_file(&mut session, path, flashing::Format::Hex).unwrap(),
        "elf" => flashing::download_file(&mut session, path, flashing::Format::Elf).unwrap(),
        _ => (),
    }
    // flashing::download_file(&mut session, path, flashing::Format::Elf).unwrap();
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
    flashing::erase_all(&mut session, Option::None).unwrap();
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