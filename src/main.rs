extern crate winapi;
extern crate wio;

//mod math;
mod collections;
mod rustyd3d12;
mod rustywindows;
mod safed3d12;
mod safewindows;

fn resize(
    width: u32,
    height: u32,
    curwidth: &mut u32,
    curheight: &mut u32,
    commandqueue: &mut rustyd3d12::SCommandQueue,
    swapchain: &mut rustyd3d12::SSwapChain,
    device: &rustyd3d12::SDevice,
    rendertargetheap: &rustyd3d12::SDescriptorHeap,
    framefencevalues: &mut [u64],
    currbuffer: &mut u32,
) -> Result<(), &'static str> {
    if *curwidth != width || *curheight != height {
        let newwidth = std::cmp::max(1, width);
        let newheight = std::cmp::max(1, height);
        commandqueue.flushblocking()?;

        swapchain.backbuffers.clear();
        framefencevalues[0] = framefencevalues[*currbuffer as usize];
        framefencevalues[1] = framefencevalues[*currbuffer as usize];

        let desc = swapchain.raw().getdesc()?;
        swapchain
            .raw()
            .resizebuffers(2, newwidth, newheight, &desc)?;

        *currbuffer = swapchain.raw().currentbackbufferindex();
        device.initrendertargetviews(swapchain, rendertargetheap)?;

        *curwidth = newwidth;
        *curheight = newheight;
    }

    Ok(())
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn main_d3d12() {
    let debuginterface = safed3d12::getdebuginterface().unwrap();
    debuginterface.enabledebuglayer();

    let mut curwidth = 800;
    let mut curheight = 600;

    let mut winapi = rustywindows::SWinAPI::create();
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();
    let mut window =
        rustywindows::SWindow::create(&windowclass, "rusgam", curwidth, curheight).unwrap();

    let d3d12 = rustyd3d12::SFactory::create().unwrap();
    let mut adapter = d3d12.bestadapter().unwrap();
    let mut device = adapter.createdevice().unwrap();

    let mut commandqueue =
        rustyd3d12::SCommandQueue::createcommandqueue(&winapi.rawwinapi(), &device, safed3d12::ECommandListType::Direct).unwrap();
    commandqueue.setup(&device, 2, 1).unwrap();

    let mut swapchain = d3d12
        .createswapchain(&window.raw(), commandqueue.rawqueue(), curwidth, curheight)
        .unwrap();
    let mut currbuffer: u32 = swapchain.raw().currentbackbufferindex();

    let rendertargetheap = device
        .createdescriptorheap(safed3d12::EDescriptorHeapType::RenderTarget, 10)
        .unwrap();

    device
        .initrendertargetviews(&mut swapchain, &rendertargetheap)
        .unwrap();

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
            let commandlisthandle = commandqueue.getunusedcommandlisthandle().unwrap();

            // -- clear the render target
            {
                // -- $$$FRK(TODO): do I want to associate these some way?
                let backbuffer = &swapchain.backbuffers[currbuffer as usize];

                // -- transition to render target
                commandqueue.transitionresource(
                    commandlisthandle, backbuffer,
                    safed3d12::EResourceStates::Present,
                    safed3d12::EResourceStates::RenderTarget
                ).unwrap();

                // -- clear
                let clearcolour = [0.4, 0.6, 0.9, 1.0];
                let rendertargetdescriptor = rendertargetheap.cpuhandle(currbuffer).unwrap();
                commandqueue.clearrendertargetview(commandlisthandle, rendertargetdescriptor, &clearcolour).unwrap();

                // -- transition to present
                commandqueue.transitionresource(
                    commandlisthandle, backbuffer,
                    safed3d12::EResourceStates::RenderTarget,
                    safed3d12::EResourceStates::Present
                ).unwrap();
            }

            // -- execute on the queue
            commandqueue.executecommandlist(commandlisthandle).unwrap();

            let syncinterval = 1;
            swapchain.raw().present(syncinterval, 0).unwrap();

            framefencevalues[currbuffer as usize] = commandqueue.pushsignal().unwrap();
        }

        currbuffer = swapchain.raw().currentbackbufferindex();

        lastframetime = curframetime;
        framecount += 1;

        commandqueue.waitforfencevalue(framefencevalues[currbuffer as usize]);

        // -- $$$FRK(TODO): framerate is uncapped

        loop {
            let msg = window.pollmessage();
            match msg {
                None => break,
                Some(m) => match m {
                    safewindows::EMsgType::Paint => {
                        println!("Paint!");
                        window.dummyrepaint();
                    }
                    safewindows::EMsgType::KeyDown { key } => match key {
                        safewindows::EKey::Q => {
                            shouldquit = true;
                            println!("Q keydown");
                        }
                        _ => (),
                    },
                    safewindows::EMsgType::Size => {
                        println!("Size");
                        let rect: safewindows::SRect = window.raw().getclientrect().unwrap();
                        let newwidth = rect.right - rect.left;
                        let newheight = rect.bottom - rect.top;

                        resize(
                            newwidth as u32,
                            newheight as u32,
                            &mut curwidth,
                            &mut curheight,
                            &mut commandqueue,
                            &mut swapchain,
                            &device,
                            &rendertargetheap,
                            &mut framefencevalues[..],
                            &mut currbuffer,
                        )
                        .unwrap();
                    }
                    safewindows::EMsgType::Invalid => (),
                },
            }
        }
    }

    // -- wait for all commands to clear
    commandqueue.flushblocking().unwrap();
}

fn main() {
    main_d3d12();
}
