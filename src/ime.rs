use crate::utility::*;
use crate::*;
use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    Globalization::*,
    UI::Input::Ime::*,
};

pub(crate) struct ImmContext {
    hwnd: HWND,
    himc: HIMC,
}

impl ImmContext {
    pub fn new(hwnd: HWND) -> Self {
        unsafe {
            let himc = ImmCreateContext();
            ImmAssociateContextEx(hwnd, himc, IACE_CHILDREN);
            Self { hwnd, himc }
        }
    }

    pub fn enable(&self) {
        unsafe {
            ImmAssociateContextEx(self.hwnd, self.himc, IACE_CHILDREN);
        }
    }

    pub fn disable(&self) {
        unsafe {
            ImmAssociateContextEx(self.hwnd, HIMC(0), IACE_IGNORENOCONTEXT);
        }
    }
}

impl Drop for ImmContext {
    fn drop(&mut self) {
        unsafe {
            ImmAssociateContextEx(self.hwnd, HIMC(0), IACE_DEFAULT);
            ImmDestroyContext(self.himc);
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Clause {
    pub range: std::ops::Range<usize>,
    pub target: bool,
}

pub(crate) struct Imc {
    hwnd: HWND,
    himc: HIMC,
}

impl Imc {
    pub fn get(hwnd: HWND) -> Self {
        let himc = unsafe { ImmGetContext(hwnd) };
        Self { hwnd, himc }
    }

    pub fn set_candidate_window_position(
        &self,
        position: impl ToPhysical<i32, Output<i32> = PhysicalPosition<i32>>,
        enable_exclude_rect: bool,
    ) {
        let position = position.to_physical(get_dpi_for_window(self.hwnd) as _);
        let position = POINT {
            x: position.x,
            y: position.y,
        };
        let form = CANDIDATEFORM {
            dwStyle: CFS_CANDIDATEPOS,
            dwIndex: 0,
            ptCurrentPos: position,
            ..Default::default()
        };
        unsafe {
            ImmSetCandidateWindow(self.himc, &form);
        }
        if !enable_exclude_rect {
            let form = CANDIDATEFORM {
                dwStyle: CFS_EXCLUDE,
                dwIndex: 0,
                ptCurrentPos: position,
                rcArea: RECT {
                    left: position.x,
                    top: position.y,
                    right: position.x,
                    bottom: position.y,
                },
            };
            unsafe {
                ImmSetCandidateWindow(self.himc, &form);
            }
        }
    }

    pub fn get_composition_string(&self) -> Option<String> {
        unsafe {
            let byte_len = ImmGetCompositionStringW(self.himc, GCS_COMPSTR, None, 0);
            if byte_len == IMM_ERROR_NODATA || byte_len == IMM_ERROR_GENERAL {
                return None;
            }
            let len = byte_len as usize / std::mem::size_of::<u16>();
            let mut buf = Vec::with_capacity(len);
            buf.resize(len, 0);
            ImmGetCompositionStringW(
                self.himc,
                GCS_COMPSTR,
                Some(buf.as_mut_ptr() as _),
                byte_len as _,
            );
            let s = String::from_utf16_lossy(&buf);
            (!s.is_empty()).then_some(s)
        }
    }

    pub fn get_composition_clauses(&self) -> Option<Vec<Clause>> {
        let targets: Vec<bool> = unsafe {
            let byte_len = ImmGetCompositionStringW(self.himc, GCS_COMPATTR, None, 0);
            if byte_len == IMM_ERROR_NODATA || byte_len == IMM_ERROR_GENERAL {
                return None;
            }
            let mut buf = Vec::<u8>::with_capacity(byte_len as _);
            buf.resize(byte_len as _, 0);
            ImmGetCompositionStringW(
                self.himc,
                GCS_COMPATTR,
                Some(buf.as_mut_ptr() as _),
                byte_len as _,
            );
            buf.into_iter()
                .map(|a| a as u32 == ATTR_TARGET_CONVERTED)
                .collect()
        };
        let clauses: Vec<std::ops::Range<usize>> = unsafe {
            let byte_len = ImmGetCompositionStringW(self.himc, GCS_COMPCLAUSE, None, 0);
            if byte_len == IMM_ERROR_NODATA || byte_len == IMM_ERROR_GENERAL {
                return None;
            }
            let mut buf = Vec::<u8>::with_capacity(byte_len as _);
            buf.resize(byte_len as _, 0);
            ImmGetCompositionStringW(
                self.himc,
                GCS_COMPCLAUSE,
                Some(buf.as_mut_ptr() as _),
                byte_len as _,
            );
            let buf = std::slice::from_raw_parts(
                buf.as_ptr() as *const u32,
                byte_len as usize / std::mem::size_of::<u32>(),
            );
            buf.windows(2)
                .map(|a| a[0] as usize..a[1] as usize)
                .collect()
        };
        Some(
            clauses
                .into_iter()
                .map(|r| Clause {
                    target: targets[r.start],
                    range: r,
                })
                .collect(),
        )
    }

    pub fn get_composition_result(&self) -> Option<String> {
        unsafe {
            let byte_len = ImmGetCompositionStringW(self.himc, GCS_RESULTSTR, None, 0);
            if byte_len == IMM_ERROR_NODATA || byte_len == IMM_ERROR_GENERAL {
                return None;
            }
            let len = byte_len as usize / std::mem::size_of::<u16>();
            let mut buf = Vec::with_capacity(len);
            buf.resize(len, 0);
            ImmGetCompositionStringW(
                self.himc,
                GCS_RESULTSTR,
                Some(buf.as_mut_ptr() as _),
                byte_len as _,
            );
            let s = String::from_utf16_lossy(&buf);
            (!s.is_empty()).then_some(s)
        }
    }

    pub fn get_cursor_position(&self) -> usize {
        unsafe { ImmGetCompositionStringW(self.himc, GCS_CURSORPOS, None, 0) as usize }
    }
}

impl Drop for Imc {
    fn drop(&mut self) {
        unsafe {
            ImmReleaseContext(self.hwnd, self.himc);
        }
    }
}
