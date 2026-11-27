use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
#[cfg(target_os = "windows")]
use std::thread;
use std::thread::JoinHandle;

use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    ShowWindow,
    ToggleGlobalEnabled,
    Quit,
}

#[derive(Default)]
struct TraySharedState {
    tx: Option<Sender<TrayCommand>>,
    repaint_ctx: Option<egui::Context>,
    global_enabled: bool,
}

static TRAY_STATE: std::sync::OnceLock<Mutex<TraySharedState>> = std::sync::OnceLock::new();

fn shared_tray_state() -> &'static Mutex<TraySharedState> {
    TRAY_STATE.get_or_init(|| Mutex::new(TraySharedState::default()))
}

pub struct TrayController {
    rx: Receiver<TrayCommand>,
    #[allow(dead_code)]
    tx: Sender<TrayCommand>,
    repaint_ctx: Arc<Mutex<Option<egui::Context>>>,
    tray_hwnd: Arc<Mutex<usize>>,
    worker: Option<JoinHandle<()>>,
}

impl TrayController {
    pub fn spawn(initial_enabled: bool) -> Self {
        let (tx, rx) = mpsc::channel();
        let repaint_ctx = Arc::new(Mutex::new(None));
        let tray_hwnd = Arc::new(Mutex::new(0usize));

        if let Ok(mut state) = shared_tray_state().lock() {
            state.tx = Some(tx.clone());
            state.global_enabled = initial_enabled;
        }

        #[cfg(target_os = "windows")]
        let worker = {
            let hwnd_ref = Arc::clone(&tray_hwnd);
            thread::Builder::new()
                .name("win32-tray".to_string())
                .spawn(move || {
                    win32_tray::run_tray_loop(hwnd_ref);
                })
                .ok()
        };

        #[cfg(not(target_os = "windows"))]
        let worker: Option<JoinHandle<()>> = None;

        Self {
            rx,
            tx,
            repaint_ctx,
            tray_hwnd,
            worker,
        }
    }

    pub fn attach_repaint_ctx(&self, ctx: egui::Context) {
        if let Ok(mut guard) = self.repaint_ctx.lock() {
            *guard = Some(ctx.clone());
        }
        if let Ok(mut state) = shared_tray_state().lock() {
            state.repaint_ctx = Some(ctx);
        }
    }

    pub fn try_recv(&self) -> Option<TrayCommand> {
        self.rx.try_recv().ok()
    }

    pub fn set_global_enabled(&self, enabled: bool) {
        if let Ok(mut state) = shared_tray_state().lock() {
            state.global_enabled = enabled;
        }
        #[cfg(target_os = "windows")]
        if let Ok(hwnd_val) = self.tray_hwnd.lock() {
            if *hwnd_val != 0 {
                win32_tray::post_update(*hwnd_val);
            }
        }
        let _ = &self.tray_hwnd;
    }
}

impl Drop for TrayController {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if let Ok(hwnd_val) = self.tray_hwnd.lock() {
            if *hwnd_val != 0 {
                win32_tray::post_quit(*hwnd_val);
            }
        }
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

#[allow(dead_code)]
fn emit_tray_command(cmd: TrayCommand) {
    if let Ok(state) = shared_tray_state().lock() {
        if let Some(tx) = &state.tx {
            let _ = tx.send(cmd);
        }
        if let Some(ctx) = &state.repaint_ctx {
            ctx.request_repaint();
        }
    }
}

#[cfg(any(target_os = "windows", test))]
#[allow(dead_code, unused_imports)]
mod win32_tray {
    use super::{emit_tray_command, shared_tray_state, TrayCommand};
    use std::ffi::c_void;
    use std::mem::{size_of, zeroed};
    use std::ptr::{null, null_mut};
    use std::sync::{Arc, Mutex};

    const WM_DESTROY: u32 = 0x0002;
    const WM_USER: u32 = 0x0400;
    const WM_TRAYICON: u32 = WM_USER + 1;
    const WM_APP_TRAY_UPDATE: u32 = 0x8002;
    const WM_APP_TRAY_QUIT: u32 = 0x8003;

    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_LBUTTONDBLCLK: u32 = 0x0203;
    const WM_RBUTTONUP: u32 = 0x0205;
    const WM_CONTEXTMENU: u32 = 0x007B;

    const NIM_ADD: u32 = 0x0000_0000;
    const NIM_MODIFY: u32 = 0x0000_0001;
    const NIM_DELETE: u32 = 0x0000_0002;

    const NIF_MESSAGE: u32 = 0x0000_0001;
    const NIF_ICON: u32 = 0x0000_0002;
    const NIF_TIP: u32 = 0x0000_0004;

    const IDI_APPLICATION: *const u16 = 32512usize as *const u16;

    const MF_STRING: u32 = 0x0000_0000;
    const MF_SEPARATOR: u32 = 0x0000_0800;
    const MF_CHECKED: u32 = 0x0000_0008;
    const MF_UNCHECKED: u32 = 0x0000_0000;

    const TPM_LEFTALIGN: u32 = 0x0000;
    const TPM_BOTTOMALIGN: u32 = 0x0020;
    const TPM_NONOTIFY: u32 = 0x0080;
    const TPM_RETURNCMD: u32 = 0x0100;
    const TPM_RIGHTBUTTON: u32 = 0x0002;

    const CMD_SHOW: usize = 1001;
    const CMD_TOGGLE: usize = 1002;
    const CMD_QUIT: usize = 1003;

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct Msg {
        hwnd: *mut c_void,
        message: u32,
        w_param: usize,
        l_param: isize,
        time: u32,
        pt: Point,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct NotifyIconDataW {
        cb_size: u32,
        h_wnd: *mut c_void,
        u_id: u32,
        u_flags: u32,
        u_callback_message: u32,
        h_icon: *mut c_void,
        sz_tip: [u16; 128],
        dw_state: u32,
        dw_state_mask: u32,
        sz_info: [u16; 256],
        u_timeout_or_version: u32,
        sz_info_title: [u16; 64],
        dw_info_flags: u32,
        guid_item: Guid,
        h_balloon_icon: *mut c_void,
    }

    type WndProc =
        extern "system" fn(hwnd: *mut c_void, msg: u32, w_param: usize, l_param: isize) -> isize;

    #[repr(C)]
    struct WndClassExW {
        cb_size: u32,
        style: u32,
        lpfn_wnd_proc: WndProc,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: *mut c_void,
        h_icon: *mut c_void,
        h_cursor: *mut c_void,
        hbr_background: *mut c_void,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
        h_icon_sm: *mut c_void,
    }

    #[cfg(target_os = "windows")]
    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(lp_module_name: *const u16) -> *mut c_void;
        fn GetLastError() -> u32;
    }

    #[cfg(target_os = "windows")]
    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassExW(lpwcx: *const WndClassExW) -> u16;
        fn CreateWindowExW(
            dw_ex_style: u32,
            lp_class_name: *const u16,
            lp_window_name: *const u16,
            dw_style: u32,
            x: i32,
            y: i32,
            n_width: i32,
            n_height: i32,
            h_wnd_parent: *mut c_void,
            h_menu: *mut c_void,
            h_instance: *mut c_void,
            lp_param: *mut c_void,
        ) -> *mut c_void;
        fn DestroyWindow(h_wnd: *mut c_void) -> i32;
        fn DefWindowProcW(h_wnd: *mut c_void, msg: u32, w_param: usize, l_param: isize) -> isize;
        fn GetMessageW(
            lp_msg: *mut Msg,
            hwnd: *mut c_void,
            w_msg_filter_min: u32,
            w_msg_filter_max: u32,
        ) -> i32;
        fn TranslateMessage(lp_msg: *const Msg) -> i32;
        fn DispatchMessageW(lp_msg: *const Msg) -> isize;
        fn PostMessageW(h_wnd: *mut c_void, msg: u32, w_param: usize, l_param: isize) -> i32;
        fn PostQuitMessage(n_exit_code: i32);
        fn LoadIconW(h_instance: *mut c_void, lp_icon_name: *const u16) -> *mut c_void;
        fn CreatePopupMenu() -> *mut c_void;
        fn AppendMenuW(
            h_menu: *mut c_void,
            u_flags: u32,
            u_id_new_item: usize,
            lp_new_item: *const u16,
        ) -> i32;
        fn DestroyMenu(h_menu: *mut c_void) -> i32;
        fn SetForegroundWindow(h_wnd: *mut c_void) -> i32;
        fn GetCursorPos(lp_point: *mut Point) -> i32;
        fn TrackPopupMenu(
            h_menu: *mut c_void,
            u_flags: u32,
            x: i32,
            y: i32,
            n_reserved: i32,
            h_wnd: *mut c_void,
            prc_rect: *const c_void,
        ) -> i32;
    }

    #[cfg(target_os = "windows")]
    #[link(name = "shell32")]
    extern "system" {
        fn Shell_NotifyIconW(dw_message: u32, lp_data: *const NotifyIconDataW) -> i32;
    }

    fn to_wide_null(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn write_wide_fixed<const N: usize>(buf: &mut [u16; N], s: &str) {
        buf.fill(0);
        for (dst, ch) in buf
            .iter_mut()
            .take(N.saturating_sub(1))
            .zip(s.encode_utf16())
        {
            *dst = ch;
        }
    }

    #[cfg(target_os = "windows")]
    fn build_notify_data(hwnd: *mut c_void, icon: *mut c_void) -> NotifyIconDataW {
        let mut nid: NotifyIconDataW = unsafe { zeroed() };
        nid.cb_size = size_of::<NotifyIconDataW>() as u32;
        nid.h_wnd = hwnd;
        nid.u_id = 1;
        nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.u_callback_message = WM_TRAYICON;
        nid.h_icon = icon;
        let enabled = shared_tray_state()
            .lock()
            .map(|s| s.global_enabled)
            .unwrap_or(true);
        let tip = if enabled {
            "Hotkeys Manager (Active)"
        } else {
            "Hotkeys Manager (Paused)"
        };
        write_wide_fixed(&mut nid.sz_tip, tip);
        nid
    }

    #[cfg(target_os = "windows")]
    fn show_context_menu(hwnd: *mut c_void) {
        unsafe {
            let menu = CreatePopupMenu();
            if menu.is_null() {
                return;
            }
            let enabled = shared_tray_state()
                .lock()
                .map(|s| s.global_enabled)
                .unwrap_or(true);

            let open_lbl = to_wide_null("Open Hotkeys Manager");
            let toggle_lbl = to_wide_null(if enabled {
                "Hotkeys Enabled"
            } else {
                "Hotkeys Paused (Click to Resume)"
            });
            let quit_lbl = to_wide_null("Quit Hotkeys");

            AppendMenuW(menu, MF_STRING, CMD_SHOW, open_lbl.as_ptr());
            AppendMenuW(
                menu,
                MF_STRING | if enabled { MF_CHECKED } else { MF_UNCHECKED },
                CMD_TOGGLE,
                toggle_lbl.as_ptr(),
            );
            AppendMenuW(menu, MF_SEPARATOR, 0, null());
            AppendMenuW(menu, MF_STRING, CMD_QUIT, quit_lbl.as_ptr());

            let mut pt = Point { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            SetForegroundWindow(hwnd);
            let cmd = TrackPopupMenu(
                menu,
                TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
                pt.x,
                pt.y,
                0,
                hwnd,
                null(),
            ) as usize;
            DestroyMenu(menu);

            match cmd {
                CMD_SHOW => emit_tray_command(TrayCommand::ShowWindow),
                CMD_TOGGLE => emit_tray_command(TrayCommand::ToggleGlobalEnabled),
                CMD_QUIT => emit_tray_command(TrayCommand::Quit),
                _ => {}
            }
        }
    }

    #[cfg(target_os = "windows")]
    extern "system" fn tray_wnd_proc(
        hwnd: *mut c_void,
        msg: u32,
        w_param: usize,
        l_param: isize,
    ) -> isize {
        match msg {
            WM_TRAYICON => {
                let event = (l_param as u32) & 0xFFFF;
                match event {
                    WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                        emit_tray_command(TrayCommand::ShowWindow);
                    }
                    WM_RBUTTONUP | WM_CONTEXTMENU => {
                        show_context_menu(hwnd);
                    }
                    _ => {}
                }
                0
            }
            WM_APP_TRAY_UPDATE => {
                let icon = unsafe { LoadIconW(null_mut(), IDI_APPLICATION) };
                let nid = build_notify_data(hwnd, icon);
                unsafe {
                    Shell_NotifyIconW(NIM_MODIFY, &nid);
                }
                0
            }
            WM_APP_TRAY_QUIT => {
                unsafe {
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_DESTROY => {
                let nid = build_notify_data(hwnd, null_mut());
                unsafe {
                    Shell_NotifyIconW(NIM_DELETE, &nid);
                    PostQuitMessage(0);
                }
                0
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
        }
    }

    #[cfg(target_os = "windows")]
    pub fn post_update(hwnd_val: usize) {
        unsafe {
            PostMessageW(hwnd_val as *mut c_void, WM_APP_TRAY_UPDATE, 0, 0);
        }
    }

    #[cfg(target_os = "windows")]
    pub fn post_quit(hwnd_val: usize) {
        unsafe {
            PostMessageW(hwnd_val as *mut c_void, WM_APP_TRAY_QUIT, 0, 0);
        }
    }

    #[cfg(target_os = "windows")]
    pub fn run_tray_loop(hwnd_out: Arc<Mutex<usize>>) {
        unsafe {
            let h_instance = GetModuleHandleW(null());
            let class_name = to_wide_null("HotkeysTrayWindowClass");
            let window_name = to_wide_null("HotkeysTrayWindow");

            let wc = WndClassExW {
                cb_size: size_of::<WndClassExW>() as u32,
                style: 0,
                lpfn_wnd_proc: tray_wnd_proc,
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance,
                h_icon: null_mut(),
                h_cursor: null_mut(),
                hbr_background: null_mut(),
                lpsz_menu_name: null(),
                lpsz_class_name: class_name.as_ptr(),
                h_icon_sm: null_mut(),
            };

            if RegisterClassExW(&wc) == 0 {
                log::warn!("RegisterClassExW for tray failed (err={})", GetLastError());
            }

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                null_mut(),
                null_mut(),
                h_instance,
                null_mut(),
            );

            if hwnd.is_null() {
                log::error!("CreateWindowExW for tray failed (err={})", GetLastError());
                return;
            }

            if let Ok(mut guard) = hwnd_out.lock() {
                *guard = hwnd as usize;
            }

            let icon = LoadIconW(null_mut(), IDI_APPLICATION);
            let nid = build_notify_data(hwnd, icon);
            if Shell_NotifyIconW(NIM_ADD, &nid) == 0 {
                log::warn!("Shell_NotifyIconW(NIM_ADD) failed (err={})", GetLastError());
            } else {
                log::info!("Windows system tray icon added");
            }

            let mut msg: Msg = zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            if let Ok(mut guard) = hwnd_out.lock() {
                *guard = 0;
            }
            log::info!("Windows system tray thread exited");
        }
    }
}
