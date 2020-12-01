use allocate::{SMemVec, STACK_ALLOCATOR};
use bvh;
use entity;
use entity_model;
use game_context::{SGameContext, SFrameContext};
use game_mode;
use glm::{Vec4};
use render;

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

pub fn update_debug_entity_menu(game_context: &SGameContext, frame_context: &SFrameContext) {
    game_context.data_bucket.get::<game_mode::SGameMode>()
        .and::<entity::SEntityBucket>()
        .with_mm(|game_mode, entities| {
            let imgui_ui = frame_context.imgui_ui.as_ref().expect("this should happen before imgui render");
            if let game_mode::EMode::Edit = game_mode.mode {
                if let Some(e) = game_mode.edit_mode_ctxt.editing_entity() {
                    entities.show_imgui_window(e, &imgui_ui);
                }
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

                        let mut aabbs = SMemVec::new(&sa.as_ref(), 32, 0).unwrap();
                        bvh.get_bvh_heirarchy_for_entry(em.get_bvh_entry(model_handle).unwrap(), &mut aabbs);
                        for aabb in aabbs.as_slice() {
                            render.temp().draw_aabb(aabb, &Vec4::new(1.0, 0.0, 0.0, 1.0), true);
                        }
                    }
                }
            });
        });
}