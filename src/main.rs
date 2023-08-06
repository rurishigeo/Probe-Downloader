#![windows_subsystem = "windows"]   // 隐藏windows console

use std::fmt::Debug;
use std::path::PathBuf;

use iced::{executor, Padding};
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
    Theme,
};
use iced::Event;
use iced::subscription;
use iced::widget::{button, Column, combo_box, container, pick_list, Row, text};
use iced::window;
use probe_rs::{DebugProbeError, flashing, Permissions, Probe, ProbeCreationError, Session};
use probe_rs::flashing::{BinOptions, FileDownloadError, FlashError};

pub fn main() -> iced::Result {
    DapDownload::run(Settings {
        window: window::Settings {
            size: (800, 480),
            resizable: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug)]
struct DapDownload {
    // 烧录器相关参数
    probe_list: Vec<String>,
    probe_selected: Option<usize>,

    // 目标芯片相关参数
    target_list: combo_box::State<TargetMCU>,
    target_selected: Option<TargetMCU>,

    // 烧录文件相关参数
    file_path: Option<String>,
    file_format: Option<String>,

    // 界面相关参数
    log_text: String,
}

#[derive(Debug, Clone)]
enum Message {
    // 烧录器相关消息
    ProbeSelected(String),
    ProbeRefresh,

    // 目标芯片相关消息
    TargetSelected(TargetMCU),

    // 烧录文件相关消息
    FileSelected(PathBuf),

    // 界面相关消息
    Erase,
    Flash,
}


impl Application for DapDownload {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (DapDownload, Command<Message>) {
        (Self {
            probe_list: list_probe(),
            probe_selected: match list_probe().len() {
                0 => None,
                _ => Some(0),
            },

            target_list: combo_box::State::new(TargetMCU::ALL.to_vec()),
            target_selected: None,

            file_path: None,
            file_format: None,

            log_text: String::from("Drag and drop a file here"),
        }, {
             Command::none()
         })
    }

    fn title(&self) -> String {
        String::from("DAP Download")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ProbeSelected(probe) => {
                self.probe_selected = Some(self.probe_list.iter().position(|r| r == probe.as_str()).unwrap());
                Command::none()
            }

            Message::ProbeRefresh => {
                self.probe_list = list_probe();
                self.probe_selected = match list_probe().len() {
                    0 => None,
                    _ => Some(0),
                };
                Command::none()
            }

            Message::TargetSelected(target) => {
                self.target_list.unfocus();
                self.target_selected = Some(target);
                Command::none()
            }

            Message::FileSelected(path) => {
                self.file_format = match path.extension() {
                    Some(ext) => {
                        let ext = ext.to_str().unwrap();
                        if ext == "bin" || ext == "hex" || ext == "elf" {
                            self.file_path = Some(path.to_str().unwrap().to_string());
                            self.log_text = String::from("File loaded");
                            Some(String::from(ext))
                        } else {
                            self.log_text = String::from("Unsupported file format");
                            self.file_format.clone()
                        }
                    }
                    None => {
                        self.file_path = Some(path.to_str().unwrap().to_string());
                        self.log_text = String::from("File loaded");
                        Some(String::from("elf"))
                    }
                };
                Command::none()
            }

            Message::Erase => {
                return if self.probe_selected.is_none() {
                    self.log_text = String::from("Please select a probe");
                    Command::none()
                } else if self.target_selected.is_none() {
                    self.log_text = String::from("Please select a MCU");
                    Command::none()
                } else {
                    let probe_id = self.probe_selected.unwrap();
                    let target = self.target_selected.unwrap();

                    let probe = match probe_open(probe_id) {
                        Ok(probe) => probe,
                        Err(_) => {
                            self.log_text = String::from("Probe open failed, Refresh probe list.");
                            return Command::none();
                        }
                    };
                    let session = match probe_attach(probe, target.to_string()) {
                        Ok(session) => session,
                        Err(_) => {
                            self.log_text = String::from("Can not attach to target!");
                            return Command::none();
                        }
                    };
                    match erase_target(session) {
                        Ok(_) => (),
                        Err(_) => {
                            self.log_text = String::from("Erase failed!");
                            return Command::none();
                        }
                    };
                    self.log_text = String::from("Erase done!");
                    Command::none()
                };
            }

            Message::Flash => {
                return if self.probe_selected.is_none() {
                    self.log_text = String::from("Please select a probe!");
                    Command::none()
                } else if self.target_selected.is_none() {
                    self.log_text = String::from("Please select a MCU!");
                    Command::none()
                } else if self.file_format.is_none() {
                    self.log_text = String::from("Drag and drop a file here!");
                    Command::none()
                } else {
                    let probe_id = self.probe_selected.unwrap();
                    let target = self.target_selected.unwrap();
                    let path = self.file_path.clone().unwrap();
                    let format = self.file_format.clone().unwrap();

                    let probe = match probe_open(probe_id) {
                        Ok(probe) => probe,
                        Err(_) => {
                            self.log_text = String::from("Probe open failed, Refresh probe list.");
                            return Command::none();
                        }
                    };
                    let session = match probe_attach(probe, target.to_string()) {
                        Ok(session) => session,
                        Err(_) => {
                            self.log_text = String::from("Can not attach to target!");
                            return Command::none();
                        }
                    };
                    match flash_target(session, path, format) {
                        Ok(_) => (),
                        Err(_) => {
                            self.log_text = String::from("Flash failed, Check your file!");
                            return Command::none();
                        }
                    }
                    self.log_text = String::from("Flash done!");
                    Command::none()
                };
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let probe_list = pick_list(
            &self.probe_list,
            Some({
                match self.probe_selected {
                    Some(selected) => self.probe_list.get(selected).unwrap().to_string(),
                    None => "No probe found".to_string(),
                }
            }),
            Message::ProbeSelected,
        )
            .width(150);

        let probe_list_btn = button(
            text("Ref")
        )
            .width(50)
            .on_press(Message::ProbeRefresh);

        let probe_box = Row::new()
            .width(200)
            .align_items(Alignment::Center)
            .spacing(0)
            .push(probe_list)
            .push(probe_list_btn);

        let target_list = combo_box(
            &self.target_list,
            "Select a MCU...",
            self.target_selected.as_ref(),
            Message::TargetSelected,
        )
            .width(200);

        let target_box = Column::new()
            .width(240)
            .align_items(Alignment::Center)
            .spacing(20)
            .push(probe_box)
            .push(target_list);


        let log = text(self.log_text.clone())
            .width(200);

        let file_path = text(match self.file_path.clone() {
            None => "No file selected".to_string(),
            Some(path) => "Now Loaded: ".to_string() + path.as_str(),
        })
            .width(200);

        let path_box = Column::new()
            .width(240)
            .align_items(Alignment::Center)
            .spacing(20)
            .push(log)
            .push(file_path);

        let erase_btn = button(
            text("Erase")
        )
            .width(200)
            .height(50)
            .padding(Padding::from([13, 80]))
            .on_press(Message::Erase);

        let flash_btn = button(
            text("Flash")
        )
            .width(200)
            .height(50)
            .padding(Padding::from([13, 80]))
            .on_press(Message::Flash);

        let btn_box = Column::new()
            .width(240)
            .align_items(Alignment::Center)
            .spacing(20)
            .push(erase_btn)
            .push(flash_btn);

        let content = Row::new()
            .align_items(Alignment::Center)
            .spacing(0)
            .push(target_box)
            .push(path_box)
            .push(btn_box);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, _| match event {
            Event::Window(window::Event::FileDropped(path)) => {
                Some(Message::FileSelected(path))
            }
            _ => None,
        })
    }
}

// 列出烧录器
fn list_probe() -> Vec<String> {
    let mut probe_list: Vec<String> = Vec::new();
    for probe in Probe::list_all() {
        probe_list.push(probe.identifier.clone());
    }
    probe_list
}

fn probe_open(probe_id: usize) -> Result<Probe, DebugProbeError> {
    let probes = Probe::list_all();
    let probe = probes.get(probe_id)
        .ok_or(DebugProbeError::ProbeCouldNotBeCreated(ProbeCreationError::NotFound))?
        .open()?;
    Ok(probe)
}


fn probe_attach(probe: Probe, target: String) -> Result<Session, probe_rs::Error> {
    let session = probe.attach(target, Permissions::default())?;
    Ok(session)
}


fn flash_target(mut session: Session, path: String, format: String) -> Result<(), FileDownloadError> {
    flashing::erase_all(&mut session, None).unwrap();

    let _ = match format.as_str() {
        "bin" => flashing::download_file(&mut session, path, flashing::Format::Bin(BinOptions { base_address: None, skip: 0 }))?,
        "hex" => flashing::download_file(&mut session, path, flashing::Format::Hex)?,
        "elf" => flashing::download_file(&mut session, path, flashing::Format::Elf)?,
        _ => (),
    };

    let mut core = session.core(0).unwrap();
    core.reset().unwrap();
    return Ok(());
}


fn erase_target(mut session: Session) -> Result<(), FlashError> {
    flashing::erase_all(&mut session, None)?;
    return Ok(());
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