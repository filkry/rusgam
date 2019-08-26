#![allow(dead_code)]

use safewindows;

pub struct SWinAPI {
    wapi: safewindows::SWinAPI,
    frequency: i64,
}

impl SWinAPI {
    pub unsafe fn unsafecurtimemicroseconds() -> i64 {
        let pc = safewindows::SWinAPI::queryperformancecounter();
        let fc = safewindows::SWinAPI::queryperformancefrequencycounter();

        pc / fc
    }

    pub fn curtimemicroseconds(&self) -> i64 {
        let pc = safewindows::SWinAPI::queryperformancecounter();

        pc / self.frequency
    }

    pub fn create() -> SWinAPI {
        SWinAPI{
            // -- $$$FRK(TODO): not very rusty
            wapi: safewindows::initwinapi().unwrap(),
            frequency: unsafe { safewindows::SWinAPI::queryperformancefrequencycounter() },
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
}

impl SWindow {
    pub fn create(windowclass: &safewindows::SWindowClass, title: &str, width: u32, height: u32) -> SWindow {
        // -- $$$FRK(TODO): all these unwraps are not very safee
        let safewindow = windowclass.createwindow(title, width, height).unwrap();
        SWindow {
            w: safewindow,
        }
    }

    pub fn dummyrepaint(&mut self) {
        self.w.beginpaint();
        self.w.endpaint();
    }

    pub fn processmessage<'a>(&mut self, windowproc: &'a mut dyn safewindows::TWindowProc) -> bool {
        match self.w.peekmessage(windowproc) {
            Some(mut m) => {
                self.w.translatemessage(&mut m);
                self.w.dispatchmessage(&mut m, windowproc);
                true
            }
            None => false
        }
    }

    pub fn raw(&self) -> &safewindows::SWindow {
        &self.w
    }
    pub fn rawmut(&mut self) -> &mut safewindows::SWindow {
        &mut self.w
    }
}