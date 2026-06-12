//! overflow:hidden establishes a scroll container, but the element's
//! intrinsic content contribution must still size auto grid tracks / auto
//! heights.

use blitz_dom::DocumentConfig;
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_traits::shell::{ColorScheme, Viewport};
use std::sync::Arc;

fn layout_doc(html: &str) -> HtmlDocument {
    let mut doc = HtmlDocument::from_html(
        html,
        DocumentConfig {
            viewport: Some(Viewport::new(800, 600, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(Arc::new(HtmlProvider) as _),
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    doc
}

#[test]
fn block_overflow_hidden_card_sizes_grid_track() {
    // The kopuz radio-card shape: a block display, overflow:hidden child of
    // an auto-row grid, with fixed-height content inside.
    let doc = layout_doc(
        r#"<html><body style="margin:0">
            <div style="display:grid; grid-template-columns: 1fr 1fr; gap:16px; width:600px;">
                <div id="card" style="overflow:hidden;">
                    <div style="height:168px;"></div>
                </div>
            </div>
        </body></html>"#,
    );
    let card = doc.query_selector("#card").unwrap().expect("#card");
    let layout = doc.get_node(card).unwrap().final_layout;
    assert_eq!(
        (layout.size.width, layout.size.height),
        (292.0, 168.0),
        "block overflow:hidden grid child must size its auto track from content"
    );
}

#[test]
fn flex_overflow_hidden_card_sizes_grid_track() {
    // Control: the flex variant (this one was already fixed upstream).
    let doc = layout_doc(
        r#"<html><body style="margin:0">
            <div style="display:grid; grid-template-columns: 1fr 1fr; gap:16px; width:600px;">
                <div id="card" style="display:flex; overflow:hidden;">
                    <div style="height:168px; width:50px; flex-shrink:0;"></div>
                </div>
            </div>
        </body></html>"#,
    );
    let card = doc.query_selector("#card").unwrap().expect("#card");
    let layout = doc.get_node(card).unwrap().final_layout;
    assert_eq!(
        (layout.size.width, layout.size.height),
        (292.0, 168.0),
        "flex overflow:hidden grid child must size its auto track from content"
    );
}

#[test]
fn block_overflow_hidden_auto_height_in_block_flow() {
    let doc = layout_doc(
        r#"<html><body style="margin:0">
            <div style="width:300px;">
                <div id="card" style="overflow:hidden;">
                    <div style="height:168px;"></div>
                </div>
            </div>
        </body></html>"#,
    );
    let card = doc.query_selector("#card").unwrap().expect("#card");
    let layout = doc.get_node(card).unwrap().final_layout;
    assert_eq!(
        (layout.size.width, layout.size.height),
        (300.0, 168.0),
        "block overflow:hidden in block flow must size auto height from content"
    );
}

