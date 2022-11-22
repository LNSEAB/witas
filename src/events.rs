use crate::*;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Draw {
    pub invalid_rect: PhysicalRect<i32>,
}

#[derive(Debug)]
pub struct Moved {
    pub position: ScreenPosition,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResizingEdge {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomLRight,
}

#[derive(Debug)]
pub struct Resizing {
    pub size: PhysicalSize<u32>,
    pub edge: ResizingEdge,
}

#[derive(Debug)]
pub struct Resized {
    pub size: PhysicalSize<u32>,
}

#[derive(Debug)]
pub struct MouseInput {
    pub button: MouseButton,
    pub button_state: ButtonState,
    pub mouse_state: MouseState,
}

#[derive(Debug)]
pub struct CursorMoved {
    pub mouse_state: MouseState,
}

#[derive(Debug)]
pub struct CursorEntered {
    pub mouse_state: MouseState,
}

#[derive(Debug)]
pub struct CursorLeft {
    pub mouse_state: MouseState,
}

#[derive(Debug)]
pub struct MouseWheel {
    pub axis: MouseWheelAxis,
    pub distance: i32,
    pub mouse_state: MouseState,
}

#[derive(Debug)]
pub struct KeyInput {
    pub key_code: KeyCode,
    pub key_state: KeyState,
    pub prev_pressed: bool,
}

#[derive(Debug)]
pub struct CharInput {
    pub c: char,
}

#[derive(Debug)]
pub struct Maximized {
    pub size: PhysicalSize<u32>,
}

#[derive(Debug)]
pub struct Restored {
    pub size: PhysicalSize<u32>,
}

pub struct ImeStartComposition {
    position: PhysicalPosition<i32>,
    tx: Sender<PhysicalPosition<i32>>,
}

impl ImeStartComposition {
    pub(crate) fn new(tx: Sender<PhysicalPosition<i32>>) -> Self {
        Self {
            position: (0, 0).into(),
            tx,
        }
    }

    #[inline]
    pub fn set_position(&mut self, position: impl Into<PhysicalPosition<i32>>) {
        self.position = position.into();
    }
}

impl Drop for ImeStartComposition {
    fn drop(&mut self) {
        self.tx.send(self.position).unwrap_or(());
    }
}

impl std::fmt::Debug for ImeStartComposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.position)
    }
}

#[derive(Debug)]
pub struct ImeComposition {
    pub chars: Vec<char>,
    pub clauses: Vec<ime::Clause>,
    pub cursor_position: usize,
}

impl ImeComposition {
    pub(crate) fn new(imc: &ime::Imc) -> Option<Self> {
        let s = imc.get_composition_string()?;
        let clauses = imc.get_composition_clauses()?;
        Some(Self {
            chars: s.chars().collect(),
            clauses,
            cursor_position: imc.get_cursor_position(),
        })
    }
}

#[derive(Debug)]
pub struct ImeEndComposition {
    pub result: Option<String>,
}

#[derive(Debug)]
pub struct DpiChanged {
    pub new_dpi: u32,
}

#[derive(Debug)]
pub struct DropFiles {
    pub paths: Vec<PathBuf>,
    pub position: PhysicalPosition<i32>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    Activated,
    Inactivated,
    Draw(Draw),
    Moved(Moved),
    Resizing(Resizing),
    Resized(Resized),
    MouseInput(MouseInput),
    CursorMoved(CursorMoved),
    CursorEntered(CursorEntered),
    CursorLeft(CursorLeft),
    MouseWheel(MouseWheel),
    KeyInput(KeyInput),
    CharInput(CharInput),
    ImeStartComposition(ImeStartComposition),
    ImeComposition(ImeComposition),
    ImeEndComposition(ImeEndComposition),
    Minimized,
    Maximized(Maximized),
    Restored(Restored),
    DpiChanged(DpiChanged),
    DropFiles(DropFiles),
    Closed,
    Quit,
}
