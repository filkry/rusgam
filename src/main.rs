extern crate arrayvec;
extern crate nalgebra;
extern crate nalgebra_glm as glm;
extern crate tinytga;
extern crate tobj;
extern crate winapi;
extern crate wio;
extern crate bitflags;

//mod math;
mod allocate;
mod collections;
mod directxgraphicssamples;
mod niced3d12;
mod rustywindows;
mod safewindows;
mod typeyd3d12;
mod utils;
mod enumflags;
mod camera;
mod model;

// -- std includes
use std::cell::RefCell;
use std::mem::size_of;

// -- crate includes
use arrayvec::ArrayVec;

use niced3d12 as n12;
use typeyd3d12 as t12;

#[allow(dead_code)]
type SMat44 = nalgebra::Matrix4<f32>;
type SVec3 = nalgebra::Vector3<f32>;

pub struct SInput {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    space: bool,
    c: bool,
    mouse_dx: i32,
    mouse_dy: i32,
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

fn init_depth_texture(
    width: u32,
    height: u32,
    device: &n12::SDevice,
    direct_command_pool: &mut n12::SCommandListPool,
    copy_command_pool: &mut n12::SCommandListPool,
    depth_descriptor_heap: &n12::SDescriptorHeap,
) -> Result<n12::SResource, &'static str> {
    direct_command_pool.flush_blocking().unwrap();
    copy_command_pool.flush_blocking().unwrap();

    let clear_value = t12::SClearValue {
        format: t12::EDXGIFormat::D32Float,
        value: t12::EClearValue::DepthStencil(t12::SDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        }),
    };

    // -- need to not let this be destroyed
    let mut _depth_texture_resource = device.create_committed_texture2d_resource(
        t12::EHeapType::Default,
        width,
        height,
        1,
        0,
        t12::EDXGIFormat::D32Float,
        Some(clear_value),
        t12::SResourceFlags::from(t12::EResourceFlags::AllowDepthStencil),
        t12::EResourceStates::DepthWrite,
    )?;

    let depth_stencil_view_desc = t12::SDepthStencilViewDesc {
        format: t12::EDXGIFormat::D32Float,
        view_dimension: t12::EDSVDimension::Texture2D,
        flags: t12::SDSVFlags::from(t12::EDSVFlags::None),
        data: t12::EDepthStencilViewDescData::Tex2D(t12::STex2DDSV { mip_slice: 0 }),
    };

    device.create_depth_stencil_view(
        &mut _depth_texture_resource,
        &depth_stencil_view_desc,
        depth_descriptor_heap.cpu_handle_heap_start(),
    )?;

    Ok(_depth_texture_resource)
}

fn main_d3d12() -> Result<(), &'static str> {
    // -- initialize debug
    let debuginterface = t12::SDebugInterface::new()?;
    debuginterface.enabledebuglayer();

    // -- setup window and command queue
    let winapi = rustywindows::SWinAPI::create();
    let windowclass = winapi.rawwinapi().registerclassex("rusgam").unwrap();

    let mut factory = n12::SFactory::create()?;
    let mut adapter = factory.create_best_adapter()?;
    let mut device = adapter.create_device()?;

    let commandqueue = RefCell::new(
        device.create_command_queue(&winapi.rawwinapi(), t12::ECommandListType::Direct)?,
    );
    let mut directcommandpool =
        n12::SCommandListPool::create(&device, &commandqueue, &winapi.rawwinapi(), 1, 2)?;

    let copycommandqueue = RefCell::new(
        device.create_command_queue(&winapi.rawwinapi(), t12::ECommandListType::Copy)?,
    );
    let mut copycommandpool =
        n12::SCommandListPool::create(&device, &copycommandqueue, &winapi.rawwinapi(), 1, 2)?;

    let mut window = n12::SD3D12Window::new(
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

    let depthstencilviewheap = {
        let desc = t12::SDescriptorHeapDesc {
            type_: t12::EDescriptorHeapType::DepthStencil,
            num_descriptors: 1,
            flags: t12::SDescriptorHeapFlags::from(t12::EDescriptorHeapFlags::None),
        };

        device.create_descriptor_heap(&desc)?
    };

    let srv_heap = RefCell::new(n12::descriptorallocator::SDescriptorAllocator::new(
        &device,
        32,
        t12::EDescriptorHeapType::ConstantBufferShaderResourceUnorderedAccess,
        t12::SDescriptorHeapFlags::from(t12::EDescriptorHeapFlags::ShaderVisible),
    )?);

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

    let model = model::SModel::new_from_obj("assets/first_test_asset.obj", &device, &mut copycommandpool, &mut directcommandpool, &srv_heap)?;
    let model2 = model::SModel::new_from_obj("assets/first_test_asset.obj", &device, &mut copycommandpool, &mut directcommandpool, &srv_heap)?;

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
            t12::SInputElementDesc::create(
                "TEXCOORD",
                0,
                t12::EDXGIFormat::R32G32Float,
                0,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
        ];

        t12::SInputLayoutDesc::create(&input_element_desc)
    };

    let mvp_root_parameter = t12::SRootParameter {
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

    let texture_root_parameter = {
        let descriptor_range = t12::SDescriptorRange {
            range_type: t12::EDescriptorRangeType::SRV,
            num_descriptors: 1,
            base_shader_register: 0,
            register_space: 0,
            offset_in_descriptors_from_table_start: t12::EDescriptorRangeOffset::EAppend,
        };

        let mut root_descriptor_table = t12::SRootDescriptorTable::new();
        root_descriptor_table
            .descriptor_ranges
            .push(descriptor_range);

        t12::SRootParameter {
            type_: t12::ERootParameterType::DescriptorTable,
            type_data: t12::ERootParameterTypeData::DescriptorTable {
                table: root_descriptor_table,
            },
            shader_visibility: t12::EShaderVisibility::Pixel,
        }
    };

    let sampler = t12::SStaticSamplerDesc {
        filter: t12::EFilter::MinMagMipPoint,
        address_u: t12::ETextureAddressMode::Border,
        address_v: t12::ETextureAddressMode::Border,
        address_w: t12::ETextureAddressMode::Border,
        mip_lod_bias: 0.0,
        max_anisotropy: 0,
        comparison_func: t12::EComparisonFunc::Never,
        border_color: t12::EStaticBorderColor::OpaqueWhite,
        min_lod: 0.0,
        max_lod: std::f32::MAX,
        shader_register: 0,
        register_space: 0,
        shader_visibility: t12::EShaderVisibility::Pixel,
    };

    let root_signature_flags = t12::SRootSignatureFlags::create(&[
        t12::ERootSignatureFlags::AllowInputAssemblerInputLayout,
        t12::ERootSignatureFlags::DenyHullShaderRootAccess,
        t12::ERootSignatureFlags::DenyDomainShaderRootAccess,
        t12::ERootSignatureFlags::DenyGeometryShaderRootAccess,
    ]);

    let mut root_signature_desc = t12::SRootSignatureDesc::new(root_signature_flags);
    root_signature_desc.parameters.push(mvp_root_parameter);
    root_signature_desc.parameters.push(texture_root_parameter);
    root_signature_desc.static_samplers.push(sampler);

    let root_signature =
        device.create_root_signature(root_signature_desc, t12::ERootSignatureVersion::V1)?;

    let mut rtv_formats = t12::SRTFormatArray {
        rt_formats: ArrayVec::new(),
    };
    rtv_formats.rt_formats.push(t12::EDXGIFormat::R8G8B8A8UNorm);

    // -- pipeline state object
    let pipeline_state_stream = SPipelineStateStream {
        root_signature: n12::SPipelineStateStreamRootSignature::create(&root_signature),
        input_layout: n12::SPipelineStateStreamInputLayout::create(&mut input_layout_desc),
        primitive_topology: n12::SPipelineStateStreamPrimitiveTopology::create(
            t12::EPrimitiveTopologyType::Triangle,
        ),
        vertex_shader: n12::SPipelineStateStreamVertexShader::create(&vert_byte_code),
        pixel_shader: n12::SPipelineStateStreamPixelShader::create(&pixel_byte_code),
        depth_stencil_format: n12::SPipelineStateStreamDepthStencilFormat::create(
            t12::EDXGIFormat::D32Float,
        ),
        rtv_formats: n12::SPipelineStateStreamRTVFormats::create(&rtv_formats),
    };
    let pipeline_state_stream_desc = t12::SPipelineStateStreamDesc::create(&pipeline_state_stream);
    let pipeline_state = device
        .raw()
        .create_pipeline_state(&pipeline_state_stream_desc)?;

    // -- depth texture
    #[allow(unused_variables)]
    let mut _depth_texture_resource = init_depth_texture(
        window.width(),
        window.height(),
        &device,
        &mut directcommandpool,
        &mut copycommandpool,
        &depthstencilviewheap,
    );

    // -- update loop

    let mut _framecount: u64 = 0;
    let mut lastframetime = winapi.curtimemicroseconds();

    let mut framefencevalues = [0; 2];

    let mut shouldquit = false;

    let start_time = winapi.curtimemicroseconds();
    let rot_axis = SVec3::new(0.0, 1.0, 0.0);

    let mut camera = camera::SCamera::new(glm::Vec3::new(0.0, 0.0, -10.0));

    let mut input = SInput{
        w: false,
        a: false,
        s: false,
        d: false,
        space: false,
        c: false,

        mouse_dx: 0,
        mouse_dy: 0,
    };

    while !shouldquit {
        let curframetime = winapi.curtimemicroseconds();
        let dt = curframetime - lastframetime;
        let _dtms = dt as f64;
        let dts = (dt as f32) / 1_000_000.0;

        let total_time = curframetime - start_time;

        // -- update
        let cur_angle = ((total_time as f32) / 1_000_000.0) * (3.14159 / 4.0);
        let model_matrix = SMat44::new_rotation(rot_axis * cur_angle);
        let model2_matrix = glm::translation(&glm::Vec3::new(1.0, 0.0, 0.0));

        let perspective_matrix: SMat44 = {
            let aspect = (window.width() as f32) / (window.height() as f32);
            let fovy: f32 = 3.14159 / 4.0;
            let znear = 0.1;
            let zfar = 100.0;

            //SMat44::new_perspective(aspect, fovy, znear, zfar)
            glm::perspective_lh(aspect, fovy, znear, zfar)
        };

        camera.update_from_input(&input, dts);
        input.mouse_dx = 0;
        input.mouse_dy = 0;
        let view_matrix = camera.to_view_matrix();

        //println!("View: {}", view_matrix);
        //println!("Perspective: {}", perspective_matrix);

        //println!("Frame time: {}us", _dtms);

        // -- wait for buffer to be available
        commandqueue
            .borrow()
            .wait_for_internal_fence_value(framefencevalues[window.currentbackbufferindex()]);

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
                list.clear_depth_stencil_view(depthstencilviewheap.cpu_handle_heap_start(), 1.0)?;

                // -- set up pipeline
                list.set_pipeline_state(&pipeline_state);
                // root signature has to be set explicitly despite being on PSO, according to tutorial
                list.set_graphics_root_signature(&root_signature.raw());

                // -- setup rasterizer state
                list.rs_set_viewports(&[&viewport]);
                list.rs_set_scissor_rects(t12::SScissorRects::create(&[&scissorrect]));

                // -- setup the output merger
                list.om_set_render_targets(&[&render_target_view], false, &depth_texture_view);

                model.render(list, &(perspective_matrix * view_matrix), &model_matrix);
                model2.render(list, &(perspective_matrix * view_matrix), &model2_matrix);

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
            framefencevalues[window.currentbackbufferindex()] =
                commandqueue.borrow_mut().signal_internal_fence()?;

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
                        safewindows::EKey::W => input.w = true,
                        safewindows::EKey::A => input.a = true,
                        safewindows::EKey::S => input.s = true,
                        safewindows::EKey::D => input.d = true,
                        safewindows::EKey::Space => input.space = true,
                        safewindows::EKey::C => input.c = true,
                        _ => (),
                    },
                    safewindows::EMsgType::KeyUp { key } => match key {
                        safewindows::EKey::W => input.w = false,
                        safewindows::EKey::A => input.a = false,
                        safewindows::EKey::S => input.s = false,
                        safewindows::EKey::D => input.d = false,
                        safewindows::EKey::Space => input.space = false,
                        safewindows::EKey::C => input.c = false,
                        _ => (),
                    },
                    safewindows::EMsgType::Input{ raw_input } => {
                        if let safewindows::rawinput::ERawInputData::Mouse{data} = raw_input.data {
                            //println!("Frame {}: Raw Mouse: {}, {}", _framecount, data.last_x, data.last_y);
                            input.mouse_dx = data.last_x;
                            input.mouse_dy = data.last_y;
                        }
                    },
                    safewindows::EMsgType::Size => {
                        //println!("Size");
                        let rect: safewindows::SRect = window.raw().getclientrect()?;
                        let newwidth = rect.right - rect.left;
                        let newheight = rect.bottom - rect.top;

                        window.resize(
                            newwidth as u32,
                            newheight as u32,
                            &mut commandqueue.borrow_mut(),
                            &mut device,
                        )?;

                        _depth_texture_resource = init_depth_texture(
                            window.width(),
                            window.height(),
                            &device,
                            &mut directcommandpool,
                            &mut copycommandpool,
                            &depthstencilviewheap,
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

fn debug_test() {}

fn main() {
    debug_test();

    main_d3d12().unwrap();
}
