//! C interface to the native MIDI layer (contracts/native-midi.md).
//!
//! Every entry point only enqueues and returns; a worker thread processes the
//! input, keeps the debug log and calls back into the platform.

use std::collections::{BTreeMap, VecDeque};
use std::ffi::{CStr, CString, c_char};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, OnceLock};
use std::thread;

use serde_json::{Value, json};

pub const LEGATO_TRANSPORT_USB: u8 = 0;
pub const LEGATO_TRANSPORT_BLUETOOTH: u8 = 1;

const DEBUG_LOG_CAPACITY: usize = 10_000;

/// Callbacks of the native layer, registered once at start.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LegatoMidiPlatform {
    pub connect: Option<unsafe extern "C" fn(device: u32)>,
    pub disconnect: Option<unsafe extern "C" fn(device: u32)>,
    pub send: Option<unsafe extern "C" fn(device: u32, data: *const u8, len: usize)>,
    pub bluetooth_scan: Option<unsafe extern "C" fn(enabled: bool)>,
    /// Pending connection request that never times out.
    pub bluetooth_connect: Option<unsafe extern "C" fn(identifier: *const c_char)>,
    /// iOS: playback + mixWithOthers.
    pub prepare_audio_session: Option<unsafe extern "C" fn()>,
    /// No-op without vibration motor (e.g. iPad).
    pub vibrate: Option<unsafe extern "C" fn()>,
}

enum Input {
    Register(LegatoMidiPlatform),
    Message {
        device: u32,
        time_us: u64,
        data: Vec<u8>,
    },
    DeviceAdded {
        device: u32,
        name: String,
        transport: u8,
    },
    DeviceRemoved {
        device: u32,
    },
    BluetoothFound {
        identifier: String,
        name: String,
    },
    ScanBluetooth(bool),
    SelectBluetooth(CString),
}

fn sender() -> &'static Sender<Input> {
    static SENDER: OnceLock<Sender<Input>> = OnceLock::new();
    SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("legato-midi".into())
            .spawn(move || Worker::default().run(rx))
            .expect("spawn legato-midi worker");
        tx
    })
}

fn enqueue(input: Input) {
    // The worker lives for the whole process; a send error cannot occur.
    let _ = sender().send(input);
}

fn debug_log() -> &'static Mutex<VecDeque<String>> {
    static LOG: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
    LOG.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn log(entry: Value) {
    let mut log = debug_log().lock().unwrap_or_else(|e| e.into_inner());
    if log.len() == DEBUG_LOG_CAPACITY {
        log.pop_front();
    }
    log.push_back(entry.to_string());
}

#[derive(Default)]
struct Worker {
    platform: Option<LegatoMidiPlatform>,
    transports: BTreeMap<u32, u8>,
    bluetooth_identifier: Option<CString>,
}

impl Worker {
    fn run(mut self, rx: Receiver<Input>) {
        for input in rx {
            self.handle(input);
        }
    }

    fn handle(&mut self, input: Input) {
        match input {
            Input::Register(platform) => self.platform = Some(platform),
            Input::Message {
                device,
                time_us,
                data,
            } => log(
                json!({ "kind": "message", "device": device, "time_us": time_us, "data": data }),
            ),
            Input::DeviceAdded {
                device,
                name,
                transport,
            } => {
                self.transports.insert(device, transport);
                if let Some(connect) = self.platform.and_then(|p| p.connect) {
                    // SAFETY: callback registered by the native layer for exactly this signature.
                    unsafe { connect(device) };
                }
                log(json!({
                    "kind": "device_added",
                    "device": device,
                    "name": name,
                    "transport": transport_name(transport),
                }));
            }
            Input::DeviceRemoved { device } => {
                if self.transports.remove(&device) == Some(LEGATO_TRANSPORT_BLUETOOTH) {
                    self.request_bluetooth_connection();
                }
                log(json!({ "kind": "device_removed", "device": device }));
            }
            Input::BluetoothFound { identifier, name } => log(json!({
                "kind": "bluetooth_found",
                "identifier": identifier,
                "name": name,
            })),
            Input::ScanBluetooth(enabled) => {
                if let Some(scan) = self.platform.and_then(|p| p.bluetooth_scan) {
                    // SAFETY: see above.
                    unsafe { scan(enabled) };
                }
            }
            Input::SelectBluetooth(identifier) => {
                self.bluetooth_identifier = Some(identifier);
                self.request_bluetooth_connection();
            }
        }
    }

    fn request_bluetooth_connection(&self) {
        let (Some(identifier), Some(connect)) = (
            self.bluetooth_identifier.as_ref(),
            self.platform.and_then(|p| p.bluetooth_connect),
        ) else {
            return;
        };
        // SAFETY: `identifier` is a valid C string that outlives the call.
        unsafe { connect(identifier.as_ptr()) };
    }
}

fn transport_name(transport: u8) -> &'static str {
    match transport {
        LEGATO_TRANSPORT_BLUETOOTH => "bluetooth",
        _ => "usb",
    }
}

/// # Safety
/// `ptr` must be null or a valid NUL-terminated string.
unsafe fn string_from(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: guaranteed by the caller.
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

/// MIDI message with the driver timestamp in microseconds (monotonic platform clock).
///
/// # Safety
/// `data` must point to `len` readable bytes (or be null with `len == 0`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn legato_midi_message(
    device: u32,
    time_us: u64,
    data: *const u8,
    len: usize,
) {
    let data = if data.is_null() || len == 0 {
        Vec::new()
    } else {
        // SAFETY: guaranteed by the caller.
        unsafe { std::slice::from_raw_parts(data, len) }.to_vec()
    };
    enqueue(Input::Message {
        device,
        time_us,
        data,
    });
}

/// `transport`: 0 = USB, 1 = Bluetooth.
///
/// # Safety
/// `name` must be null or a valid NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn legato_midi_device_added(device: u32, name: *const c_char, transport: u8) {
    enqueue(Input::DeviceAdded {
        device,
        // SAFETY: guaranteed by the caller.
        name: unsafe { string_from(name) },
        transport,
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn legato_midi_device_removed(device: u32) {
    enqueue(Input::DeviceRemoved { device });
}

/// # Safety
/// `identifier` and `name` must be null or valid NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn legato_midi_bluetooth_found(
    identifier: *const c_char,
    name: *const c_char,
) {
    enqueue(Input::BluetoothFound {
        // SAFETY: guaranteed by the caller.
        identifier: unsafe { string_from(identifier) },
        // SAFETY: guaranteed by the caller.
        name: unsafe { string_from(name) },
    });
}

/// # Safety
/// `platform` must point to a valid `LegatoMidiPlatform`; it is copied.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn legato_midi_register(platform: *const LegatoMidiPlatform) {
    if platform.is_null() {
        return;
    }
    // SAFETY: guaranteed by the caller.
    enqueue(Input::Register(unsafe { *platform }));
}

/// Starts or stops the Bluetooth scan (engine command `scan_bluetooth`).
#[unsafe(no_mangle)]
pub extern "C" fn legato_debug_scan_bluetooth(enabled: bool) {
    enqueue(Input::ScanBluetooth(enabled));
}

/// Selects a Bluetooth device found by the scan (engine command `select_input`).
///
/// # Safety
/// `identifier` must be a valid NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn legato_debug_select_bluetooth(identifier: *const c_char) {
    if identifier.is_null() {
        return;
    }
    // SAFETY: guaranteed by the caller.
    let identifier = unsafe { CStr::from_ptr(identifier) }.to_owned();
    enqueue(Input::SelectBluetooth(identifier));
}

/// Moves logged events as JSON lines into `buffer` and returns the number of bytes written.
/// Events that do not fit stay in the log for the next call.
///
/// # Safety
/// `buffer` must point to `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn legato_debug_drain(buffer: *mut u8, capacity: usize) -> usize {
    if buffer.is_null() {
        return 0;
    }
    // SAFETY: guaranteed by the caller.
    let out = unsafe { std::slice::from_raw_parts_mut(buffer, capacity) };
    let mut log = debug_log().lock().unwrap_or_else(|e| e.into_inner());
    let mut written = 0;
    while let Some(line) = log.front() {
        let end = written + line.len() + 1;
        if end > capacity {
            break;
        }
        out[written..end - 1].copy_from_slice(line.as_bytes());
        out[end - 1] = b'\n';
        written = end;
        log.pop_front();
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{Duration, Instant};

    static CONNECTED: AtomicU32 = AtomicU32::new(0);
    static BLUETOOTH_REQUESTS: AtomicU32 = AtomicU32::new(0);

    unsafe extern "C" fn connect(device: u32) {
        CONNECTED.store(device, Ordering::SeqCst);
    }

    unsafe extern "C" fn bluetooth_connect(_identifier: *const c_char) {
        BLUETOOTH_REQUESTS.fetch_add(1, Ordering::SeqCst);
    }

    fn drain_until(count: usize) -> Vec<Value> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut lines = Vec::new();
        let mut buffer = [0u8; 4096];
        while lines.len() < count && Instant::now() < deadline {
            // SAFETY: buffer is valid for its length.
            let n = unsafe { legato_debug_drain(buffer.as_mut_ptr(), buffer.len()) };
            lines.extend(
                std::str::from_utf8(&buffer[..n])
                    .unwrap()
                    .lines()
                    .map(|l| serde_json::from_str::<Value>(l).unwrap()),
            );
            thread::sleep(Duration::from_millis(5));
        }
        lines
    }

    // One test, because the interface is process-global.
    #[test]
    fn events_are_logged_and_trigger_platform_callbacks() {
        let platform = LegatoMidiPlatform {
            connect: Some(connect),
            disconnect: None,
            send: None,
            bluetooth_scan: None,
            bluetooth_connect: Some(bluetooth_connect),
            prepare_audio_session: None,
            vibrate: None,
        };
        let name = c"Piano";
        let id = c"1234";
        // SAFETY: all pointers are valid for the duration of the calls.
        unsafe {
            legato_midi_register(&platform);
            legato_debug_select_bluetooth(id.as_ptr());
            legato_midi_device_added(7, name.as_ptr(), LEGATO_TRANSPORT_BLUETOOTH);
            legato_midi_message(7, 1_000_000, [0x90, 60, 100].as_ptr(), 3);
        }
        legato_midi_device_removed(7);

        let events = drain_until(3);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0]["kind"], "device_added");
        assert_eq!(events[0]["name"], "Piano");
        assert_eq!(events[0]["transport"], "bluetooth");
        assert_eq!(events[1]["kind"], "message");
        assert_eq!(events[1]["time_us"], 1_000_000);
        assert_eq!(events[1]["data"], json!([0x90, 60, 100]));
        assert_eq!(events[2]["kind"], "device_removed");
        assert_eq!(CONNECTED.load(Ordering::SeqCst), 7);
        // Once on selection, once after the Bluetooth device was lost.
        assert_eq!(BLUETOOTH_REQUESTS.load(Ordering::SeqCst), 2);
    }
}
