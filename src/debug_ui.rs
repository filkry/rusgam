use allocate::{STACK_ALLOCATOR};
use bvh;
use collections::{SVec};
use entity;
use entity_model;
use game_context::{SGameContext, SFrameContext};
use game_mode;
use math::{Vec3, Vec4};
use render;

pub fn update_debug_entity_menu(game_context: &SGameContext, frame_context: &SFrameContext) {
    use imgui::*;

    game_context.data_bucket.get::<entity::SEntityBucket>()
        .and::<game_mode::SGameMode>()
        .with_mc(|entity, game_mode| {
            if game_mode.edit_mode_ctxt.editing_entity().is_none() {
                return;
            }
            let e = game_mode.edit_mode_ctxt.editing_entity().expect("checked above");

            let imgui_ui = frame_context.imgui_ui.as_ref().expect("shouldn't have rendered ui yet");

            Window::new(im_str!("Selected entity"))
                .size([300.0, 300.0], Condition::FirstUseEver)
                .build(imgui_ui, || {
                    if !entity.entity_valid(e) {
                        imgui_ui.text(im_str!("INVALID entity selected"));
                        return;
                    }

                    if let Some (n) = entity.get_entity_debug_name(e) {
                        imgui_ui.text(im_str!("debug_name: {}", n._debug_ptr.as_ref().expect("")));
                    }

                    imgui_ui.text(im_str!("index: {}, generation: {}", e.index(), e.generation()));
                    imgui_ui.separator();
                    let mut pos = {
                        let t = entity.get_entity_location(e).t;
                        [t.x, t.y, t.z]
                    };
                    if DragFloat3::new(imgui_ui, im_str!("Position"), &mut pos).speed(0.1).build() {
                        entity.set_position(&game_context, e, Vec3::new(pos[0], pos[1], pos[2]));
                    }
                });
        });
}

pub fn update_debug_main_menu(game_context: &SGameContext, frame_context: &SFrameContext) {
    game_context.data_bucket.get::<game_mode::SGameMode>()
        //.and::<gjk::SGJKDebug>() // $$$FRK(TOOD): restore this by making it possible to click two entities
        .and::<bvh::STree<entity::SEntityHandle>>()
        .with_mc(|game_mode, bvh| {
            let imgui_ui = frame_context.imgui_ui.as_ref().expect("this should happen before imgui render");

            if let game_mode::EMode::Edit = game_mode.mode {

                if game_mode.show_imgui_demo_window {
                    let mut opened = true;
                    imgui_ui.show_demo_window(&mut opened);
                }

                imgui_ui.main_menu_bar(|| {
                    imgui_ui.menu(imgui::im_str!("Misc"), true, || {
                        if imgui::MenuItem::new(imgui::im_str!("Toggle Demo Window")).build(&imgui_ui) {
                            game_mode.show_imgui_demo_window = !game_mode.show_imgui_demo_window;
                        }
                    });

                    bvh.imgui_menu(&imgui_ui, &mut game_mode.draw_selected_bvh);

                    //gjk_debug.imgui_menu(&imgui_ui, &game_context.data_bucket, game_mode.edit_mode_ctxt.editing_entity(), Some(rotating_entity));
                });
            }
        });
}

pub fn update_draw_entity_bvh(game_context: &SGameContext, _frame_context: &SFrameContext) {
    // -- draw selected object's BVH heirarchy
    STACK_ALLOCATOR.with(|sa| {
        game_context.data_bucket.get::<render::SRender>()
            .and::<entity_model::SBucket>()
            .and::<bvh::STree<entity::SEntityHandle>>()
            .and::<game_mode::SGameMode>()
            .with_mccc(|render, em, bvh, game_mode| {
                if game_mode.draw_selected_bvh {
                    if let Some(e) = game_mode.edit_mode_ctxt.editing_entity() {
                        let model_handle = em.handle_for_entity(e).unwrap();

                        let mut aabbs = SVec::new(&sa.as_ref(), 32, 0).unwrap();
                        bvh.get_bvh_heirarchy_for_entry(em.get_bvh_entry(model_handle).unwrap(), &mut aabbs);
                        for aabb in aabbs.as_slice() {
                            render.temp().draw_aabb(aabb, &Vec4::new(1.0, 0.0, 0.0, 1.0), true);
                        }
                    }
                }
            });
        });
}

/*
pub fn update_debug_draw_entity_colliding() {
    // -- draw selected object colliding/not with rotating_entity
    STACK_ALLOCATOR.with(|sa| {
        game_context.data_bucket.get::<render::SRender>()
            .and::<entity::SEntityBucket>()
            .and::<entity_model::SBucket>()
            .and::<game_mode::SGameMode>()
            .with_mccc(|render, entities, em, game_mode| {
                if let Some(e) = game_mode.edit_mode_ctxt.editing_entity() {
                    let e_model_handle = em.handle_for_entity(e).unwrap();
                    let rot_model_handle = em.handle_for_entity(rotating_entity).unwrap();
                    let loc = entities.get_entity_location(e);

                    let world_verts = {
                        let model = em.get_model(e_model_handle);
                        let mesh_local_vs = render.mesh_loader().get_mesh_local_vertices(model.mesh);

                        let mut world_verts = SVec::new(&sa.as_ref(), mesh_local_vs.len(), 0).unwrap();

                        for v in mesh_local_vs.as_slice() {
                            world_verts.push(loc.mul_point(&v));
                        }

                        world_verts
                    };

                    let rot_box_world_verts = {
                        let model = em.get_model(rot_model_handle);
                        let loc = entities.get_entity_location(rotating_entity);
                        let mesh_local_vs = render.mesh_loader().get_mesh_local_vertices(model.mesh);

                        let mut world_verts = SVec::new(&sa.as_ref(), mesh_local_vs.len(), 0).unwrap();

                        for v in mesh_local_vs.as_slice() {
                            world_verts.push(loc.mul_point(&v));
                        }

                        world_verts
                    };

                    if gjk::gjk(world_verts.as_slice(), rot_box_world_verts.as_slice()) {
                        render.temp().draw_sphere(&loc.t, 1.0, &Vec4::new(1.0, 0.0, 0.0, 0.1), true, None);
                    }
                }
            });
    });
}
*/
