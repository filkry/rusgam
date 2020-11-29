#![allow(dead_code)]

use safewindows;

use std::ops::{Deref, DerefMut};

pub static winapi : SWinAPI = SWinAPI::create();

pub struct SWinAPI {
    wapi: safewindows::SWinAPI,
    frequency: i64,
}

impl SWinAPI {
    pub unsafe fn unsafecurtimemicroseconds() -> i64 {
        let pc = safewindows::SWinAPI::queryperformancecounter();
        let fc = safewindows::SWinAPI::queryperformancefrequencycounter();

        pc / (fc / 1_000_000)
    }

    pub fn curtimemicroseconds(&self) -> i64 {
        let pc = safewindows::SWinAPI::queryperformancecounter();

        pc / self.frequency
    }

    pub fn create() -> SWinAPI {
        SWinAPI {
            // -- $$$FRK(TODO): not very rusty
            wapi: safewindows::initwinapi().unwrap(),
            frequency: unsafe {
                safewindows::SWinAPI::queryperformancefrequencycounter() / 1_000_000
            },
        }
    }

    pub fn rawwinapi(&self) -> &safewindows::SWinAPI {
        &self.wapi
    }
    pub fn rawwinapimut(&mut self) -> &mut safewindows::SWinAPI {
        &mut self.wapi
    }
}

pub struct SWindow {
    w: safewindows::SWindow,
    windowproc: SWindowProc,
}

impl SWindow {
    pub fn create(
        windowclass: &safewindows::SWindowClass,
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<SWindow, safewindows::SErr> {
        let safewindow = windowclass.createwindow(title, width, height)?;
        Ok(SWindow {
            w: safewindow,
            windowproc: SWindowProc {
                pendingmsgs: std::collections::VecDeque::new(),
            },
        })
    }

    pub fn dummyrepaint(&mut self) {
        self.w.beginpaint();
        self.w.endpaint();
    }

    pub fn pollmessage(&mut self) -> Option<safewindows::EMsgType> {
        if let Some(msg) = self.windowproc.pendingmsgs.pop_front() {
            return Some(msg);
        }

        match self.w.peekmessage(&mut self.windowproc) {
            Some(mut m) => {
                self.w.translatemessage(&mut m);
                self.w.dispatchmessage(&mut m, &mut self.windowproc);
                self.windowproc.pendingmsgs.pop_front()
            }
            None => None,
        }
    }

    pub fn raw(&self) -> &safewindows::SWindow {
        &self.w
    }
    pub fn rawmut(&mut self) -> &mut safewindows::SWindow {
        &mut self.w
    }
}

impl Deref for SWindow {
    type Target = safewindows::SWindow;

    fn deref(&self) -> &Self::Target {
        &self.w
    }
}

impl DerefMut for SWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.w
    }
}

pub struct SWindowProc {
    // -- $$$FRK(TODO) allocations
    pendingmsgs: std::collections::VecDeque<safewindows::EMsgType>,
}

impl safewindows::TWindowProc for SWindowProc {
    fn windowproc(&mut self, _window: &mut safewindows::SWindow, msg: safewindows::EMsgType) -> () {
        self.pendingmsgs.push_back(msg);
    }
}
