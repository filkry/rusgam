extern crate nalgebra;
extern crate nalgebra_glm as glm;
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
type SPnt3 = nalgebra::Point3<f32>;
type SVec3 = nalgebra::Vector3<f32>;
type SVec4 = nalgebra::Vector4<f32>;

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

pub fn init_depth_texture(
    width: u32,
    height: u32,
    device: &n12::SDevice,
    direct_command_pool: &mut n12::SCommandListPool,
    copy_command_pool: &mut n12::SCommandListPool,
    depth_descriptor_heap: &n12::SDescriptorHeap,
) -> Result<n12::SResource, &'static str> {
    direct_command_pool.flush_blocking().unwrap();
    copy_command_pool.flush_blocking().unwrap();

    let clear_value = t12::SClearValue{
        format: t12::EDXGIFormat::D32Float,
        value: t12::EClearValue::DepthStencil(t12::SDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        }),
    };

    // -- need to not let this be destroyed
    let mut depth_texture_resource = device.create_committed_texture2d_resource(
        t12::EHeapType::Default,
        width,
        height,
        1,
        0,
        t12::EDXGIFormat::D32Float,
        clear_value,
        t12::SResourceFlags::from(t12::EResourceFlags::AllowDepthStencil),
        t12::EResourceStates::DepthWrite,
    )?;

    let depth_stencil_view_desc = t12::SDepthStencilViewDesc {
        format: t12::EDXGIFormat::D32Float,
        view_dimension: t12::EDSVDimension::Texture2D,
        flags: t12::SDSVFlags::from(t12::EDSVFlags::None),
        data: t12::EDepthStencilViewDescData::Tex2D(t12::STex2DDSV {
            mip_slice: 0,
        }),
    };

    device.create_depth_stencil_view(
        &mut depth_texture_resource,
        &depth_stencil_view_desc,
        depth_descriptor_heap.cpu_handle_heap_start()
    )?;

    Ok(depth_texture_resource)
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
    let depthstencilviewheap =
        device.create_descriptor_heap(t12::EDescriptorHeapType::DepthStencil, 1)?;

    let mut viewport = t12::SViewport::new(
        0.0,
        0.0,
        window.width() as f32,
        window.height() as f32,
        None,
        None,
    );
    let scissorrect = t12::SRect {
        left: 0,
        right: std::i32::MAX,
        top: 0,
        bottom: std::i32::MAX,
    };

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
    let (vert_buffer_resource, vert_buffer_view, index_buffer_resource, index_buffer_view) = {
        let handle = copycommandpool.alloc_list()?;
        let copycommandlist = copycommandpool.get_list(handle)?;

        let vertbufferresource = {
            let vertbufferflags =
                t12::SResourceFlags::from(t12::EResourceFlags::ENone);
            copycommandlist
                .update_buffer_resource(&device, &cubeverts, vertbufferflags)
                ?
        };
        let vertexbufferview = vertbufferresource
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
        let indexbufferview = indexbufferresource
            .destinationresource
            .create_index_buffer_view(t12::EFormat::R16UINT)
            ?;

        let fenceval = copycommandpool.execute_and_free_list(handle)?;
        copycommandpool.wait_for_internal_fence_value(fenceval);

        (vertbufferresource, vertexbufferview, indexbufferresource, indexbufferview)
    };

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
    let pipeline_state = device.raw().create_pipeline_state(&pipeline_state_stream_desc)?;

    // -- depth texture
    #[allow(unused_variables)]
    let mut depth_texture_resource = init_depth_texture(
        window.width(),
        window.height(),
        &device,
        &mut directcommandpool,
        &mut copycommandpool,
        &depthstencilviewheap
    );

    // -- update loop

    let mut _framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();

    let mut framefencevalues = [0; 2];

    let mut shouldquit = false;

    let start_time = winapi.curtimemicroseconds();
    let rot_axis = SVec3::new(0.0, 1.0, 0.0);

    let view_matrix = {
        let eye_position = SPnt3::new(0.0, 0.0, -10.0);
        let target_position = SPnt3::new(0.0, 0.0, 0.0);
        let up_direction = SVec3::y();

        SMat44::look_at_lh(&eye_position, &target_position, &up_direction)
    };

    while !shouldquit {
        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let _dtms = dt as f64;

        let total_time = curframetime - start_time;

        // -- update
        let cur_angle = ((total_time as f32) / 1_000_000.0) * (3.14159 / 4.0);
        let model_matrix = SMat44::new_rotation(rot_axis * cur_angle);

        let perspective_matrix : SMat44 = {
            let aspect = (window.width() as f32) / (window.height() as f32);
            let fovy : f32 = 3.14159 / 4.0;
            let znear = 0.1;
            let zfar = 100.0;

            //SMat44::new_perspective(aspect, fovy, znear, zfar)
            glm::perspective_lh(aspect, fovy, znear, zfar)
        };

        //println!("View: {}", view_matrix);
        //println!("Perspective: {}", perspective_matrix);

        //println!("Frame time: {}us", _dtms);

        // -- wait for buffer to be available
        commandqueue.borrow().wait_for_internal_fence_value(framefencevalues[window.currentbackbufferindex()]);

        // -- render
        {
            let backbufferidx = window.currentbackbufferindex();
            assert!(backbufferidx == window.swapchain.current_backbuffer_index());

            let handle = directcommandpool.alloc_list()?;

            {
                let list = directcommandpool.get_list(handle)?;

                let backbuffer = window.currentbackbuffer();
                let render_target_view = window.currentrendertargetdescriptor()?;
                let depth_texture_view = depthstencilviewheap.cpu_handle_heap_start();

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
                list.clear_depth_stencil_view(
                    depthstencilviewheap.cpu_handle_heap_start(),
                    1.0,
                )?;

                // -- set up pipeline
                list.set_pipeline_state(&pipeline_state);
                // root signature has to be set explicitly despite being on PSO, according to tutorial
                list.set_graphics_root_signature(&root_signature);

                // -- setup input assembler
                list.ia_set_primitive_topology(t12::EPrimitiveTopology::TriangleList);
                list.ia_set_vertex_buffers(0, &[&vert_buffer_view]);
                list.ia_set_index_buffer(&index_buffer_view);

                // -- setup rasterizer state
                list.rs_set_viewports(&[&viewport]);
                list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

                // -- setup the output merger
                list.om_set_render_targets(&[&render_target_view], false, &depth_texture_view);

                // -- update root parameters
                let mvp = perspective_matrix * view_matrix * model_matrix;
                list.set_graphics_root_32_bit_constants(0, &mvp, 0);

                /*
                let test_vert = SPnt3::new(1.0, 0.0, 0.0);
                let test_vert_xformed = perspective_matrix * view_matrix * model_matrix * test_vert.to_homogeneous();
                println!("Vert: {}", test_vert_xformed);
                */

                // -- draw
                list.draw_indexed_instanced(indices.len() as u32, 1, 0, 0, 0);

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

                        depth_texture_resource = init_depth_texture(
                            window.width(),
                            window.height(),
                            &device,
                            &mut directcommandpool,
                            &mut copycommandpool,
                            &depthstencilviewheap
                        );

                        viewport = t12::SViewport::new(
                            0.0,
                            0.0,
                            window.width() as f32,
                            window.height() as f32,
                            None,
                            None,
                        );

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
