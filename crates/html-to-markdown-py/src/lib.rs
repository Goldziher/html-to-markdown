use pyo3::prelude::*;

/// Convert HTML to Markdown
#[pyfunction]
fn convert(html: &str) -> PyResult<String> {
    Ok(html_to_markdown::convert(html))
}

/// Python bindings for html-to-markdown
#[pymodule]
fn _html_to_markdown(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    Ok(())
}
