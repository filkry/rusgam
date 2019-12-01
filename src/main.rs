extern crate nalgebra;
extern crate winapi;
extern crate wio;
extern crate arrayvec;

//mod math;
mod collections;
mod directxgraphicssamples;
mod niced3d12;
mod rustywindows;
mod safewindows;
mod typeyd3d12;

// -- std includes
use std::mem::{size_of};
use std::cell::{RefCell};

// -- crate includes
use arrayvec::{ArrayVec};

use typeyd3d12 as t12;
use niced3d12 as n12;

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

#[repr(C)]
struct SPipelineStateStream<'a> {
    root_signature: n12::SPipelineStateStreamRootSignature<'a>,
    input_layout: n12::SPipelineStateStreamInputLayout<'a>,
    primitive_topology: n12::SPipelineStateStreamPrimitiveTopology,
    vertex_shader: n12::SPipelineStateStreamVertexShader<'a>,
    pixel_shader: n12::SPipelineStateStreamPixelShader<'a>,
    depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat,
    rtv_formats: n12::SPipelineStateStreamRTVFormats<'a>,
}

fn main_d3d12() -> Result<(), &'static str> {
    // -- initialize debug
    let debuginterface = t12::getdebuginterface()?;
    debuginterface.enabledebuglayer();

    // -- setup window and command queue
    let winapi = rustywindows::SWinAPI::create();
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();

    let mut factory = n12::SFactory::create()?;
    let mut adapter = factory.create_best_adapter()?;
    let mut device = adapter.create_device()?;

    let commandqueue = RefCell::new(n12::SCommandQueue::create(
        &mut device,
        &winapi.rawwinapi(),
        t12::ECommandListType::Direct,
    )?);
    let mut directcommandpool = n12::SCommandListPool::create(
        &device,
        &commandqueue,
        &winapi.rawwinapi(),
        1,
        2,
    )?;

    let copycommandqueue = RefCell::new(n12::SCommandQueue::create(
        &mut device,
        &winapi.rawwinapi(),
        t12::ECommandListType::Copy,
    )?);
    let mut copycommandpool = n12::SCommandListPool::create(
        &device,
        &copycommandqueue,
        &winapi.rawwinapi(),
        1,
        2,
    )?;

    let mut window = n12::created3d12window(
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
    let _depthbufferresource: Option<n12::SResource> = None;
    let _depthstencilviewheap =
        device.create_descriptor_heap(t12::EDescriptorHeapType::DepthStencil, 1);

    let _viewport = t12::SViewport::new(
        0.0,
        0.0,
        window.width() as f32,
        window.height() as f32,
        None,
        None,
    );
    let _scissorrect = t12::SRect {
        left: 0,
        right: std::i32::MAX,
        top: 0,
        bottom: std::i32::MAX,
    };

    let _fov: f32 = 45.0;
    let _modelmatrix = SMat44::identity();
    let _viewmatrix = SMat44::identity();
    let _projectionmatrix = SMat44::identity();

    let _contentloaded = false;

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

        let vertbufferresource = {
            let vertbufferflags =
                t12::SResourceFlags::from(t12::EResourceFlags::ENone);
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
                t12::SResourceFlags::from(t12::EResourceFlags::ENone);
            copycommandlist
                .update_buffer_resource(&device, &indices, indexbufferflags)
                ?
        };
        let _indexbufferview = indexbufferresource
            .destinationresource
            .create_index_buffer_view(t12::EFormat::R16UINT)
            ?;

        let fenceval = copycommandpool.execute_and_free_list(handle)?;
        copycommandpool.wait_for_internal_fence_value(fenceval);
    }

    // -- load shaders
    let vertblob = t12::read_file_to_blob("shaders_built/vertex.cso")?;
    let pixelblob = t12::read_file_to_blob("shaders_built/pixel.cso")?;

    let vert_byte_code = t12::SShaderBytecode::create(&vertblob);
    let pixel_byte_code = t12::SShaderBytecode::create(&pixelblob);

    // -- root signature stuff
    let mut input_layout_desc = {
        let input_element_desc = [
            t12::SInputElementDesc::create(
                "POSITION",
                0,
                t12::EDXGIFormat::R32G32B32Float,
                0,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
            t12::SInputElementDesc::create(
                "COLOR",
                0,
                t12::EDXGIFormat::R32G32B32Float,
                0,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
        ];

        t12::SInputLayoutDesc::create(&input_element_desc)
    };

    let root_parameter = t12::SRootParameter {
        type_: t12::ERootParameterType::E32BitConstants,
        type_data: t12::ERootParameterTypeData::Constants {
            constants: t12::SRootConstants {
                shader_register: 0,
                register_space: 0,
                num_32_bit_values: (size_of::<SMat44>() / 4) as u32,
            },
        },
        shader_visibility: t12::EShaderVisibility::Vertex,
    };

    let root_signature_flags = t12::SRootSignatureFlags::create(&[
        t12::ERootSignatureFlags::AllowInputAssemblerInputLayout,
        t12::ERootSignatureFlags::DenyHullShaderRootAccess,
        t12::ERootSignatureFlags::DenyDomainShaderRootAccess,
        t12::ERootSignatureFlags::DenyGeometryShaderRootAccess,
        t12::ERootSignatureFlags::DenyPixelShaderRootAccess,
    ]);

    let mut root_signature_desc = t12::SRootSignatureDesc::new(root_signature_flags);
    root_signature_desc.parameters.push(root_parameter);

    let serialized_root_signature = t12::serialize_root_signature(
        &mut root_signature_desc,
        t12::ERootSignatureVersion::V1,
    ).ok().expect("Could not serialize root signature.");

    let root_signature = device.raw().create_root_signature(&serialized_root_signature)?;

    let mut rtv_formats = t12::SRTFormatArray {
        rt_formats: ArrayVec::new(),
    };
    rtv_formats.rt_formats.push(t12::EDXGIFormat::R8G8B8A8UNorm);

    // -- pipeline state object
    let pipeline_state_stream = SPipelineStateStream {
        root_signature: n12::SPipelineStateStreamRootSignature::create(&root_signature),
        input_layout: n12::SPipelineStateStreamInputLayout::create(&mut input_layout_desc),
        primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(t12::EPrimitiveTopologyType::Triangle),
        vertex_shader: n12::SPipelineStateStreamVertexShader::create(&vert_byte_code),
        pixel_shader: n12::SPipelineStateStreamPixelShader::create(&pixel_byte_code),
        depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(t12::EDXGIFormat::D32Float),
        rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
    };
    let pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&pipeline_state_stream);
    let _pipelinestate = device.raw().create_pipeline_state(&pipeline_state_stream_desc);

    // -- update loop

    let mut _framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();

    let mut framefencevalues = [0; 2];

    let mut shouldquit = false;

    while !shouldquit {
        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let _dtms = dt as f64;

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
                    t12::EResourceStates::Present,
                    t12::EResourceStates::RenderTarget,
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
                    t12::EResourceStates::RenderTarget,
                    t12::EResourceStates::Present,
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
        _framecount += 1;

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
