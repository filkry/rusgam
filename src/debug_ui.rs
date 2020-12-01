use bvh;
use entity;
use game_context::{SGameContext, SFrameContext};
use game_mode;

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