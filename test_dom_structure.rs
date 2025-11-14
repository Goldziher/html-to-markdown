use html_to_markdown_rs::convert;

fn main() {
    let html = r#"<div>before</div>
<script type="text/javascript">const msg = "<tag>";</script>
<p>after</p>"#;

    let result = convert(html, None).unwrap();
    println!("Result: {:?}", result);
    println!("Contains 'after': {}", result.contains("after"));
}
