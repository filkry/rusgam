extern crate winapi;
extern crate wio;

//mod math;
mod collections;
mod rusd3d12;
mod safewindows;
mod rustywindows;

macro_rules! properror {
    ($result:expr) => {
        match $result {
            Ok(a) => a,
            Err(e) => {
                return Err(e);
            }
        }
    };
}

pub struct SWindowProc {
    quit: bool,
}

impl SWindowProc {
    fn shouldquit(&self) -> bool {
        self.quit
    }
}

impl safewindows::TWindowProc for SWindowProc {
    fn windowproc(&mut self, _window: &mut safewindows::SWindow, msg: safewindows::EMsgType) -> () {
        match msg {
            safewindows::EMsgType::Paint => {
                // -- $$$FRK(FUCK MY LIFE): here we are again, can't build on top of this
                //window.dummyrepaint();
            }
            safewindows::EMsgType::KeyDown { key } => match key {
                safewindows::EKey::Q => {
                    self.quit = true;
                    println!("Q keydown");
                }
                _ => (),
            },
            safewindows::EMsgType::Size {
                width: _,
                height: _,
            } => {
                println!("Size");
            }
            safewindows::EMsgType::Invalid => (),
        }
    }
}

struct SCommandQueue {
    q: rusd3d12::SCommandQueue,
    fence: rusd3d12::SFence,
    fenceevent: safewindows::SEventHandle,
    nextfencevalue: u64,
}

impl SCommandQueue {
    pub fn createcommandqueue(
        winapi: &safewindows::SWinAPI,
        device: &mut rusd3d12::SDevice,
    ) -> Result<SCommandQueue, &'static str> {
        let qresult = device
            .createcommandqueue(rusd3d12::ECommandListType::Direct)
            .unwrap();
        Ok(SCommandQueue {
            q: qresult,
            fence: device.createfence().unwrap(),
            fenceevent: winapi.createeventhandle().unwrap(),
            nextfencevalue: 0,
        })
    }

    pub fn pushsignal(&mut self) -> Result<u64, &'static str> {
        self.nextfencevalue += 1;
        self.q.pushsignal(&self.fence, self.nextfencevalue)
    }

    pub fn waitforfencevalue(&self, val: u64) {
        self.fence
            .waitforvalue(val, &self.fenceevent, <u64>::max_value())
            .unwrap();
    }

    pub fn flushblocking(&mut self) -> Result<(), &'static str> {
        let lastfencevalue = properror!(self.pushsignal());
        self.waitforfencevalue(lastfencevalue);
        Ok(())
    }

    pub fn rawqueue(&mut self) -> &mut rusd3d12::SCommandQueue {
        &mut self.q
    }
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn main_d3d12() {
    let debuginterface = rusd3d12::getdebuginterface().unwrap();
    debuginterface.enabledebuglayer();

    let mut winapi = rustywindows::SWinAPI::create();
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();
    let mut window = rustywindows::SWindow::create(&windowclass, "rusgam", 800, 600);

    let d3d12 = rusd3d12::initd3d12().unwrap();
    let mut adapter = d3d12.getadapter().unwrap();
    let mut device = adapter.createdevice().unwrap();

    let mut commandqueue = SCommandQueue::createcommandqueue(&winapi.rawwinapi(), &mut device).unwrap();
    let swapchain = d3d12
        .createswapchain(&window.raw(), commandqueue.rawqueue(), 800, 600)
        .unwrap();
    let mut currbuffer: u32 = swapchain.currentbackbufferindex();

    let rendertargetheap = device
        .createdescriptorheap(rusd3d12::EDescriptorHeapType::RenderTarget, 10)
        .unwrap();

    device
        .initrendertargetviews(&swapchain, &rendertargetheap)
        .unwrap();
    let commandallocators = [
        device
            .createcommandallocator(rusd3d12::ECommandListType::Direct)
            .unwrap(),
        device
            .createcommandallocator(rusd3d12::ECommandListType::Direct)
            .unwrap(),
    ];
    let mut commandlist = device
        .createcommandlist(&commandallocators[currbuffer as usize])
        .unwrap();
    commandlist.close().unwrap();

    let mut framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();

    let mut framefencevalues = [0; 2];

    window.rawmut().show();
    let mut shouldquit = false;

    while !shouldquit {
        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let dtms = dt as f64;

        //println!("Frame {} time: {}us", framecount, dtms);

        // -- render
        {
            let commandallocator = &commandallocators[currbuffer as usize];
            commandallocator.reset();
            commandlist.reset(commandallocator).unwrap();

            // -- clear the render target
            {
                // -- $$$FRK(TODO): do I want to associate these some way?
                let backbuffer = &swapchain.backbuffers[currbuffer as usize];
                let rendertargetdescriptor = rendertargetheap.cpuhandle(currbuffer);

                // -- transition to render target
                let transtorendertargetbarrier = d3d12.createtransitionbarrier(
                    backbuffer,
                    rusd3d12::EResourceStates::Present,
                    rusd3d12::EResourceStates::RenderTarget,
                );
                commandlist.pushresourcebarrier(&transtorendertargetbarrier);

                // -- clear
                let clearcolour = [0.4, 0.6, 0.9, 1.0];

                commandlist.pushclearrendertargetview(rendertargetdescriptor, &clearcolour);

                // -- transition to present
                let transtopresentbarrier = d3d12.createtransitionbarrier(
                    backbuffer,
                    rusd3d12::EResourceStates::RenderTarget,
                    rusd3d12::EResourceStates::Present,
                );
                commandlist.pushresourcebarrier(&transtopresentbarrier);

                // -- close the command list
                commandlist.close().unwrap();

                // -- execute on the queue
                commandqueue.rawqueue().executecommandlist(&mut commandlist);

                let syncinterval = 1;
                swapchain.present(syncinterval, 0).unwrap();

                framefencevalues[currbuffer as usize] = commandqueue.pushsignal().unwrap();
            }
        }

        currbuffer = swapchain.currentbackbufferindex();

        lastframetime = curframetime;
        framecount += 1;

        commandqueue.waitforfencevalue(framefencevalues[currbuffer as usize]);

        // -- $$$FRK(TODO): framerate is uncapped

        loop {
            let mut msghandler = SWindowProc { quit: false };
            let hadmessage = window.processmessage(&mut msghandler);
            shouldquit |= msghandler.shouldquit();
            if !hadmessage {
                break;
            }
        }
    }

    // -- wait for all commands to clear
    commandqueue.flushblocking().unwrap();
}

fn main() {
    main_d3d12();
}
