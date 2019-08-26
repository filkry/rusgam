extern crate winapi;
extern crate wio;

//mod math;
mod collections;
mod rusd3d12;

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

impl rusd3d12::TWindowProc for SWindowProc {
    fn windowproc(&mut self, window: &mut rusd3d12::SWindow, msg: rusd3d12::EMsgType) -> () {
        match msg {
            rusd3d12::EMsgType::Paint => {
                window.dummyrepaint();
            }
            rusd3d12::EMsgType::KeyDown { key } => match key {
                rusd3d12::EKey::Q => {
                    self.quit = true;
                    println!("Q keydown");
                }
                _ => (),
            },
            rusd3d12::EMsgType::Size {
                width: _,
                height: _,
            } => {
                println!("Size");
            }
            rusd3d12::EMsgType::Invalid => (),
        }
    }
}

struct SCommandQueue {
    q: rusd3d12::SCommandQueue,
    fence: rusd3d12::SFence,
    fenceevent: rusd3d12::SEventHandle,
    nextfencevalue: u64,
}

impl SCommandQueue {
    pub fn createcommandqueue(
        winapi: &rusd3d12::SWinAPI,
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

    let winapi = rusd3d12::initwinapi().unwrap();
    let windowclass = winapi.registerclassex("rusgam").unwrap();

    let mut window = rusd3d12::SWindow::create();

    windowclass
        .createwindow(&mut window, "rusgame2", 800, 600)
        .unwrap();

    let d3d12 = rusd3d12::initd3d12().unwrap();
    let mut adapter = d3d12.getadapter().unwrap();
    let mut device = adapter.createdevice().unwrap();

    let mut commandqueue = SCommandQueue::createcommandqueue(&winapi, &mut device).unwrap();
    let swapchain = d3d12
        .createswapchain(&window, commandqueue.rawqueue(), 800, 600)
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

    window.show();
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
            let hadmessage = window.peekmessage(&mut msghandler);
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
