______________________________________________________________________

## priority: high

# Java 25 with FFM API

**Java 25 · FFM API · Checkstyle · PMD · JUnit 5 · Maven/Gradle**

- Java 25 exclusively; FFM API for native interop, sealed classes, records, pattern matching
- Build: Maven (pom.xml) or Gradle (build.gradle.kts); compiler release=25
- JUnit 5: @Nested classes, @ParameterizedTest, AssertJ fluent assertions, 80%+ coverage
- Checkstyle: 4-space indent, line ≤120 chars, Javadoc on public APIs
- PMD: UnusedVariable, EmptyCatchBlock, AvoidDuplicateLiterals enabled
- FFM patterns: Arena for memory management, try-with-resources, bounds validation
- Naming: PascalCase (classes), camelCase (methods/fields), UPPER_SNAKE_CASE (constants)
- Best practices: final on classes/methods, immutable records, Optional<T> not null
