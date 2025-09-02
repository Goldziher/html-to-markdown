from html_to_markdown import convert_to_markdown
from html_to_markdown.preprocessor import create_preprocessor, preprocess_html


class TestHTMLPreprocessor:
    def test_remove_navigation(self) -> None:
        html = """
        <html>
        <body>
            <nav>
                <ul>
                    <li><a href="#home">Home</a></li>
                    <li><a href="#about">About</a></li>
                </ul>
            </nav>
            <main>
                <h1>Main Content</h1>
                <p>This is the actual content.</p>
            </main>
        </body>
        </html>
        """

        config = create_preprocessor("standard", remove_navigation=True)
        cleaned = preprocess_html(html, **config)

        assert "<nav>" not in cleaned
        assert "Home" not in cleaned
        assert "About" not in cleaned

        assert "Main Content" in cleaned
        assert "actual content" in cleaned

    def test_remove_forms(self) -> None:
        html = """
        <html>
        <body>
            <h1>Contact Us</h1>
            <form>
                <label for="name">Name:</label>
                <input type="text" id="name" name="name">
                <button type="submit">Submit</button>
            </form>
            <p>Thank you for visiting!</p>
        </body>
        </html>
        """

        config = create_preprocessor("standard", remove_forms=True)
        cleaned = preprocess_html(html, **config)

        assert "<form>" not in cleaned
        assert "<input>" not in cleaned
        assert "<button>" not in cleaned
        assert "<label>" not in cleaned

        assert "Contact Us" in cleaned
        assert "Thank you for visiting!" in cleaned

    def test_preserve_tables(self) -> None:
        html = """
        <html>
        <body>
            <table>
                <thead>
                    <tr><th>Name</th><th>Age</th></tr>
                </thead>
                <tbody>
                    <tr><td>John</td><td>30</td></tr>
                    <tr><td>Jane</td><td>25</td></tr>
                </tbody>
            </table>
        </body>
        </html>
        """

        config = create_preprocessor("standard", preserve_tables=True)
        cleaned = preprocess_html(html, **config)

        assert "<table>" in cleaned
        assert "<thead>" in cleaned
        assert "<tbody>" in cleaned
        assert "<tr>" in cleaned
        assert "<th>" in cleaned
        assert "<td>" in cleaned


class TestIntegratedPreprocessing:
    def test_wikipedia_style_cleaning(self) -> None:
        html = """
        <html>
        <body>
            <a href="#bodyContent">Jump to content</a>
            <input type="checkbox" id="vector-main-menu-dropdown-checkbox" />
            <label for="vector-main-menu-dropdown-checkbox">Main menu</label>
            <button>move to sidebar</button>
            <button>hide</button>
            <nav>
                <a href="/wiki/Main_Page">Main page</a>
            </nav>
            <main id="bodyContent">
                <h1>Article Title</h1>
                <p>This is the actual article content that should be preserved.</p>
            </main>
        </body>
        </html>
        """

        result_no_preprocess = convert_to_markdown(html, preprocess_html=False)

        result_with_preprocess = convert_to_markdown(html, preprocess_html=True, preprocessing_preset="standard")

        assert "Jump to content" not in result_with_preprocess
        assert "Main menu" not in result_with_preprocess
        assert "move to sidebar" not in result_with_preprocess
        assert "hide" not in result_with_preprocess

        assert "Article Title" in result_with_preprocess
        assert "actual article content" in result_with_preprocess

        assert len(result_with_preprocess) < len(result_no_preprocess)

    def test_form_removal_integration(self) -> None:
        html = """
        <html>
        <body>
            <h1>Page Title</h1>
            <form method="post">
                <fieldset>
                    <legend>User Information</legend>
                    <label for="email">Email:</label>
                    <input type="email" id="email" name="email" required>
                    <button type="submit">Subscribe</button>
                </fieldset>
            </form>
            <p>Important information below the form.</p>
        </body>
        </html>
        """

        result = convert_to_markdown(html, preprocess_html=True, remove_forms=True)

        assert "email" not in result.lower()
        assert "subscribe" not in result.lower()
        assert "fieldset" not in result.lower()

        assert "Page Title" in result
        assert "Important information" in result

    def test_minimal_preset(self) -> None:
        html = """
        <html>
        <body>
            <header>Site Header</header>
            <nav>Navigation</nav>
            <main>
                <article>
                    <h1>Article</h1>
                    <p>Content</p>
                </article>
            </main>
            <aside>Sidebar</aside>
            <footer>Footer</footer>
        </body>
        </html>
        """

        result = convert_to_markdown(html, preprocess_html=True, preprocessing_preset="minimal")

        assert "Article" in result
        assert "Content" in result

        assert "Navigation" not in result

    def test_aggressive_preset(self) -> None:
        html = """
        <html>
        <body>
            <header>Site Header</header>
            <nav>Navigation</nav>
            <main>
                <h1>Main Title</h1>
                <p>Main content paragraph.</p>
            </main>
            <aside>Sidebar content</aside>
            <footer>Site Footer</footer>
        </body>
        </html>
        """

        result = convert_to_markdown(html, preprocess_html=True, preprocessing_preset="aggressive")

        assert "Main Title" in result
        assert "Main content" in result

        assert "Site Header" not in result
        assert "Navigation" not in result
        assert "Sidebar" not in result
        assert "Site Footer" not in result

    def test_wikipedia_navigation_removal(self) -> None:
        html = """
        <html>
        <body>
            <div class="vector-header">Header content</div>
            <div class="mw-jump-link"><a href="#content">Jump to content</a></div>
            <nav class="vector-main-menu">
                <ul>
                    <li><a href="/wiki/Contents">Contents</a></li>
                    <li><a href="/wiki/Current_events">Current events</a></li>
                    <li><a href="/wiki/Random">Random article</a></li>
                </ul>
            </nav>
            <main id="content">
                <h1>Article Title</h1>
                <p>Actual article content here.</p>
            </main>
        </body>
        </html>
        """

        result = convert_to_markdown(html, preprocess_html=True, remove_navigation=True)

        assert "vector-header" not in result
        assert "mw-jump-link" not in result
        assert "vector-main-menu" not in result

        assert "Article Title" in result
        assert "Actual article content" in result

        result_no_preprocess = convert_to_markdown(html, preprocess_html=False)
        assert len(result) < len(result_no_preprocess)

    def test_custom_tag_removal(self) -> None:
        html = """
        <html>
        <body>
            <main>
                <h1>Title</h1>
                <p>Important content</p>
                <advertisement>Ad content here</advertisement>
                <cookie-banner>Cookie notice</cookie-banner>
                <social-widgets>Social media widgets</social-widgets>
            </main>
        </body>
        </html>
        """

        config = create_preprocessor(
            "standard", custom_tags_to_remove={"advertisement", "cookie-banner", "social-widgets"}
        )
        cleaned = preprocess_html(html, **config)

        assert "advertisement" not in cleaned.lower()
        assert "cookie" not in cleaned.lower()
        assert "social" not in cleaned.lower()

        assert "Title" in cleaned
        assert "Important content" in cleaned

    def test_class_based_navigation_removal(self) -> None:
        html = """
        <html>
        <body>
            <div class="navbar-header">Site navigation</div>
            <div class="sidebar-menu">Sidebar menu</div>
            <div class="breadcrumb-nav">Home > Page</div>
            <main class="content">
                <h1>Main Content</h1>
                <p>This should be preserved.</p>
            </main>
            <footer class="site-footer">Footer content</footer>
        </body>
        </html>
        """

        config = create_preprocessor("standard", remove_navigation=True)
        cleaned = preprocess_html(html, **config)

        assert "Site navigation" not in cleaned
        assert "Sidebar menu" not in cleaned
        assert "Home >" not in cleaned

        assert "Main Content" in cleaned
        assert "This should be preserved" in cleaned

    def test_direct_function_usage(self) -> None:
        html = """
        <html>
        <body>
            <nav>Navigation</nav>
            <form><input type="text"><button>Submit</button></form>
            <main>
                <h1>Content</h1>
                <table><tr><td>Data</td></tr></table>
            </main>
        </body>
        </html>
        """

        cleaned = preprocess_html(html, remove_navigation=True, remove_forms=True, preserve_tables=True)

        assert "<nav>" not in cleaned
        assert "<form>" not in cleaned
        assert "<input>" not in cleaned
        assert "<button>" not in cleaned

        assert "Content" in cleaned
        assert "<table>" in cleaned
        assert "Data" in cleaned


def test_preprocessor_edge_cases() -> None:
    result = preprocess_html("<p>test</p>", remove_navigation=True)
    assert "test" in result

    result = preprocess_html("", remove_navigation=True)
    assert result == ""

    config = create_preprocessor(preset="minimal")
    assert isinstance(config, dict)

    config = create_preprocessor(preset="aggressive", remove_navigation=False)
    assert isinstance(config, dict)

    malformed = "<div><p>unclosed paragraph<div>nested</div>"
    result = preprocess_html(malformed, remove_navigation=False)
    assert "unclosed paragraph" in result

    nav_html = """
    <header>Header nav</header>
    <nav>Main nav</nav>
    <aside>Sidebar</aside>
    <div class="navigation">Class-based nav</div>
    <div id="menu">ID-based nav</div>
    <p>Content</p>
    """
    result = preprocess_html(nav_html, remove_navigation=True)
    assert "Header nav" not in result
    assert "Main nav" not in result
    assert "Class-based nav" not in result
    assert "Content" in result


def test_form_removal_edge_cases() -> None:
    form_html = """
    <form>Form content</form>
    <input type="text" value="input">
    <textarea>textarea content</textarea>
    <select><option>option</option></select>
    <button>button</button>
    <fieldset><legend>legend</legend></fieldset>
    <p>Keep this content</p>
    """

    result = preprocess_html(form_html, remove_forms=True)

    assert "Form content" not in result
    assert "input" not in result
    assert "textarea content" not in result
    assert "option" not in result
    assert "button" not in result
    assert "legend" not in result
    assert "Keep this content" in result
