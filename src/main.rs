extern crate winapi;
extern crate wio;

//mod math;
mod collections;
mod rustyd3d12;
mod rustywindows;
mod safed3d12;
mod safewindows;

#[allow(unused_variables)]
#[allow(unused_mut)]
fn main_d3d12() {

    // -- initialize debug
    let debuginterface = safed3d12::getdebuginterface().unwrap();
    debuginterface.enabledebuglayer();


    // -- setup all data
    let mut curwidth = 800;
    let mut curheight = 600;

    let mut winapi = rustywindows::SWinAPI::create();
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();

    let mut factory = rustyd3d12::SFactory::create().unwrap();
    let mut adapter = factory.bestadapter().unwrap();
    let mut device = adapter.createdevice().unwrap();

    let mut commandqueue = rustyd3d12::SCommandQueue::createcommandqueue(
        &winapi.rawwinapi(),
        &device,
        safed3d12::ECommandListType::Direct,
    ).unwrap();
    commandqueue.setup(&device, 2, 1).unwrap();

    let mut window = rustyd3d12::createsd3d12window(
        &mut factory,
        &windowclass,
        &device,
        &mut commandqueue,
        "rusgam",
        curwidth,
        curheight
    ).unwrap();
    window.initrendertargetviews(&device).unwrap();
    window.show();


    // -- update loop

    let mut framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();

    let mut framefencevalues = [0; 2];

    let mut shouldquit = false;

    while !shouldquit {
        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let dtms = dt as f64;

        //println!("Frame {} time: {}us", framecount, dtms);

        // -- wait for buffer to be available
        commandqueue.waitforfencevalue(framefencevalues[window.currentbackbufferindex()]);

        // -- render
        {
            let backbufferidx = window.currentbackbufferindex();
            assert!(backbufferidx == window.swapchain.raw().currentbackbufferindex());

            let commandlisthandle = commandqueue.getunusedcommandlisthandle().unwrap();

            // -- clear the render target
            {
                let backbuffer = window.currentbackbuffer();

                // -- transition to render target
                commandqueue.transitionresource(
                    commandlisthandle, backbuffer,
                    safed3d12::EResourceStates::Present,
                    safed3d12::EResourceStates::RenderTarget
                ).unwrap();

                // -- clear
                let clearcolour = [0.4, 0.6, 0.9, 1.0];
                commandqueue.clearrendertargetview(
                    commandlisthandle,
                    window.currentrendertargetdescriptor().unwrap(),
                    &clearcolour,
                ).unwrap();

                // -- transition to present
                commandqueue.transitionresource(
                    commandlisthandle, backbuffer,
                    safed3d12::EResourceStates::RenderTarget,
                    safed3d12::EResourceStates::Present
                ).unwrap();
            }

            // -- execute on the queue
            assert_eq!(window.currentbackbufferindex(), backbufferidx);
            commandqueue.executecommandlist(commandlisthandle).unwrap();
            framefencevalues[window.currentbackbufferindex()] = commandqueue.pushsignal().unwrap();

            // -- present the swap chain and switch to next buffer in swap chain
            window.present().unwrap();
        }

        lastframetime = curframetime;
        framecount += 1;

        // -- $$$FRK(TODO): framerate is uncapped

        loop {
            let msg = window.pollmessage();
            match msg {
                None => break,
                Some(m) => match m {
                    safewindows::EMsgType::Paint => {
                        //println!("Paint!");
                        window.dummyrepaint();
                    }
                    safewindows::EMsgType::KeyDown { key } => match key {
                        safewindows::EKey::Q => {
                            shouldquit = true;
                            //println!("Q keydown");
                        }
                        _ => (),
                    },
                    safewindows::EMsgType::Size => {
                        //println!("Size");
                        let rect: safewindows::SRect = window.raw().getclientrect().unwrap();
                        let newwidth = rect.right - rect.left;
                        let newheight = rect.bottom - rect.top;

                        window.resize(
                            newwidth as u32,
                            newheight as u32,
                            &mut commandqueue,
                            &device,
                        ).unwrap();

                        // -- $$$FRK(TODO): why do we do this?
                        let maxframefencevalue = std::cmp::max(framefencevalues[0], framefencevalues[1]);
                        framefencevalues[0] = maxframefencevalue;
                        framefencevalues[1] = maxframefencevalue;
                    }
                    safewindows::EMsgType::Invalid => (),
                },
            }
        }

        // -- increase frame time for testing
        //std::thread::sleep(std::time::Duration::from_millis(111));
    }

    // -- wait for all commands to clear
    commandqueue.flushblocking().unwrap();
}

fn main() {
    main_d3d12();
}
