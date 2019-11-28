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

use std::cell::{RefCell};

#[allow(dead_code)]
type SMat44 = nalgebra::Matrix4<f32>;
type SVec3 = nalgebra::Vector3<f32>;

#[allow(dead_code)]
struct SVertexPosColour {
    position: SVec3,
    colour: SVec3,
}

#[allow(unused_variables)]
#[allow(unused_mut)]

fn main_d3d12() -> Result<(), &'static str> {
    // -- initialize debug
    let debuginterface = typeyd3d12::getdebuginterface()?;
    debuginterface.enabledebuglayer();

    // -- setup window and command queue
    let mut winapi = rustywindows::SWinAPI::create();
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();

    let mut factory = niced3d12::SFactory::create()?;
    let mut adapter = factory.create_best_adapter()?;
    let mut device = adapter.create_device()?;

    let mut commandqueue = RefCell::new(niced3d12::SCommandQueue::create(
        &mut device,
        &winapi.rawwinapi(),
        typeyd3d12::ECommandListType::Direct,
    )?);
    let mut directcommandpool = niced3d12::SCommandListPool::create(
        &device,
        &commandqueue,
        &winapi.rawwinapi(),
        1,
        2,
    )?;

    let mut copycommandqueue = RefCell::new(niced3d12::SCommandQueue::create(
        &mut device,
        &winapi.rawwinapi(),
        typeyd3d12::ECommandListType::Copy,
    )?);
    let mut copycommandpool = niced3d12::SCommandListPool::create(
        &device,
        &copycommandqueue,
        &winapi.rawwinapi(),
        1,
        2,
    )?;

    let mut window = niced3d12::created3d12window(
        &windowclass,
        &factory,
        &mut device,
        &mut commandqueue.borrow_mut(),
        "rusgam",
        800,
        600,
    )?;

    window.init_render_target_views(&mut device)?;
    window.show();

    // -- tutorial2 data
    let vertexbufferresource: Option<niced3d12::SResource> = None;
    let vertexbufferview: Option<typeyd3d12::SVertexBufferView> = None;
    let indexbufferresource: Option<niced3d12::SResource> = None;
    let indexbufferview: Option<typeyd3d12::SIndexBufferView> = None;

    let depthbufferresource: Option<niced3d12::SResource> = None;
    let depthstencilviewheap =
        device.create_descriptor_heap(typeyd3d12::EDescriptorHeapType::DepthStencil, 1);

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
        let handle = copycommandpool.alloc_list()?;
        let copycommandlist = copycommandpool.get_list(handle)?;

        let mut vertbufferresource = {
            let vertbufferflags =
                typeyd3d12::SResourceFlags::from(typeyd3d12::EResourceFlags::ENone);
            copycommandlist
                .update_buffer_resource(&device, &cubeverts, vertbufferflags)
                ?
        };
        let _vertexbufferview = vertbufferresource
            .destinationresource
            .create_vertex_buffer_view()
            ?;

        let indexbufferresource = {
            let indexbufferflags =
                typeyd3d12::SResourceFlags::from(typeyd3d12::EResourceFlags::ENone);
            copycommandlist
                .update_buffer_resource(&device, &indices, indexbufferflags)
                ?
        };
        let _indexbufferview = indexbufferresource
            .destinationresource
            .create_index_buffer_view(typeyd3d12::EFormat::R16UINT)
            ?;

        copycommandpool.execute_and_free_list(handle)?;
    }

    // -- load shaders
    let vertblob = typeyd3d12::read_file_to_blob("shaders_built/vertex.cso")?;
    let pixelblob = typeyd3d12::read_file_to_blob("shaders_built/pixel.cso")?;

    // -- input assembler stuff
    let input_element_desc = [
        typeyd3d12::SInputElementDesc::create(
            "POSITION",
            0,
            typeyd3d12::EDXGIFormat::R32G32B32Float,
            0,
            winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
            typeyd3d12::EInputClassification::PerVertexData,
            0,
        ),
        typeyd3d12::SInputElementDesc::create(
            "COLOR",
            0,
            typeyd3d12::EDXGIFormat::R32G32B32Float,
            0,
            winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
            typeyd3d12::EInputClassification::PerVertexData,
            0,
        ),
    ];

    let root_signature_flags = typeyd3d12::SRootSignatureFlags::create(&[
        typeyd3d12::ERootSignatureFlags::AllowInputAssemblerInputLayout,
        typeyd3d12::ERootSignatureFlags::DenyHullShaderRootAccess,
        typeyd3d12::ERootSignatureFlags::DenyDomainShaderRootAccess,
        typeyd3d12::ERootSignatureFlags::DenyGeometryShaderRootAccess,
        typeyd3d12::ERootSignatureFlags::DenyPixelShaderRootAccess,
    ]);

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
        commandqueue.borrow().wait_for_internal_fence_value(framefencevalues[window.currentbackbufferindex()]);

        // -- render
        {
            let backbufferidx = window.currentbackbufferindex();
            assert!(backbufferidx == window.swapchain.current_backbuffer_index());

            let handle = directcommandpool.alloc_list()?;

            // -- clear the render target
            {
                let list = directcommandpool.get_list(handle)?;

                let backbuffer = window.currentbackbuffer();

                // -- transition to render target
                list.transition_resource(
                    backbuffer,
                    typeyd3d12::EResourceStates::Present,
                    typeyd3d12::EResourceStates::RenderTarget,
                )?;

                // -- clear
                let clearcolour = [0.4, 0.6, 0.9, 1.0];
                list.clear_render_target_view(
                    window.currentrendertargetdescriptor()?,
                    &clearcolour,
                )?;

                // -- transition to present
                list.transition_resource(
                    backbuffer,
                    typeyd3d12::EResourceStates::RenderTarget,
                    typeyd3d12::EResourceStates::Present,
                )?;
            }

            // -- execute on the queue
            assert_eq!(window.currentbackbufferindex(), backbufferidx);
            directcommandpool.execute_and_free_list(handle)?;
            framefencevalues[window.currentbackbufferindex()] = commandqueue.borrow_mut().signal_internal_fence()?;

            // -- present the swap chain and switch to next buffer in swap chain
            window.present()?;
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
                        let rect: safewindows::SRect = window.raw().getclientrect()?;
                        let newwidth = rect.right - rect.left;
                        let newheight = rect.bottom - rect.top;

                        window
                            .resize(
                                newwidth as u32,
                                newheight as u32,
                                &mut commandqueue.borrow_mut(),
                                &mut device,
                            )
                            ?;

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
    commandqueue.borrow_mut().flush_blocking()?;

    Ok(())
}

fn main() {
    main_d3d12().unwrap();
}
