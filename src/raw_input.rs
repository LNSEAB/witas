use crate::*;
use std::cell::RefCell;
use windows::core::HSTRING;
use windows::Win32::{
    Devices::HumanInterfaceDevice::*,
    Foundation::{CloseHandle, E_FAIL, HANDLE, HWND, LPARAM, LRESULT, WPARAM},
    Storage::FileSystem::{
        CreateFileW, FILE_ACCESS_FLAGS, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ,
        FILE_SHARE_WRITE, OPEN_EXISTING,
    },
    UI::Input::KeyboardAndMouse::VIRTUAL_KEY,
    UI::Input::*,
    UI::WindowsAndMessaging::{
        DefWindowProcW, GIDC_ARRIVAL, GIDC_REMOVAL, RIM_INPUT, RIM_INPUTSINK, RI_KEY_BREAK,
        RI_MOUSE_BUTTON_4_DOWN, RI_MOUSE_BUTTON_4_UP, RI_MOUSE_BUTTON_5_DOWN, RI_MOUSE_BUTTON_5_UP,
        RI_MOUSE_HWHEEL, RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP,
        RI_MOUSE_MIDDLE_BUTTON_DOWN, RI_MOUSE_MIDDLE_BUTTON_UP, RI_MOUSE_RIGHT_BUTTON_DOWN,
        RI_MOUSE_RIGHT_BUTTON_UP, RI_MOUSE_WHEEL, WM_INPUT, WM_INPUT_DEVICE_CHANGE,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct Limit {
    pub min: i64,
    pub max: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WindowState {
    Foreground,
    Background,
}

impl From<WPARAM> for WindowState {
    fn from(src: WPARAM) -> Self {
        match src.0 as _ {
            RIM_INPUT => Self::Foreground,
            RIM_INPUTSINK => Self::Background,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum DeviceType {
    Keyboard,
    Mouse,
    GamePad,
    Other,
}

#[derive(Clone, Debug)]
pub struct DeviceHandle(HANDLE);

impl PartialEq for DeviceHandle {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for DeviceHandle {}

#[derive(Clone, Debug)]
pub struct Device {
    handle: DeviceHandle,
    ty: DeviceType,
    name: String,
}

impl Device {
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub fn device_type(&self) -> DeviceType {
        self.ty
    }

    #[inline]
    pub fn raw_handle(&self) -> HANDLE {
        self.handle.0
    }

    #[inline]
    pub fn get_info(&self) -> Result<DeviceInfo> {
        unsafe { get_device_info(self.handle.0).map_err(|e| e.into()) }
    }
}

impl PartialEq for Device {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Eq for Device {}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl PartialEq<Device> for DeviceHandle {
    #[inline]
    fn eq(&self, other: &Device) -> bool {
        self.0 == other.handle.0
    }
}

impl PartialEq<DeviceHandle> for Device {
    #[inline]
    fn eq(&self, other: &DeviceHandle) -> bool {
        other == self
    }
}

#[derive(Default, Debug)]
pub struct KeyboardInfo {
    pub function_num: u32,
    pub indicator_num: u32,
    pub keys_total: u32,
}

#[derive(Default, Debug)]
pub struct MouseInfo {
    pub button_num: u32,
    pub sample_rate: u32,
    pub has_hwheel: bool,
}

#[derive(Default, Debug)]
pub struct GamePadInfo {
    pub button_num: u32,
    pub x: Option<Limit>,
    pub y: Option<Limit>,
    pub z: Option<Limit>,
    pub rx: Option<Limit>,
    pub ry: Option<Limit>,
    pub rz: Option<Limit>,
    pub hat: Option<Limit>,
}

#[derive(Debug)]
pub enum DeviceInfo {
    Keyboard(KeyboardInfo),
    Mouse(MouseInfo),
    GamePad(GamePadInfo),
}

#[derive(Debug)]
pub struct KeyboardData {
    pub handle: DeviceHandle,
    pub key_code: KeyCode,
    pub key_state: KeyState,
    pub extra: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Relative;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Absolute;

#[derive(Clone, Copy, Debug)]
pub enum MousePosition {
    Relative(Position<i32, Relative>),
    Absolute(Position<i32, Absolute>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MouseButtonStates(u32);

impl MouseButtonStates {
    #[inline]
    pub fn contains(&self, button: MouseButton, state: ButtonState) -> bool {
        match state {
            ButtonState::Pressed => match button {
                MouseButton::Left => (self.0 & RI_MOUSE_LEFT_BUTTON_DOWN) != 0,
                MouseButton::Right => (self.0 & RI_MOUSE_RIGHT_BUTTON_DOWN) != 0,
                MouseButton::Middle => (self.0 & RI_MOUSE_MIDDLE_BUTTON_DOWN) != 0,
                MouseButton::Ex(0) => (self.0 & RI_MOUSE_BUTTON_4_DOWN) != 0,
                MouseButton::Ex(1) => (self.0 & RI_MOUSE_BUTTON_5_DOWN) != 0,
                _ => unimplemented!(),
            },
            ButtonState::Released => match button {
                MouseButton::Left => (self.0 & RI_MOUSE_LEFT_BUTTON_UP) != 0,
                MouseButton::Right => (self.0 & RI_MOUSE_RIGHT_BUTTON_UP) != 0,
                MouseButton::Middle => (self.0 & RI_MOUSE_MIDDLE_BUTTON_UP) != 0,
                MouseButton::Ex(0) => (self.0 & RI_MOUSE_BUTTON_4_UP) != 0,
                MouseButton::Ex(1) => (self.0 & RI_MOUSE_BUTTON_5_UP) != 0,
                _ => unimplemented!(),
            },
        }
    }
}

#[derive(Debug)]
pub struct MouseData {
    pub handle: DeviceHandle,
    pub position: MousePosition,
    pub wheel: Option<i16>,
    pub hwheel: Option<i16>,
    pub buttons: MouseButtonStates,
    pub extra: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Buttons(u64);

impl Buttons {
    fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn contains(&self, index: u64) -> bool {
        assert!(index < 64);
        (self.0 & (1 << index)) != 0
    }
}

#[derive(Debug)]
pub struct GamePadData {
    pub handle: DeviceHandle,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rx: i32,
    pub ry: i32,
    pub rz: i32,
    pub hat: i32,
    pub buttons: Buttons,
}

impl GamePadData {
    fn new(handle: HANDLE) -> Self {
        Self {
            handle: DeviceHandle(handle),
            x: 0,
            y: 0,
            z: 0,
            rx: 0,
            ry: 0,
            rz: 0,
            hat: 0,
            buttons: Buttons::new(),
        }
    }
}

#[derive(Debug)]
pub enum InputData {
    Keyboard(KeyboardData),
    Mouse(MouseData),
    GamePad(GamePadData),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeviceChangeState {
    Arrival,
    Removal,
}

unsafe fn get_preparsed_data(handle: HANDLE, dest: &mut Vec<u8>) -> windows::core::Result<()> {
    let mut len = 0;
    let ret = GetRawInputDeviceInfoW(handle, RIDI_PREPARSEDDATA, None, &mut len);
    if (ret as i32) == -1 {
        return Err(windows::core::Error::from_win32());
    }
    dest.clear();
    dest.resize(len as _, 0);
    let ret = GetRawInputDeviceInfoW(
        handle,
        RIDI_PREPARSEDDATA,
        Some(dest.as_mut_ptr() as _),
        &mut len,
    );
    if (ret as i32) == -1 {
        return Err(windows::core::Error::from_win32());
    }
    Ok(())
}

unsafe fn get_device_name(handle: HANDLE) -> windows::core::Result<HSTRING> {
    let mut len = 0;
    let ret = GetRawInputDeviceInfoW(handle, RIDI_DEVICENAME, None, &mut len);
    if ret != 0 {
        return Err(windows::core::Error::from_win32());
    }
    let mut buffer = vec![0u16; len as usize + 1];
    let ret = GetRawInputDeviceInfoW(
        handle,
        RIDI_DEVICENAME,
        Some(buffer.as_mut_ptr() as _),
        &mut len,
    );
    if (ret as i32) == -1 {
        return Err(windows::core::Error::from_win32());
    }
    Ok(HSTRING::from_wide(&buffer))
}

unsafe fn get_device_product_string(interface: &HSTRING) -> windows::core::Result<String> {
    let handle = CreateFileW(
        interface,
        FILE_ACCESS_FLAGS(0),
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        OPEN_EXISTING,
        FILE_FLAGS_AND_ATTRIBUTES(0),
        HANDLE(0),
    )?;
    let mut buffer = [0u8; 4093 - 1];
    let ret = HidD_GetProductString(handle, buffer.as_mut_ptr() as _, buffer.len() as _);
    CloseHandle(handle);
    if !ret.as_bool() {
        return Err(windows::core::Error::from_win32());
    }
    let buffer = std::slice::from_raw_parts(
        buffer.as_ptr() as *const u16,
        buffer.len() / std::mem::size_of::<u16>(),
    );
    let term = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
    Ok(String::from_utf16_lossy(&buffer[..term]))
}

unsafe fn get_raw_input_device_info(handle: HANDLE) -> windows::core::Result<RID_DEVICE_INFO> {
    let mut len = std::mem::size_of::<RID_DEVICE_INFO>() as u32;
    let mut info = RID_DEVICE_INFO {
        cbSize: len,
        dwType: RID_DEVICE_INFO_TYPE(0),
        Anonymous: RID_DEVICE_INFO_0 {
            keyboard: Default::default(),
        },
    };
    let ret = GetRawInputDeviceInfoW(
        handle,
        RIDI_DEVICEINFO,
        Some(&mut info as *mut _ as _),
        &mut len,
    );
    if (ret as i32) < 0 {
        return Err(windows::core::Error::from_win32());
    }
    Ok(info)
}

unsafe fn get_device_type(handle: HANDLE) -> windows::core::Result<DeviceType> {
    let info = get_raw_input_device_info(handle)?;
    match info.dwType {
        RIM_TYPEKEYBOARD => Ok(DeviceType::Keyboard),
        RIM_TYPEMOUSE => Ok(DeviceType::Mouse),
        RIM_TYPEHID => {
            let hid = info.Anonymous.hid;
            if hid.usUsagePage != HID_USAGE_PAGE_GENERIC {
                return Ok(DeviceType::Other);
            }
            if hid.usUsage != HID_USAGE_GENERIC_JOYSTICK && hid.usUsage != HID_USAGE_GENERIC_GAMEPAD
            {
                return Ok(DeviceType::Other);
            }
            Ok(DeviceType::GamePad)
        }
        _ => Ok(DeviceType::Other),
    }
}

unsafe fn get_button_caps(
    preparsed: &[u8],
    mut len: u16,
) -> windows::core::Result<Vec<HIDP_BUTTON_CAPS>> {
    let mut caps = Vec::with_capacity(len as _);
    caps.resize(len as _, HIDP_BUTTON_CAPS::default());
    HidP_GetButtonCaps(
        HidP_Input,
        caps.as_mut_ptr(),
        &mut len,
        preparsed.as_ptr() as _,
    )?;
    Ok(caps)
}

unsafe fn get_value_caps(
    preparsed: &[u8],
    mut len: u16,
) -> windows::core::Result<Vec<HIDP_VALUE_CAPS>> {
    let mut caps = Vec::with_capacity(len as _);
    caps.resize(len as _, HIDP_VALUE_CAPS::default());
    HidP_GetValueCaps(
        HidP_Input,
        caps.as_mut_ptr(),
        &mut len,
        preparsed.as_ptr() as _,
    )?;
    Ok(caps)
}

unsafe fn get_device_info(handle: HANDLE) -> windows::core::Result<DeviceInfo> {
    let info = get_raw_input_device_info(handle)?;
    let data = match info.dwType {
        RIM_TYPEKEYBOARD => {
            let keyboard = info.Anonymous.keyboard;
            DeviceInfo::Keyboard(KeyboardInfo {
                function_num: keyboard.dwNumberOfFunctionKeys,
                indicator_num: keyboard.dwNumberOfIndicators,
                keys_total: keyboard.dwNumberOfKeysTotal,
            })
        }
        RIM_TYPEMOUSE => {
            let mouse = info.Anonymous.mouse;
            DeviceInfo::Mouse(MouseInfo {
                button_num: mouse.dwNumberOfButtons,
                sample_rate: mouse.dwSampleRate,
                has_hwheel: mouse.fHasHorizontalWheel.as_bool(),
            })
        }
        RIM_TYPEHID => {
            let mut preparsed = vec![];
            get_preparsed_data(handle, &mut preparsed)?;
            let caps = {
                let mut caps = HIDP_CAPS::default();
                HidP_GetCaps(preparsed.as_mut_ptr() as _, &mut caps)?;
                caps
            };
            let button_caps = get_button_caps(&preparsed, caps.NumberInputButtonCaps)?;
            let button_num = if button_caps[0].IsRange.as_bool() {
                let range = button_caps[0].Anonymous.Range;
                (range.UsageMax - range.UsageMin + 1) as u32
            } else {
                0
            };
            let value_caps = get_value_caps(&preparsed, caps.NumberInputValueCaps)?;
            let mut info = GamePadInfo {
                button_num,
                ..Default::default()
            };
            for caps in &value_caps {
                let usage = if !caps.IsRange.as_bool() {
                    caps.Anonymous.NotRange.Usage
                } else {
                    continue;
                };
                let limit = if caps.LogicalMin > caps.LogicalMax {
                    match caps.BitSize {
                        8 => Limit {
                            min: caps.LogicalMin as u8 as _,
                            max: caps.LogicalMax as u8 as _,
                        },
                        16 => Limit {
                            min: caps.LogicalMin as u16 as _,
                            max: caps.LogicalMax as u16 as _,
                        },
                        32 => Limit {
                            min: caps.LogicalMin as u32 as _,
                            max: caps.LogicalMax as u32 as _,
                        },
                        _ => return Err(E_FAIL.into()),
                    }
                } else {
                    Limit {
                        min: caps.LogicalMin as _,
                        max: caps.LogicalMax as _,
                    }
                };
                match usage {
                    0x30 => info.x = Some(limit),
                    0x31 => info.y = Some(limit),
                    0x32 => info.z = Some(limit),
                    0x33 => info.rx = Some(limit),
                    0x34 => info.ry = Some(limit),
                    0x35 => info.rz = Some(limit),
                    0x39 => info.hat = Some(limit),
                    _ => {}
                };
            }
            DeviceInfo::GamePad(info)
        }
        _ => unreachable!(),
    };
    Ok(data)
}

pub fn get_device_list() -> Result<Vec<Device>> {
    unsafe {
        let mut len = 0;
        let ret = GetRawInputDeviceList(
            None,
            &mut len,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as _,
        );
        if (ret as i32) == -1 {
            return Err(Error::from_win32());
        }
        let mut devices = vec![RAWINPUTDEVICELIST::default(); len as usize];
        let ret = GetRawInputDeviceList(
            Some(devices.as_mut_ptr()),
            &mut len,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as _,
        );
        if (ret as i32) == -1 {
            return Err(Error::from_win32());
        }
        let devices = devices
            .iter()
            .filter_map(|device| {
                let device = Device {
                    handle: DeviceHandle(device.hDevice),
                    ty: get_device_type(device.hDevice).ok()?,
                    name: get_device_name(device.hDevice)
                        .and_then(|name| get_device_product_string(&name))
                        .ok()?,
                };
                Some(device)
            })
            .collect::<Vec<_>>();
        Ok(devices)
    }
}

struct GamePadObject {
    handle: DeviceHandle,
    button_caps: Vec<HIDP_BUTTON_CAPS>,
    value_caps: Vec<HIDP_VALUE_CAPS>,
    usage: Vec<u16>,
    preparsed_buffer: Vec<u8>,
}

impl GamePadObject {
    unsafe fn new(handle: HANDLE) -> windows::core::Result<Self> {
        let mut preparsed = vec![];
        get_preparsed_data(handle, &mut preparsed)?;
        let mut caps = HIDP_CAPS::default();
        HidP_GetCaps(preparsed.as_ptr() as _, &mut caps)?;
        let button_caps = get_button_caps(&preparsed, caps.NumberInputButtonCaps)?;
        let value_caps = get_value_caps(&preparsed, caps.NumberInputValueCaps)?;
        let usage_num = HidP_MaxUsageListLength(
            HidP_Input,
            button_caps[0].UsagePage,
            preparsed.as_ptr() as _,
        );
        Ok(Self {
            handle: DeviceHandle(handle),
            button_caps,
            value_caps,
            usage: vec![0u16; usage_num as usize],
            preparsed_buffer: preparsed,
        })
    }
}

thread_local! {
    static GAMEPADS: RefCell<Vec<GamePadObject>> = RefCell::new(vec![]);
}

pub(crate) fn register_devices(hwnd: HWND, state: WindowState) -> Result<()> {
    let flags = RIDEV_DEVNOTIFY
        | if state == WindowState::Background {
            RIDEV_INPUTSINK
        } else {
            RAWINPUTDEVICE_FLAGS(0)
        };
    let devices = [
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_KEYBOARD,
            dwFlags: flags,
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: flags,
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_JOYSTICK,
            dwFlags: flags,
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_GAMEPAD,
            dwFlags: flags,
            hwndTarget: hwnd,
        },
    ];
    unsafe {
        let ret = RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as _);
        if !ret.as_bool() {
            return Err(Error::from_win32());
        }
        let device_list = get_device_list()?;
        GAMEPADS.with(|gamepads| {
            let mut gamepads = gamepads.borrow_mut();
            for device in &device_list {
                if Ok(DeviceType::GamePad) == get_device_type(device.handle.0) {
                    let Ok(obj) = GamePadObject::new(device.handle.0) else { continue };
                    gamepads.push(obj);
                }
            }
        });
        Ok(())
    }
}

unsafe fn input_keyboard_data(input: &mut RAWINPUT) -> InputData {
    let keyboard = input.data.keyboard;
    let handle = input.header.hDevice;
    let key_code = KeyCode {
        vkey: as_virtual_key(VIRTUAL_KEY(keyboard.VKey as _)),
        scan_code: ScanCode(keyboard.MakeCode as _),
    };
    let key_state = if (keyboard.Flags & RI_KEY_BREAK as u16) != 0 {
        KeyState::Released
    } else {
        KeyState::Pressed
    };
    let extra = keyboard.ExtraInformation;
    InputData::Keyboard(KeyboardData {
        handle: DeviceHandle(handle),
        key_code,
        key_state,
        extra,
    })
}

unsafe fn input_mouse_data(input: &mut RAWINPUT) -> InputData {
    let mouse = input.data.mouse;
    let handle = input.header.hDevice;
    let position = if (mouse.usFlags & MOUSE_MOVE_ABSOLUTE as u16) != 0 {
        todo!();
    } else {
        MousePosition::Relative(Position::new(mouse.lLastX, mouse.lLastY))
    };
    let button_flags = mouse.Anonymous.Anonymous.usButtonFlags;
    let wheel = ((button_flags & RI_MOUSE_WHEEL as u16) != 0)
        .then_some(mouse.Anonymous.Anonymous.usButtonData as i16);
    let hwheel = ((button_flags & RI_MOUSE_HWHEEL as u16) != 0)
        .then_some(mouse.Anonymous.Anonymous.usButtonData as i16);
    let buttons = MouseButtonStates(button_flags as _);
    let extra = mouse.ulExtraInformation;
    InputData::Mouse(MouseData {
        handle: DeviceHandle(handle),
        position,
        wheel,
        hwheel,
        buttons,
        extra,
    })
}

unsafe fn input_gamepad_data(input: &mut RAWINPUT) -> windows::core::Result<InputData> {
    let hid = &mut input.data.hid;
    let handle = input.header.hDevice;
    GAMEPADS.with(|gamepads| -> windows::core::Result<InputData> {
        let mut gamepads = gamepads.borrow_mut();
        let Some(gamepad) = gamepads
            .iter_mut()
            .find(|gamepad| gamepad.handle == DeviceHandle(handle)) else { return Err(E_FAIL.into()); };
        let preparsed = &mut gamepad.preparsed_buffer;
        get_preparsed_data(handle, preparsed)?;
        let mut data = GamePadData::new(handle);
        let mut len = gamepad.usage.len() as _;
        let report =
            std::slice::from_raw_parts_mut(hid.bRawData.as_mut_ptr() as _, hid.dwSizeHid as _);
        HidP_GetUsages(
            HidP_Input,
            gamepad.button_caps[0].UsagePage,
            0,
            gamepad.usage.as_mut_ptr(),
            &mut len,
            preparsed.as_ptr() as _,
            report,
        )?;
        let range = if gamepad.button_caps[0].IsRange.as_bool() {
            gamepad.button_caps[0].Anonymous.Range.UsageMin
        } else {
            gamepad.button_caps[0].Anonymous.NotRange.Usage
        };
        for i in 0..(len as usize) {
            data.buttons.0 |= 1 << (gamepad.usage[i] - range);
        }
        for caps in &gamepad.value_caps {
            let usage = if caps.IsRange.as_bool() {
                caps.Anonymous.Range.UsageMin
            } else {
                caps.Anonymous.NotRange.Usage
            };
            let mut value = 0;
            let report =
                std::slice::from_raw_parts_mut(hid.bRawData.as_ptr() as _, hid.dwSizeHid as _);
            let ret = HidP_GetUsageValue(HidP_Input, caps.UsagePage, 0, usage, &mut value, preparsed.as_ptr() as _, report);
            if ret.is_err() {
                continue;
            }
            let value = value as i32;
            match usage {
                0x30 => data.x = value,
                0x31 => data.y = value,
                0x32 => data.z = value,
                0x33 => data.rx = value,
                0x34 => data.ry = value,
                0x35 => data.rz = value,
                0x39 => data.hat = value,
                _ => {}
            }
        }
        Ok(InputData::GamePad(data))
    })
}

thread_local! {
    static RAW_INPUT_DATA_BUFFER: RefCell<Vec<u8>> = RefCell::new(vec![]);
}

#[derive(Debug)]
pub struct DeviceChange {
    pub device: DeviceHandle,
    pub state: DeviceChangeState,
}

#[derive(Debug)]
pub enum RawInputEvent {
    Input(InputData),
    DeviceChange(DeviceChange),
    Quit,
}

pub(crate) unsafe fn on_input(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let input_handle = HRAWINPUT(lparam.0);
    RAW_INPUT_DATA_BUFFER.with(|buffer| {
        let mut data = buffer.borrow_mut();
        let mut len = 0;
        let ret = GetRawInputData(
            input_handle,
            RID_INPUT,
            None,
            &mut len,
            std::mem::size_of::<RAWINPUTHEADER>() as _,
        );
        if (ret as i32) < 0 {
            return DefWindowProcW(hwnd, WM_INPUT, wparam, lparam);
        }
        data.clear();
        data.resize(len as _, 0);
        let ret = GetRawInputData(
            input_handle,
            RID_INPUT,
            Some(data.as_mut_ptr() as _),
            &mut len,
            std::mem::size_of::<RAWINPUTHEADER>() as _,
        );
        if (ret as i32) < 0 {
            return DefWindowProcW(hwnd, WM_INPUT, wparam, lparam);
        }
        let input = (data.as_mut_ptr() as *mut RAWINPUT).as_mut().unwrap();
        let data = match input.header.dwType {
            0 => input_mouse_data(input),
            1 => input_keyboard_data(input),
            2 => match input_gamepad_data(input) {
                Ok(gamepad) => gamepad,
                Err(_) => {
                    return DefWindowProcW(hwnd, WM_INPUT, wparam, lparam);
                }
            },
            _ => unreachable!(),
        };
        Context::send_raw_input_event(hwnd, RawInputEvent::Input(data));
        DefWindowProcW(hwnd, WM_INPUT, wparam, lparam)
    })
}

pub(crate) unsafe fn on_input_device_change(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let def_window_proc = || DefWindowProcW(hwnd, WM_INPUT_DEVICE_CHANGE, wparam, lparam);
    let handle = HANDLE(lparam.0 as _);
    match wparam.0 as u32 {
        GIDC_ARRIVAL => {
            let Ok(ty) = get_device_type(handle) else { return def_window_proc() };
            if ty == DeviceType::GamePad {
                let Ok(obj) = GamePadObject::new(handle) else { return def_window_proc() };
                GAMEPADS.with(move |gamepads| {
                    gamepads.borrow_mut().push(obj);
                });
            }
            Context::send_raw_input_event(
                hwnd,
                RawInputEvent::DeviceChange(DeviceChange {
                    device: DeviceHandle(handle),
                    state: DeviceChangeState::Arrival,
                }),
            );
        }
        GIDC_REMOVAL => {
            GAMEPADS.with(|gamepads| {
                let mut gamepads = gamepads.borrow_mut();
                let index = gamepads
                    .iter()
                    .position(|gamepad| gamepad.handle == DeviceHandle(handle));
                if let Some(index) = index {
                    gamepads.remove(index);
                }
            });
        }
        _ => unreachable!(),
    }
    DefWindowProcW(hwnd, WM_INPUT_DEVICE_CHANGE, wparam, lparam)
}
