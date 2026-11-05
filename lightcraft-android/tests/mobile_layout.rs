use lightcraft_ui_egui::{LightcraftApp, Services, headless::Headless, state::RightPanel};
use serde_json::json;
use std::time::Duration;

fn phone(size: [f32; 2]) -> Headless {
    let mut app = LightcraftApp::new(lightcraft_engine::Session::new(), Services::default());
    app.mobile_host = true;
    app.ui.left_panel = false;
    app.ui.right = RightPanel::None;
    app.ui.filmstrip = false;
    let mut view = Headless::new(app, size, 1.0);
    for _ in 0..5 {
        view.step();
    }
    view
}

#[test]
fn phone_navigation_and_header_stay_inside_portrait_and_landscape() {
    for size in [[380.0, 853.0], [853.0, 380.0]] {
        let mut view = phone(size);
        let widgets = view.request("ui.widgets", json!({"filter":"mobile:"}), Duration::from_secs(5));
        assert_eq!(widgets["ok"], true, "{widgets}");
        let buttons = widgets["result"].as_array().unwrap();
        assert!(buttons.len() >= 7, "{widgets}");
        for widget in buttons {
            let rect = &widget["rect"];
            assert!(rect[0].as_f64().unwrap() >= 0.0 && rect[1].as_f64().unwrap() >= 0.0, "{widget}");
            assert!(rect[0].as_f64().unwrap() + rect[2].as_f64().unwrap() <= size[0] as f64 + 0.5, "{widget}");
            assert!(rect[1].as_f64().unwrap() + rect[3].as_f64().unwrap() <= size[1] as f64 + 0.5, "{widget}");
        }
    }
}

#[test]
fn export_dialog_actions_remain_on_screen_on_a_short_phone() {
    let size = [853.0, 380.0];
    let mut view = phone(size);
    view.request("engine.execute", json!({"command":"dialog.export"}), Duration::from_secs(5));
    for _ in 0..5 {
        view.step();
    }
    let widgets = view.request("ui.widgets", json!({"filter":"button:dialog"}), Duration::from_secs(5));
    let buttons = widgets["result"].as_array().unwrap();
    assert!(!buttons.is_empty(), "{widgets}");
    for widget in buttons {
        let r = &widget["rect"];
        assert!(r[1].as_f64().unwrap() + r[3].as_f64().unwrap() <= size[1] as f64, "{widget}");
    }
}

#[test]
fn rounded_corners_pad_only_the_navigation_controls() {
    use lightcraft_ui_egui::{
        ControlRequest, control,
        screen_edges::{self, Corner, Geometry},
    };
    let mut app = LightcraftApp::new(lightcraft_engine::Session::new(), Services::default());
    app.mobile_host = true;
    app.ui.left_panel = false;
    app.ui.right = RightPanel::None;
    let ctx = egui::Context::default();
    for frame in 0..5 {
        let input = lightcraft_ui_egui::headless::HeadlessView::raw_input(egui::vec2(380.0, 853.0), 1.0, frame as f64 / 60.0, vec![]);
        let mut output = ctx.run_ui(input, |ui| {
            screen_edges::set(
                &ctx,
                Geometry {
                    corners: vec![
                        Corner { center: egui::pos2(40.0, 40.0), radius: 40.0, position: 0 },
                        Corner { center: egui::pos2(340.0, 40.0), radius: 40.0, position: 1 },
                        Corner { center: egui::pos2(340.0, 813.0), radius: 40.0, position: 2 },
                        Corner { center: egui::pos2(40.0, 813.0), radius: 40.0, position: 3 },
                    ],
                },
            );
            app.logic(&ctx);
            app.ui(ui);
        });
        output.textures_delta.clear();
    }
    let (req, _) = ControlRequest::new("ui.widgets", json!({"filter":"mobile:library"}));
    let control::Outcome::Done(out) = control::handle(&mut app, &ctx, &req) else { panic!("expected widget list") };
    assert!(out["result"][0]["rect"][0].as_f64().unwrap() >= 39.0, "{out}");
    assert_eq!(ctx.content_rect().width(), 380.0);
}
