//! UA stylesheet: browsers center the text of <button> elements
//! (text-align: center in every major UA sheet).

use blitz_dom::DocumentConfig;
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_traits::shell::{ColorScheme, Viewport};
use std::sync::Arc;

#[test]
fn button_text_is_centered() {
    let mut doc = HtmlDocument::from_html(
        r#"<html><body style="margin:0">
            <button style="width:200px; padding:0; border:none;">
                <span id="label" style="display:inline-block; width:30px; height:10px;"></span>
            </button>
        </body></html>"#,
        DocumentConfig {
            viewport: Some(Viewport::new(800, 600, 1.0, ColorScheme::Light)),
            html_parser_provider: Some(Arc::new(HtmlProvider) as _),
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    let label = doc.query_selector("#label").unwrap().expect("#label");
    let layout = doc.get_node(label).unwrap().final_layout;
    let expected_x = (200.0 - layout.size.width) / 2.0;
    assert!(
        (layout.location.x - expected_x).abs() <= 1.0,
        "button label should be horizontally centered: x = {}, expected ~{} (label width {})",
        layout.location.x,
        expected_x,
        layout.size.width,
    );
}
