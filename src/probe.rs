use probe_rs::flashing::{BinOptions, FileDownloadError, FlashError};
use probe_rs::{flashing, DebugProbeError, DebugProbeInfo, Permissions, ProbeCreationError, Session, Probe};


pub struct MyProbe {
    probe_list: Vec<DebugProbeInfo>,
    probe_name_list: Vec<String>,
    probe_selected: Option<usize>,
}

impl MyProbe {
    pub fn new() -> Self {
        MyProbe {
            probe_list: Vec::new(),
            probe_name_list: Vec::new(),
            probe_selected: None,
        }
    }
    
    pub fn update(&mut self) {
        self.probe_list = probe_rs::Probe::list_all();
        for probe in probe_rs::Probe::list_all() {
            self.probe_name_list.push(probe.identifier.clone());
        }
    }
    
    pub fn open(&self) -> Result<probe_rs::Probe, DebugProbeError> {
        let probe = self.probe_list
            .get(self.probe_selected.unwrap())
            .ok_or(DebugProbeError::ProbeCouldNotBeCreated(
                ProbeCreationError::NotFound,
            ))?
            .open()?;
        Ok(probe)
    }
    
    pub fn attach(probe: probe_rs::Probe, target: String) -> Result<Session, probe_rs::Error> {
        let session = probe.attach(target, Permissions::default())?;
        Ok(session)
    }
}

// 列出烧录器
pub fn list_probe() -> Vec<String> {
    let mut probe_list: Vec<String> = Vec::new();
    for probe in probe_rs::Probe::list_all() {
        probe_list.push(probe.identifier.clone());
    }
    probe_list
}

pub fn probe_open(probe_id: usize) -> Result<Probe, DebugProbeError> {
    let probes = probe_rs::Probe::list_all();
    let probe = probes
        .get(probe_id)
        .ok_or(DebugProbeError::ProbeCouldNotBeCreated(
            ProbeCreationError::NotFound,
        ))?
        .open()?;
    Ok(probe)
}

pub fn probe_attach(probe: probe_rs::Probe, target: String) -> Result<Session, probe_rs::Error> {
    let session = probe.attach(target, Permissions::default())?;
    Ok(session)
}

pub fn flash_target(
    mut session: Session,
    path: String,
    format: String,
) -> Result<(), FileDownloadError> {
    flashing::erase_all(&mut session, None).unwrap();

    let _ = match format.as_str() {
        "bin" => flashing::download_file(
            &mut session,
            path,
            flashing::Format::Bin(BinOptions {
                base_address: None,
                skip: 0,
            }),
        )?,
        "hex" => flashing::download_file(&mut session, path, flashing::Format::Hex)?,
        "elf" => flashing::download_file(&mut session, path, flashing::Format::Elf)?,
        _ => (),
    };

    let mut core = session.core(0).unwrap();
    core.reset().unwrap();
    return Ok(());
}

pub fn erase_target(mut session: Session) -> Result<(), FlashError> {
    flashing::erase_all(&mut session, None)?;
    return Ok(());
}