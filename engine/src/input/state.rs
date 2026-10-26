//! Input state machine (data-model.md#inputstate-eingabezustand); times are passed in,
//! so it runs without threads or clocks.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::storage::{MidiDevice, Transport};

/// Repeats of the disconnect warning after the first one (FR-005).
const WARNING_REPEATS_MS: [u64; 2] = [10_000, 30_000];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum InputState {
    Disconnected,
    MidiConnected,
    ConnectionLost,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InputInfo {
    pub id: u32,
    pub name: String,
    pub transport: Transport,
}

/// Present devices and the selected one, which may be absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InputList {
    pub midi: Vec<InputInfo>,
    pub selected: Option<MidiDevice>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub state: InputState,
    pub device: Option<String>,
    pub receiving: bool,
    pub warning_active: bool,
}

/// What the runtime has to do on the MIDI layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Connect(u32),
    Disconnect(u32),
    /// Warning sound and vibration.
    Warn,
}

pub struct InputMachine {
    devices: BTreeMap<u32, InputInfo>,
    selected: Option<MidiDevice>,
    connected: Option<u32>,
    lost: bool,
    receiving: bool,
    /// Due times of the pending warnings; empty once reconnected or acknowledged.
    warnings_due: Vec<u64>,
}

impl InputMachine {
    /// `selected`: the input restored from the settings (FR-002).
    pub fn new(selected: Option<MidiDevice>) -> Self {
        InputMachine {
            devices: BTreeMap::new(),
            selected,
            connected: None,
            lost: false,
            receiving: false,
            warnings_due: Vec::new(),
        }
    }

    pub fn status(&self) -> Status {
        let state = match (self.connected, self.lost) {
            (Some(_), _) => InputState::MidiConnected,
            (None, true) => InputState::ConnectionLost,
            (None, false) => InputState::Disconnected,
        };
        Status {
            state,
            device: self.selected.as_ref().map(|d| d.name.clone()),
            receiving: self.receiving,
            warning_active: !self.warnings_due.is_empty(),
        }
    }

    pub fn selected(&self) -> Option<&MidiDevice> {
        self.selected.as_ref()
    }

    pub fn connected(&self) -> Option<u32> {
        self.connected
    }

    pub fn list(&self) -> InputList {
        InputList {
            midi: self.devices.values().cloned().collect(),
            selected: self.selected.clone(),
        }
    }

    /// The selected device is connected again as soon as it appears (FR-006).
    pub fn device_added(&mut self, id: u32, name: String, transport: Transport) -> Vec<Action> {
        let info = InputInfo {
            id,
            name,
            transport,
        };
        let is_selected = self.connected.is_none() && self.matches_selected(&info);
        self.devices.insert(id, info);
        if is_selected {
            self.connect(id)
        } else {
            Vec::new()
        }
    }

    pub fn device_removed(&mut self, id: u32, now_ms: u64) -> Vec<Action> {
        self.devices.remove(&id);
        if self.connected != Some(id) {
            return Vec::new();
        }
        self.connected = None;
        self.lost = true;
        self.receiving = false;
        self.warnings_due = WARNING_REPEATS_MS.iter().map(|d| now_ms + d).collect();
        vec![Action::Warn]
    }

    /// Whether the message belongs to the selected input; without one nothing is assigned (FR-007).
    pub fn message(&mut self, id: u32) -> bool {
        let accepted = self.connected == Some(id);
        self.receiving |= accepted;
        accepted
    }

    /// `None` deselects; an unknown device changes nothing.
    pub fn select(&mut self, id: Option<u32>) -> Vec<Action> {
        let info = match id {
            Some(id) => match self.devices.get(&id) {
                Some(info) => Some(info.clone()),
                None => return Vec::new(),
            },
            None => None,
        };
        let mut actions: Vec<Action> = self
            .connected
            .take()
            .map(Action::Disconnect)
            .into_iter()
            .collect();
        self.lost = false;
        self.receiving = false;
        self.warnings_due.clear();
        self.selected = info.as_ref().map(|i| MidiDevice {
            name: i.name.clone(),
            transport: i.transport,
            bluetooth_id: None,
        });
        if let Some(info) = info {
            actions.extend(self.connect(info.id));
        }
        actions
    }

    pub fn acknowledge_warning(&mut self) {
        self.warnings_due.clear();
    }

    /// Manual reconnect (FR-006): reopens the selected device if it is present.
    pub fn reconnect(&mut self) -> Vec<Action> {
        let Some(id) = self
            .devices
            .values()
            .find(|info| self.matches_selected(info))
            .map(|info| info.id)
        else {
            return Vec::new();
        };
        let mut actions: Vec<Action> = self
            .connected
            .take()
            .map(Action::Disconnect)
            .into_iter()
            .collect();
        actions.extend(self.connect(id));
        actions
    }

    pub fn next_deadline(&self) -> Option<u64> {
        self.warnings_due.first().copied()
    }

    pub fn tick(&mut self, now_ms: u64) -> Vec<Action> {
        let due = self
            .warnings_due
            .iter()
            .take_while(|&&t| t <= now_ms)
            .count();
        self.warnings_due.drain(..due);
        vec![Action::Warn; due]
    }

    fn matches_selected(&self, info: &InputInfo) -> bool {
        self.selected
            .as_ref()
            .is_some_and(|d| d.name == info.name && d.transport == info.transport)
    }

    fn connect(&mut self, id: u32) -> Vec<Action> {
        self.connected = Some(id);
        self.lost = false;
        self.receiving = false;
        self.warnings_due.clear();
        vec![Action::Connect(id)]
    }
}
