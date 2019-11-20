extern crate nalgebra;
extern crate winapi;
extern crate wio;

//mod math;
mod collections;
mod directxgraphicssamples;
mod niced3d12;
mod rustywindows;
mod safewindows;
mod typeyd3d12;

type SMat44 = nalgebra::Matrix4<f32>;
type SVec3 = nalgebra::Vector3<f32>;

#[allow(dead_code)]
struct SVertexPosColour {
    position: SVec3,
    colour: SVec3,
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn main_d3d12() {
    // -- initialize debug
    let debuginterface = typeyd3d12::getdebuginterface().unwrap();
    debuginterface.enabledebuglayer();

    // -- setup window and command queue
    let mut winapi = rustywindows::SWinAPI::create();
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();

    let mut d3dctxt = niced3d12::SD3D12Context::create().unwrap();
    let mut adapter = d3dctxt.create_best_adapter().unwrap();
    let mut device = adapter.create_device(&d3dctxt).unwrap();

    /*
    let mut commandqueue = niced3d12::SCommandQueue::create_command_queue(
        &d3dctxt,
        &winapi.rawwinapi(),
        &mut device,
        typeyd3d12::ECommandListType::Direct,
    )
    .unwrap();
    commandqueue.setup(&device, 2, 1).unwrap();

    let mut copycommandqueue = niced3d12::SCommandQueue::createcommandqueue(
        &winapi.rawwinapi(),
        &device,
        typeyd3d12::ECommandListType::Copy,
    )
    .unwrap();
    copycommandqueue.setup(&device, 2, 1).unwrap();

    let mut window = niced3d12::createsd3d12window(
        &mut d3dctxt,
        &windowclass,
        &device,
        &mut commandqueue,
        "rusgam",
        800,
        600,
    )
    .unwrap();
    window.initrendertargetviews(&device).unwrap();
    window.show();

    // -- tutorial2 data
    let vertexbufferresource: Option<typeyd3d12::SResource> = None;
    let vertexbufferview: Option<typeyd3d12::SVertexBufferView> = None;
    let indexbufferresource: Option<typeyd3d12::SResource> = None;
    let indexbufferview: Option<typeyd3d12::SIndexBufferView> = None;

    let depthbufferresource: Option<typeyd3d12::SResource> = None;
    let depthstencilviewheap =
        device.createdescriptorheap(typeyd3d12::EDescriptorHeapType::DepthStencil, 1);

    let rootsignature: Option<typeyd3d12::SRootSignature> = None;
    let pipelinestate: Option<typeyd3d12::SPipelineState> = None;
    let viewport = typeyd3d12::SViewport::new(
        0.0,
        0.0,
        window.width() as f32,
        window.height() as f32,
        None,
        None,
    );
    let scissorrect = typeyd3d12::SRect {
        left: 0,
        right: std::i32::MAX,
        top: 0,
        bottom: std::i32::MAX,
    };

    let fov: f32 = 45.0;
    let modelmatrix = SMat44::identity();
    let viewmatrix = SMat44::identity();
    let projectionmatrix = SMat44::identity();

    let contentloaded = false;

    let cubeverts = [
        SVertexPosColour {
            position: SVec3::new(-1.0, -1.0, -1.0),
            colour: SVec3::new(0.0, 0.0, 0.0),
        },
        SVertexPosColour {
            position: SVec3::new(-1.0, 1.0, -1.0),
            colour: SVec3::new(0.0, 1.0, 0.0),
        },
        SVertexPosColour {
            position: SVec3::new(1.0, 1.0, -1.0),
            colour: SVec3::new(1.0, 1.0, 0.0),
        },
        SVertexPosColour {
            position: SVec3::new(1.0, -1.0, -1.0),
            colour: SVec3::new(1.0, 0.0, 0.0),
        },
        SVertexPosColour {
            position: SVec3::new(-1.0, -1.0, 1.0),
            colour: SVec3::new(0.0, 0.0, 1.0),
        },
        SVertexPosColour {
            position: SVec3::new(-1.0, 1.0, 1.0),
            colour: SVec3::new(0.0, 1.0, 1.0),
        },
        SVertexPosColour {
            position: SVec3::new(1.0, 1.0, 1.0),
            colour: SVec3::new(1.0, 1.0, 1.0),
        },
        SVertexPosColour {
            position: SVec3::new(1.0, -1.0, 1.0),
            colour: SVec3::new(1.0, 0.0, 1.0),
        },
    ];

    #[rustfmt::skip]
    let indices : [u16; 36] = [
        0, 1, 2,
        0, 2, 3,
        4, 6, 5,
        4, 7, 6,
        4, 5, 1,
        4, 1, 0,
        3, 2, 6,
        3, 6, 7,
        1, 5, 6,
        1, 6, 2,
        4, 0, 3,
        4, 3, 7
    ];

    // -- upload data to GPU
    {
        let copycommandlisthandle = copycommandqueue.getunusedcommandlisthandle().unwrap();
        let copycommandlist = copycommandqueue
            .getcommandlist(copycommandlisthandle)
            .unwrap();

        let mut vertbufferresource = {
            let vertbufferflags =
                typeyd3d12::SResourceFlags::from(typeyd3d12::EResourceFlags::ENone);
            copycommandlist
                .updatebufferresource(&mut device, &cubeverts, vertbufferflags)
                .unwrap()
        };
        let _vertexbufferview = vertbufferresource
            .destination
            .createvertexbufferview()
            .unwrap();

        let indexbufferresource = {
            let indexbufferflags =
                typeyd3d12::SResourceFlags::from(typeyd3d12::EResourceFlags::ENone);
            copycommandlist
                .updatebufferresource(&mut device, &indices, indexbufferflags)
                .unwrap()
        };
        let _indexbufferview = indexbufferresource
            .destination
            .createindexbufferview(typeyd3d12::EFormat::R16UINT)
            .unwrap();

        copycommandqueue
            .executecommandlist(copycommandlisthandle)
            .unwrap();
    }

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
                commandqueue
                    .transitionresource(
                        commandlisthandle,
                        backbuffer,
                        typeyd3d12::EResourceStates::Present,
                        typeyd3d12::EResourceStates::RenderTarget,
                    )
                    .unwrap();

                // -- clear
                let clearcolour = [0.4, 0.6, 0.9, 1.0];
                commandqueue
                    .clearrendertargetview(
                        commandlisthandle,
                        window.currentrendertargetdescriptor().unwrap(),
                        &clearcolour,
                    )
                    .unwrap();

                // -- transition to present
                commandqueue
                    .transitionresource(
                        commandlisthandle,
                        backbuffer,
                        typeyd3d12::EResourceStates::RenderTarget,
                        typeyd3d12::EResourceStates::Present,
                    )
                    .unwrap();
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

                        window
                            .resize(
                                newwidth as u32,
                                newheight as u32,
                                &mut commandqueue,
                                &device,
                            )
                            .unwrap();

                        // -- $$$FRK(TODO): why do we do this?
                        let maxframefencevalue =
                            std::cmp::max(framefencevalues[0], framefencevalues[1]);
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
    */
}

fn main() {
    main_d3d12();
}
