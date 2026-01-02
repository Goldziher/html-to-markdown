package dev.kreuzberg.htmltomarkdown;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.MethodSource;

import java.io.File;
import java.io.IOException;
import java.util.Arrays;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.assertEquals;

class ComprehensiveTest {
    record TestCase(String name, String html, String expectedMarkdown, Object options) { }

    static Stream<TestCase> basicHtmlProvider() throws IOException {
        ObjectMapper mapper = new ObjectMapper();
        File fixtureFile = new File("../fixtures/basic-html.json");
        TestCase[] cases = mapper.readValue(fixtureFile, TestCase[].class);
        return Arrays.stream(cases);
    }

    @ParameterizedTest(name = "{0}")
    @MethodSource("basicHtmlProvider")
    void testBasicHtmlConversion(TestCase testCase) {
        String result = HtmlToMarkdown.convert(testCase.html());
        assertEquals(testCase.expectedMarkdown().trim(), result.trim());
    }
}
