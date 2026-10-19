import java.net.HttpURLConnection
import java.net.URL
import java.util.zip.ZipFile
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.library") version "9.4.0"
}

group = "io.xberg.android"
version = "0.1.0"

android {
    namespace = "io.xberg.android.e2e"
    compileSdk = 35

    defaultConfig {
        minSdk = 21
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    testOptions {
        // Host JVM unit tests: no Android device/emulator required.
        // Tests run against the published AAR and JVM-side deps via `gradle test`.
        unitTests {
            isReturnDefaultValues = true
        }
    }
}

kotlin {
    // Set JVM target for compilation. gradle.properties enables auto-detection
    // of host JDK installations so Gradle uses the available JDK version on the
    // build machine, preventing provisioning failures when the target version is not installed.
    jvmToolchain(17)
    compilerOptions {
        jvmTarget = JvmTarget.JVM_17
    }
}

// Repositories declared in settings.gradle.kts via
// dependencyResolutionManagement (FAIL_ON_PROJECT_REPOS). Re-declaring them
// here triggers Gradle "repository was added by build file" errors.

dependencies {
    // Published Android AAR from Maven Central (verifies artifact resolution)
    implementation("io.xberg:html-to-markdown-android:3.18.4")
    // Jackson for JSON assertion helpers
    testImplementation("com.fasterxml.jackson.core:jackson-annotations:2.22")
    testImplementation("com.fasterxml.jackson.core:jackson-databind:2.22.2")
    testImplementation("com.fasterxml.jackson.datatype:jackson-datatype-jdk8:2.22.2")

    // jackson-module-kotlin registers constructors/properties for Kotlin data
    // classes, which have no default constructor and cannot be deserialized by
    // plain Jackson without this module.
    testImplementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.22.2")

    // jspecify for null-safety annotations on wrapped types
    testImplementation("org.jspecify:jspecify:1.0.1")

    // Kotlin coroutines for async test helpers
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.11.0")

    // JUnit 5 API and engine
    testImplementation("org.junit.jupiter:junit-jupiter-api:6.1.3")
    testImplementation("org.junit.jupiter:junit-jupiter-engine:6.1.3")
    testImplementation("org.junit.platform:junit-platform-launcher:6.1.3")

    // Kotlin stdlib test helpers
    testImplementation(kotlin("test"))

    // JNA for loading the native library from java.library.path
    testImplementation("net.java.dev.jna:jna:5.19.1")

}

tasks.register("verifyAarPublished") {
    description = "Verify the published Android AAR contains jni and classes.jar"
    doLast {
        val aarCoord = "io.xberg:html-to-markdown-android:3.18.4"
        val (groupId, artifactId, version) = run {
            val parts = aarCoord.split(':')
            Triple(parts[0], parts[1], parts[2])
        }
        val aarFileName = "${artifactId}-${version}.aar"
        val mavenUrl = "https://repo1.maven.org/maven2/${groupId.replace('.', '/')}/${artifactId}/${version}/${aarFileName}"
        val aarFile = layout.buildDirectory.file("tmp/${aarFileName}").get().asFile

        println("Downloading AAR from Maven Central: ${mavenUrl}")
        aarFile.parentFile.mkdirs()

        val connection = URL(mavenUrl).openConnection() as HttpURLConnection
        connection.requestMethod = "GET"
        connection.connect()

        if (connection.responseCode != 200) {
            throw GradleException("Failed to download AAR: HTTP ${connection.responseCode}")
        }

        connection.inputStream.use { input ->
            aarFile.outputStream().use { output ->
                input.copyTo(output)
            }
        }

        println("Verifying AAR contents...")
        ZipFile(aarFile).use { zip ->
            val entries = zip.entries().toList()
            val hasJni = entries.any { it.name.startsWith("jni/") }
            val hasClasses = entries.any { it.name == "classes.jar" }

            if (!hasJni) {
                throw GradleException("AAR missing jni directory")
            }
            if (!hasClasses) {
                throw GradleException("AAR missing classes.jar")
            }

            val abiDirs = entries
                .filter { it.name.startsWith("jni/") }
                .map { it.name.substringAfter("jni/").substringBefore("/") }
                .filter { it.isNotEmpty() }
                .distinct()

            println("  + jni: YES")
            println("  + classes.jar: YES")
            println("  + Android ABIs: " + abiDirs.sorted().joinToString(", "))
            println("\nAAR verification PASSED!")
        }
    }
}

// Build host JNI library for JVM unit tests (macOS/Linux/Windows).
// The generated Kotlin Bridge object calls System.loadLibrary("htm_jni") for JVM
// unit tests running on developer machines. This task builds the host-platform binary
// and stages it into src/test/resources/host-jni/<platform>/ for the test loader.
// Set alef.skipHostJni=true to disable this (e.g., in CI where only AAR validation is needed).
tasks.register("buildHostJni", Exec::class) {
    if (project.properties["alef.skipHostJni"] != "true") {
        val jniCargoPath = "../../crates/html-to-markdown-rs-jni/Cargo.toml"
        description = "Build host-platform JNI library from ../../crates/html-to-markdown-rs-jni"
        commandLine("cargo", "build", "--release", "--manifest-path", jniCargoPath)
        errorOutput = System.err
    } else {
        description = "Build host JNI (disabled via alef.skipHostJni=true)"
        commandLine("true")
    }
}

tasks.register("copyHostJni", Copy::class) {
    if (project.properties["alef.skipHostJni"] != "true") {
        description = "Copy host JNI library to test resources"
        dependsOn("buildHostJni")

        val hostPlatform = if (System.getProperty("os.name").lowercase().contains("mac")) {
            "darwin"
        } else if (System.getProperty("os.name").lowercase().contains("win")) {
            "windows"
        } else {
            "linux"
        }
        val libName = when (hostPlatform) {
            "darwin" -> "libhtm_jni.dylib"
            "windows" -> "htm_jni.dll"
            else -> "libhtm_jni.so"
        }

        // Cargo builds to the workspace target directory by default, even when
        // --manifest-path points at a member crate. The previous
        // `if (workspaceTarget.exists()) ... else crateTarget` dual-path was
        // evaluated at gradle configuration time, before `cargo build` finished
        // or before the workspace target dir existed, so the glob could match
        // zero files and the test runtime would fail with `UnsatisfiedLinkError`
        // at static-init time. Always read from the workspace target.
        val workspaceTarget = file("../../target/release")

        from(workspaceTarget) {
            include(libName)
        }
        into(layout.projectDirectory.dir("src/test/resources/host-jni/$hostPlatform"))
    }
}

tasks.withType<Test> {
    useJUnitPlatform()
    dependsOn("verifyAarPublished")
    if (project.properties["alef.skipHostJni"] != "true") {
        val hostPlatform = if (System.getProperty("os.name").lowercase().contains("mac")) {
            "darwin"
        } else if (System.getProperty("os.name").lowercase().contains("win")) {
            "windows"
        } else {
            "linux"
        }
        systemProperty(
            "java.library.path",
            project.layout.projectDirectory.dir("src/test/resources/host-jni/$hostPlatform").asFile.absolutePath
        )
        dependsOn("copyHostJni")
    }
}

tasks.matching { it.name.startsWith("processDebug") || it.name.startsWith("processRelease") }.configureEach {
    if (project.properties["alef.skipHostJni"] != "true" && name.contains("UnitTestJavaRes")) {
        dependsOn("copyHostJni")
    }
}
