//! Moving the pointer over an element applies its `:hover` rules — including
//! when the element declares a CSS transition on the changing property, in
//! which case the style must progress to (and settle at) the hover value.

use anyrender::render_to_buffer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{DocumentConfig, EventDriver, NoopEventHandler};
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_paint::paint_scene;
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, PointerCoords,
    PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};

use std::sync::Arc;

const RED: [u8; 3] = [255, 0, 0];
const BLUE: [u8; 3] = [0, 0, 255];

fn pointer_event(x: f32, y: f32) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons: MouseEventButtons::None,
        mods: Default::default(),
        details: PointerDetails::default(),
    }
}

fn doc(html: &str) -> HtmlDocument {
    let mut doc = HtmlDocument::from_html(
        html,
        DocumentConfig {
            viewport: Some(Viewport::new(200, 200, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(Arc::new(HtmlProvider) as _),
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    doc
}

fn hover_at(doc: &mut HtmlDocument, x: f32, y: f32) {
    let mut driver = EventDriver::new(doc, NoopEventHandler);
    driver.handle_ui_event(UiEvent::PointerMove(pointer_event(x, y)));
}

fn pixel(doc: &mut HtmlDocument, x: usize, y: usize) -> [u8; 3] {
    let buffer = render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| paint_scene(scene, doc, 1.0, 200, 200, 0, 0),
        200,
        200,
    );
    let idx = (y * 200 + x) * 4;
    [buffer[idx], buffer[idx + 1], buffer[idx + 2]]
}

#[test]
fn hover_applies_hover_rules() {
    let mut doc = doc(r#"<html><head><style>
            #btn { width: 44px; height: 36px; background-color: rgb(255, 0, 0); border: none; padding: 0; }
            #btn:hover { background-color: rgb(0, 0, 255); }
        </style></head><body style="margin:0">
        <button id="btn"></button>
    </body></html>"#);

    assert_eq!(pixel(&mut doc, 20, 18), RED);

    hover_at(&mut doc, 20.0, 18.0);
    doc.resolve(0.1);

    assert_eq!(pixel(&mut doc, 20, 18), BLUE);
}

#[test]
fn hover_rules_apply_with_a_transition_declared() {
    // The custom-titlebar button shape: hover restyle + `transition: all`.
    // After the transition duration has elapsed the hover value must hold.
    let mut doc = doc(r#"<html><head><style>
            #btn {
                width: 44px; height: 36px; border: none; padding: 0;
                background-color: rgb(255, 0, 0);
                transition: all 0.15s ease;
            }
            #btn:hover { background-color: rgb(0, 0, 255); }
        </style></head><body style="margin:0">
        <button id="btn"></button>
    </body></html>"#);

    assert_eq!(pixel(&mut doc, 20, 18), RED);

    // The transition starts during the first style resolution after the
    // hover state change (t = 0).
    hover_at(&mut doc, 20.0, 18.0);
    doc.resolve(0.0);

    // Mid-transition: must have left the start color
    doc.resolve(0.075);
    assert_ne!(
        pixel(&mut doc, 20, 18),
        RED,
        "background should be transitioning toward the hover value"
    );

    // Past the end: must have settled at the hover color
    doc.resolve(1.0);
    assert_eq!(pixel(&mut doc, 20, 18), BLUE);
}

#[test]
fn real_titlebar_button_lights_up_on_hover() {
    // Full-fidelity: the kopuz titlebar button with the app's stylesheets.
    static TAILWIND: &str =
        include_str!("/home/umceko/projects/kopuz/crates/kopuz/assets/tailwind.css");
    static MAIN: &str = include_str!("/home/umceko/projects/kopuz/crates/kopuz/assets/main.css");
    let html = format!(
        r#"<html><head><style>{TAILWIND}</style><style>{MAIN}</style></head><body style="margin:0">
        <div class="flex items-center h-9 bg-black/50 border-b border-white/5 flex-shrink-0 select-none relative" style="width:200px;">
            <div class="flex-1"></div>
            <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                <span class="text-[11px] text-white/35 tracking-[0.2em] font-mono uppercase">Kopuz</span>
            </div>
            <div class="flex items-center h-full">
                <button id="btn" class="w-11 h-full flex items-center justify-center text-white/25 hover:text-white/70 hover:bg-white/6 transition-all duration-150">
                    <i class="fa-solid fa-minus text-[10px] leading-none"></i>
                </button>
            </div>
        </div>
    </body></html>"#
    );
    let mut doc = doc(&html);

    // Button occupies x 156..200, y 0..36. Sample away from the glyph.
    let at_rest = pixel(&mut doc, 160, 4);

    hover_at(&mut doc, 178.0, 18.0);
    doc.resolve(0.0);
    doc.resolve(1.0);

    let hovered = pixel(&mut doc, 160, 4);
    assert_ne!(
        hovered, at_rest,
        "hover:bg-white/6 should change the button pixel"
    );
}

#[test]
fn hovering_a_buttons_text_applies_the_buttons_hover_rules() {
    // Hit lands on the glyph (text); hover must still set on the ancestor
    // button via the ancestor chain.
    let mut doc = doc(r#"<html><head><style>
            #btn { width: 44px; height: 36px; background-color: rgb(255, 0, 0); font-size: 20px; border: none; padding: 0; }
            #btn:hover { background-color: rgb(0, 0, 255); }
        </style></head><body style="margin:0">
        <button id="btn">x</button>
    </body></html>"#);

    hover_at(&mut doc, 22.0, 18.0);
    doc.resolve(0.1);

    // Sample a corner pixel away from the glyph
    assert_eq!(pixel(&mut doc, 3, 3), BLUE);
}

/// Tailwind v4 wraps `:hover` rules in `@media (hover: hover)`. A windowed
/// desktop renderer is hover-capable with a fine pointer, so these queries
/// must match (otherwise every hover utility is silently dropped).
#[test]
fn media_hover_and_pointer_queries_match_on_desktop() {
    for query in [
        "(hover: hover)",
        "(any-hover: hover)",
        "(pointer: fine)",
        "(any-pointer: fine)",
        "(hover)",
        "(pointer)",
    ] {
        let mut d = doc(&format!(
            r#"<html><head><style>
                #btn {{ width:44px; height:36px; border:none; padding:0; background:rgb(255,0,0); }}
                @media {query} {{ #btn:hover {{ background-color: rgb(0,0,255); }} }}
            </style></head><body style="margin:0"><button id="btn"></button></body></html>"#
        ));
        assert_eq!(pixel(&mut d, 20, 18), RED, "{query}: rest color");
        hover_at(&mut d, 20.0, 18.0);
        d.resolve(0.1);
        assert_eq!(pixel(&mut d, 20, 18), BLUE, "{query}: hover should apply");
    }
}

/// The negative forms must NOT match on a hover-capable desktop.
#[test]
fn media_hover_none_and_pointer_coarse_do_not_match_on_desktop() {
    for query in ["(hover: none)", "(any-hover: none)", "(pointer: coarse)"] {
        let mut d = doc(&format!(
            r#"<html><head><style>
                #btn {{ width:44px; height:36px; border:none; padding:0; background:rgb(255,0,0); }}
                @media {query} {{ #btn:hover {{ background-color: rgb(0,0,255); }} }}
            </style></head><body style="margin:0"><button id="btn"></button></body></html>"#
        ));
        hover_at(&mut d, 20.0, 18.0);
        d.resolve(0.1);
        assert_eq!(pixel(&mut d, 20, 18), RED, "{query}: must not match");
    }
}
