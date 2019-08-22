extern crate winapi;
extern crate wio;

//mod math;
mod rusd3d12;
mod collections;

#[allow(unused_variables)]
#[allow(unused_mut)]
fn main_d3d12() {
    let debuginterface = rusd3d12::getdebuginterface().unwrap();
    debuginterface.enabledebuglayer();

    let winapi = rusd3d12::initwinapi().unwrap();
    let windowclass = winapi.registerclassex("rusgam").unwrap();

    let mut window = rusd3d12::SWindow::create(1024);
    window.allocqueue();

    windowclass.createwindow(&mut window, "rusgame2", 800, 600).unwrap();

    let d3d12 = rusd3d12::initd3d12().unwrap();
    let mut adapter = d3d12.getadapter().unwrap();
    let device = adapter.createdevice().unwrap();

    let mut commandqueue = device.createcommandqueue(
        rusd3d12::ECommandListType::Direct).unwrap();
    let swapchain = d3d12.createswapchain(&window, &mut commandqueue, 800, 600).unwrap();
    let mut currbuffer: u32 = swapchain.currentbackbufferindex();

    let rendertargetheap = device.createdescriptorheap(
        rusd3d12::EDescriptorHeapType::RenderTarget,
        10).unwrap();

    device.initrendertargetviews(&swapchain, &rendertargetheap).unwrap();
    let commandallocators = [
        device.createcommandallocator(rusd3d12::ECommandListType::Direct).unwrap(),
        device.createcommandallocator(rusd3d12::ECommandListType::Direct).unwrap()
    ];
    let mut commandlist = device.createcommandlist(&commandallocators[currbuffer as usize]).unwrap();
    commandlist.close().unwrap();

    let fence = device.createfence().unwrap();
    let fenceevent = winapi.createeventhandle().unwrap();

    let mut framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();

    let mut framefencevalues = [0; 2];
    let mut nextfencevalue = 0;

    window.show();

    let mut quit: bool = false;
    while !quit {
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

                commandlist.pushclearrendertargetview(
                    rendertargetdescriptor,
                    &clearcolour,
                );

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
                commandqueue.executecommandlist(&mut commandlist);

                let syncinterval = 1;
                swapchain.present(syncinterval, 0).unwrap();

                framefencevalues[currbuffer as usize] =
                    commandqueue.pushsignal(&fence, &mut nextfencevalue).unwrap();
            }
        }

        currbuffer = swapchain.currentbackbufferindex();

        lastframetime = curframetime;
        framecount += 1;

        fence.waitforvalue(framefencevalues[currbuffer as usize], &fenceevent, <u64>::max_value()).unwrap();

        // -- $$$FRK(TODO): framerate is uncapped

        loop {
            match winapi.peekmessage(&window) {
                Some(msg) => {
                    match msg.msgtype() {
                        rusd3d12::EMsgType::Paint => {
                            window.dummyrepaint();
                        },
                        rusd3d12::EMsgType::KeyDown{key} => {
                            match key {
                                rusd3d12::EKey::Q => {
                                    quit = true;
                                    println!("Q keydown");
                                },
                                _ => ()
                            }
                        },
                        rusd3d12::EMsgType::Size{width, height} => {
                            println!("Size");
                        },
                        rusd3d12::EMsgType::Invalid => (),
                    }
                }
                None => break
            }
        }
    }

    // -- wait for all commands to clear
    fence.waitforvalue(framefencevalues[0], &fenceevent, <u64>::max_value()).unwrap();
    fence.waitforvalue(framefencevalues[1], &fenceevent, <u64>::max_value()).unwrap();

}

fn main() {
    main_d3d12();
}
