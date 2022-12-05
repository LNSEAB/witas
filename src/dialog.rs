use crate::*;
use std::path::PathBuf;
use tokio::sync::oneshot;
use windows::core::{Interface, HSTRING, PCWSTR, PWSTR};
use windows::Win32::{
    Foundation::{ERROR_CANCELLED, E_FAIL, HWND},
    System::Com::*,
    UI::Shell::Common::*,
    UI::Shell::*,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FileDialogOptions(pub u32);

impl FileDialogOptions {
    pub const OVERWRITE_PROMPT: Self = FileDialogOptions(FOS_OVERWRITEPROMPT.0);
    pub const STRICT_FILE_TYPES: Self = FileDialogOptions(FOS_STRICTFILETYPES.0);
    pub const NO_CHANGE_DIR: Self = FileDialogOptions(FOS_NOCHANGEDIR.0);
    pub const PICK_FOLDERS: Self = FileDialogOptions(FOS_PICKFOLDERS.0);
    pub const FORCE_FILE_SYSTEM: Self = FileDialogOptions(FOS_FORCEFILESYSTEM.0);
    pub const ALL_NON_STORAGE_ITEMS: Self = FileDialogOptions(FOS_ALLNONSTORAGEITEMS.0);
    pub const NO_VALIDATE: Self = FileDialogOptions(FOS_NOVALIDATE.0);
    const ALLOW_MULTI_SELECT: Self = FileDialogOptions(FOS_ALLOWMULTISELECT.0);
    pub const PATH_MUST_EXIST: Self = FileDialogOptions(FOS_PATHMUSTEXIST.0);
    pub const FILE_MUST_EXIST: Self = FileDialogOptions(FOS_FILEMUSTEXIST.0);
    pub const CREATE_PROMPT: Self = FileDialogOptions(FOS_CREATEPROMPT.0);
    pub const SHARE_AWARE: Self = FileDialogOptions(FOS_SHAREAWARE.0);
    pub const NO_READONLY_RETURN: Self = FileDialogOptions(FOS_NOREADONLYRETURN.0);
    pub const NO_TEST_FILE_CREATE: Self = FileDialogOptions(FOS_NOTESTFILECREATE.0);
    pub const HIDE_MRU_PLACES: Self = FileDialogOptions(FOS_HIDEMRUPLACES.0);
    pub const HIDE_PINNED_PLACES: Self = FileDialogOptions(FOS_HIDEPINNEDPLACES.0);
    pub const NODE_REFERENCE_LINKS: Self = FileDialogOptions(FOS_NODEREFERENCELINKS.0);
    pub const OK_BUTTON_NEED_SINTERACTION: Self = FileDialogOptions(FOS_OKBUTTONNEEDSINTERACTION.0);
    pub const DONT_ADD_TO_RECENT: Self = FileDialogOptions(FOS_DONTADDTORECENT.0);
    pub const FORCE_SHOW_HIDDEN: Self = FileDialogOptions(FOS_FORCESHOWHIDDEN.0);
    pub const DEFAULT_NO_MINI_MODE: Self = FileDialogOptions(FOS_DEFAULTNOMINIMODE.0);
    pub const FORCE_PREVIEW_PANE_ON: Self = FileDialogOptions(FOS_FORCEPREVIEWPANEON.0);
    pub const SUPPORT_STREAMABLE_ITEMS: Self = FileDialogOptions(FOS_SUPPORTSTREAMABLEITEMS.0);
}

impl std::ops::BitAnd for FileDialogOptions {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for FileDialogOptions {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitXor for FileDialogOptions {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitAndAssign for FileDialogOptions {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOrAssign for FileDialogOptions {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitXorAssign for FileDialogOptions {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl From<FileDialogOptions> for FILEOPENDIALOGOPTIONS {
    fn from(src: FileDialogOptions) -> Self {
        Self(src.0)
    }
}

pub struct FilterSpec {
    pub name: String,
    pub spec: String,
}

impl FilterSpec {
    pub fn new(name: impl Into<String>, spec: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            spec: spec.into(),
        }
    }
}

pub trait FilterSpecs {
    fn to_vec(self) -> Vec<FilterSpec>;
}

impl FilterSpecs for Vec<FilterSpec> {
    #[inline]
    fn to_vec(self) -> Vec<FilterSpec> {
        self
    }
}

impl<T, U> FilterSpecs for Vec<(T, U)>
where
    T: Into<String>,
    U: Into<String>,
{
    #[inline]
    fn to_vec(self) -> Vec<FilterSpec> {
        self.into_iter()
            .map(|spec| FilterSpec::new(spec.0, spec.1))
            .collect()
    }
}

impl<T, U, const N: usize> FilterSpecs for [(T, U); N]
where
    T: Into<String>,
    U: Into<String>,
{
    #[inline]
    fn to_vec(self) -> Vec<FilterSpec> {
        self.into_iter()
            .map(|spec| FilterSpec::new(spec.0, spec.1))
            .collect()
    }
}

struct DisplayName(PWSTR);

impl DisplayName {
    fn to_path_buf(&self) -> Result<PathBuf> {
        unsafe {
            let len = (0..isize::MAX)
                .position(|i| *self.0 .0.offset(i) == 0)
                .ok_or(windows::core::Error::from(E_FAIL))?;
            let slice = std::slice::from_raw_parts(self.0 .0, len);
            let path: PathBuf = String::from_utf16_lossy(slice).into();
            Ok(path)
        }
    }
}

impl Drop for DisplayName {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(Some(self.0 .0 as _));
        }
    }
}

trait DisplayNameWrapper {
    fn display_name(&self) -> Result<PathBuf>;
}

impl DisplayNameWrapper for IShellItem {
    fn display_name(&self) -> Result<PathBuf> {
        unsafe { DisplayName(self.GetDisplayName(SIGDN_FILESYSPATH)?).to_path_buf() }
    }
}

pub trait OpenDialogResult: Sized + Send {
    const OPTIONS: FileDialogOptions;

    fn get_result(dialog: &IFileOpenDialog) -> Result<Self>;
}

impl OpenDialogResult for PathBuf {
    const OPTIONS: FileDialogOptions = FileDialogOptions(0);

    fn get_result(dialog: &IFileOpenDialog) -> Result<Self> {
        unsafe {
            let result = dialog.GetResult()?;
            let result = result.display_name()?;
            Ok(result.canonicalize()?)
        }
    }
}

impl OpenDialogResult for Vec<PathBuf> {
    const OPTIONS: FileDialogOptions = FileDialogOptions::ALLOW_MULTI_SELECT;

    fn get_result(dialog: &IFileOpenDialog) -> Result<Self> {
        unsafe {
            let result = dialog.GetResults()?;
            let len = result.GetCount()?;
            let mut paths: Self = Vec::with_capacity(len as usize);
            for i in 0..len {
                let Ok(item) = result.GetItemAt(i) else { continue };
                let Ok(path) = item.display_name() else { continue };
                paths.push(path.canonicalize()?);
            }
            Ok(paths)
        }
    }
}

pub struct FileDialogParams {
    title: Option<String>,
    ok_button_label: Option<String>,
    default_folder: Option<PathBuf>,
    default_extension: Option<String>,
    file_name_label: Option<String>,
    file_types: Vec<FilterSpec>,
    file_type_index: usize,
    options: FileDialogOptions,
    owner: Option<HWND>,
}

impl Default for FileDialogParams {
    #[inline]
    fn default() -> Self {
        Self {
            title: None,
            ok_button_label: None,
            default_folder: None,
            default_extension: None,
            file_name_label: None,
            file_types: vec![],
            file_type_index: 1,
            options: FileDialogOptions(0),
            owner: None,
        }
    }
}

fn show_dialog<T>(dialog: &T, params: FileDialogParams) -> Result<()>
where
    T: Interface,
{
    unsafe {
        let dialog: IFileDialog = dialog.cast()?;
        if let Some(title) = params.title {
            dialog.SetTitle(&HSTRING::from(title))?;
        }
        if let Some(label) = params.ok_button_label {
            dialog.SetOkButtonLabel(&HSTRING::from(label))?;
        }
        if let Some(path) = params.default_folder {
            let path = path.canonicalize()?;
            let path = path.to_string_lossy();
            let path = path.as_ref().trim_start_matches(r"\\?\");
            let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(path), None)?;
            dialog.SetDefaultFolder(&item)?;
        }
        if let Some(ext) = params.default_extension {
            dialog.SetDefaultExtension(&HSTRING::from(ext))?;
        }
        if let Some(label) = params.file_name_label {
            dialog.SetFileNameLabel(&HSTRING::from(label))?;
        }
        if !params.file_types.is_empty() {
            assert!(params.file_types.len() <= u32::MAX as usize);
            let buffer = params
                .file_types
                .iter()
                .map(|ft| (HSTRING::from(&ft.name), HSTRING::from(&ft.spec)))
                .collect::<Vec<_>>();
            let file_types = buffer
                .iter()
                .map(|(name, spec)| COMDLG_FILTERSPEC {
                    pszName: PCWSTR(name.as_ptr()),
                    pszSpec: PCWSTR(spec.as_ptr()),
                })
                .collect::<Vec<_>>();
            dialog.SetFileTypes(&file_types)?;
            dialog.SetFileTypeIndex(params.file_type_index as _)?;
        }
        dialog.SetOptions(params.options.into())?;
        dialog.Show(None).map_err(|e| e.into())
    }
}

pub struct FileOpenDialog<T = ()> {
    params: FileDialogParams,
    _t: std::marker::PhantomData<T>,
}

impl FileOpenDialog<()> {
    #[inline]
    pub fn new() -> FileOpenDialog<PathBuf> {
        FileOpenDialog {
            params: FileDialogParams {
                options: FileDialogOptions::PATH_MUST_EXIST | FileDialogOptions::FILE_MUST_EXIST,
                ..Default::default()
            },
            _t: std::marker::PhantomData,
        }
    }
}

impl<T> FileOpenDialog<T>
where
    T: OpenDialogResult + 'static,
{
    #[inline]
    pub fn allow_multi_select(self) -> FileOpenDialog<Vec<PathBuf>> {
        FileOpenDialog {
            params: self.params,
            _t: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.params.title = Some(title.into());
        self
    }

    #[inline]
    pub fn ok_button_label(mut self, label: impl Into<String>) -> Self {
        self.params.ok_button_label = Some(label.into());
        self
    }

    #[inline]
    pub fn default_folder(mut self, path: impl Into<PathBuf>) -> Self {
        self.params.default_folder = Some(path.into());
        self
    }

    #[inline]
    pub fn default_extension(mut self, ext: impl Into<String>) -> Self {
        self.params.default_extension = Some(ext.into());
        self
    }

    #[inline]
    pub fn file_name_label(mut self, label: impl Into<String>) -> Self {
        self.params.file_name_label = Some(label.into());
        self
    }

    #[inline]
    pub fn file_types(mut self, file_types: impl FilterSpecs) -> Self {
        self.params.file_types = file_types.to_vec();
        self
    }

    #[inline]
    pub fn file_type_index(mut self, index: usize) -> Self {
        self.params.file_type_index = index;
        self
    }

    #[inline]
    pub fn options(mut self, options: FileDialogOptions) -> Self {
        self.params.options = options;
        self
    }

    #[inline]
    pub fn owner(mut self, window: &Window) -> Self {
        self.params.owner = Some(HWND(window.raw_handle() as _));
        self
    }

    pub async fn show(mut self) -> Result<Option<T>> {
        let (tx, rx): (oneshot::Sender<Result<T>>, _) = oneshot::channel();
        self.params.options |= T::OPTIONS;
        UiThread::send_task(move || {
            let dialog: IFileOpenDialog = unsafe {
                match CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER) {
                    Ok(dialog) => dialog,
                    Err(e) => {
                        tx.send(Err(e.into())).unwrap_or(());
                        return;
                    }
                }
            };
            if let Err(e) = show_dialog(&dialog, self.params) {
                tx.send(Err(e)).unwrap_or(());
                return;
            }
            tx.send(T::get_result(&dialog)).unwrap_or(());
        });
        match rx.await {
            Ok(ret) => match ret {
                Ok(ret) => Ok(Some(ret)),
                Err(e) => match e {
                    Error::Api(e) if e.code() == ERROR_CANCELLED.into() => Ok(None),
                    _ => Err(e),
                },
            },
            Err(_) => Err(Error::UiThreadClosed),
        }
    }
}

pub struct FileSaveDialog {
    params: FileDialogParams,
}

impl FileSaveDialog {
    #[inline]
    pub fn new() -> Self {
        Self {
            params: FileDialogParams {
                options: FileDialogOptions::PATH_MUST_EXIST
                    | FileDialogOptions::NO_READONLY_RETURN
                    | FileDialogOptions::OVERWRITE_PROMPT,
                ..Default::default()
            },
        }
    }

    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.params.title = Some(title.into());
        self
    }

    #[inline]
    pub fn ok_button_label(mut self, label: impl Into<String>) -> Self {
        self.params.ok_button_label = Some(label.into());
        self
    }

    #[inline]
    pub fn default_folder(mut self, path: impl Into<PathBuf>) -> Self {
        self.params.default_folder = Some(path.into());
        self
    }

    #[inline]
    pub fn default_extension(mut self, ext: impl Into<String>) -> Self {
        self.params.default_extension = Some(ext.into());
        self
    }

    #[inline]
    pub fn file_name_label(mut self, label: impl Into<String>) -> Self {
        self.params.file_name_label = Some(label.into());
        self
    }

    #[inline]
    pub fn file_types(mut self, file_types: impl FilterSpecs) -> Self {
        self.params.file_types = file_types.to_vec();
        self
    }

    #[inline]
    pub fn file_type_index(mut self, index: usize) -> Self {
        self.params.file_type_index = index;
        self
    }

    #[inline]
    pub fn options(mut self, options: FileDialogOptions) -> Self {
        self.params.options = options;
        self
    }

    #[inline]
    pub fn owner(mut self, window: &Window) -> Self {
        self.params.owner = Some(HWND(window.raw_handle() as _));
        self
    }

    pub async fn show(self) -> Result<Option<PathBuf>> {
        let (tx, rx): (oneshot::Sender<Result<PathBuf>>, _) = oneshot::channel();
        UiThread::send_task(move || {
            let dialog: IFileSaveDialog = unsafe {
                match CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER) {
                    Ok(dialog) => dialog,
                    Err(e) => {
                        tx.send(Err(e.into())).unwrap_or(());
                        return;
                    }
                }
            };
            if let Err(e) = show_dialog(&dialog, self.params) {
                tx.send(Err(e)).unwrap_or(());
                return;
            }
            unsafe {
                let result = match dialog.GetResult() {
                    Ok(result) => result,
                    Err(e) => {
                        tx.send(Err(e.into())).unwrap_or(());
                        return;
                    }
                };
                match result.display_name() {
                    Ok(path) => {
                        tx.send(Ok(path)).unwrap_or(());
                    }
                    Err(e) => {
                        tx.send(Err(e)).unwrap_or(());
                    }
                }
            }
        });
        match rx.await {
            Ok(ret) => match ret {
                Ok(ret) => Ok(Some(ret)),
                Err(e) => match e {
                    Error::Api(e) if e.code() == ERROR_CANCELLED.into() => Ok(None),
                    _ => Err(e),
                },
            },
            Err(_) => Err(Error::UiThreadClosed),
        }
    }
}
