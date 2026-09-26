import com.vanniktech.maven.publish.AndroidSingleVariantLibrary
import com.vanniktech.maven.publish.JavadocJar
import com.vanniktech.maven.publish.SourcesJar
import com.vanniktech.maven.publish.DeploymentValidation

plugins {
    alias(libs.plugins.android.library)
    
    alias(libs.plugins.maven.publish)
    alias(libs.plugins.compose.compiler)
}

if (project.hasProperty("coverage")) {
    apply(plugin = "jacoco")

    tasks.withType<Test>().configureEach {
        configure<JacocoTaskExtension> {
            isIncludeNoLocationClasses = true
            excludes = listOf("jdk.internal.*")
        }
    }
}

mavenPublishing {
    publishToMavenCentral(automaticRelease = true, validateDeployment = DeploymentValidation.PUBLISHED)
    signAllPublications()

    configure(AndroidSingleVariantLibrary(
        javadocJar = JavadocJar.None(),
        sourcesJar = SourcesJar.Sources(),
        variant = "release"
    ))
}

android {
    namespace = "io.element.android.wysiwyg.compose.next"
    testNamespace = "io.element.android.wysiwyg.compose.next.test"

    compileSdk = 36

    defaultConfig {
        minSdk = 23

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        debug {
            testCoverage {
                enableUnitTestCoverage = true
                enableAndroidTestCoverage = true
            }
        }
        release {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    testCoverage {
        jacocoVersion = "0.8.12"
    }
    testOptions {
        unitTests.isReturnDefaultValues = true
    }
    buildFeatures {
        compose = true
    }
    packagingOptions {
        resources.excludes += "META-INF/LICENSE.md"
        resources.excludes += "META-INF/LICENSE-notice.md"
    }
}

// Unit tests use a host build of the Rust library if available, see `RustTest`.
// Build it with `cargo build -p uniffi-wysiwyg-composer` from the repository root.
tasks.withType<Test>().configureEach {
    systemProperty("jna.library.path", "${layout.settingsDirectory.asFile}/../../target/debug")
}

kotlin {
    jvmToolchain(21)
    compilerOptions {
        // Needed for TextFieldBuffer.changes and TextFieldState.undoState
        optIn.add("androidx.compose.foundation.ExperimentalFoundationApi")
    }
}

dependencies {
    implementation(project(":library"))

    implementation(libs.timber)
    implementation(libs.jsoup)
    implementation(platform(libs.androidx.compose.bom))
    implementation("androidx.compose.foundation:foundation")
    implementation(libs.androidx.compose.ui)
    releaseImplementation(libs.androidx.compose.ui.tooling.preview)
    debugImplementation(libs.androidx.compose.ui.tooling)

    testImplementation(libs.test.junit)
    testImplementation(libs.test.mockk)
    // Desktop JNA, to load a host build of the Rust library in unit tests
    testImplementation("net.java.dev.jna:jna:5.18.1")
    androidTestImplementation(libs.test.androidx.junit)
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
}

mavenPublishing {
    coordinates(property("MAVEN_GROUP") as String, property("POM_ARTIFACT_ID") as String, property("MAVEN_VERSION_NAME") as String)
    if (!providers.gradleProperty("mavenCentralUsername").isPresent) {
        println("No maven central provider")
    }
    pom {
        name.set("Matrix Rich Text Editor - Compose (native TextField)")
        description.set("Cross-platform rich text editor that generates HTML output.")
        inceptionYear.set("2022")
        url.set("https://github.com/element-hq/matrix-rich-text-editor")
        licenses {
            license {
                name.set("GNU Affero General Public License (AGPL) version 3.0")
                url.set("https://www.gnu.org/licenses/agpl-3.0.txt")
                distribution.set("https://www.gnu.org/licenses/agpl-3.0.txt")
            }
            license {
                name.set("Element Commercial License")
                url.set("https://raw.githubusercontent.com/element-hq/matrix-rich-text-editor/refs/heads/main/LICENSE-COMMERCIAL")
                distribution.set("https://raw.githubusercontent.com/element-hq/matrix-rich-text-editor/refs/heads/main/LICENSE-COMMERCIAL")
            }
        }
        developers {
            developer {
                id.set("matrixdev")
                name.set("matrixdev")
                url.set("https://github.com/element-hq/")
                email.set("android@element.io")
            }
        }
        scm {
            url.set("https://github.com/element-hq/matrix-rich-text-editor/")
            connection.set("scm:git:git://github.com/element-hq/matrix-rich-text-editor.git")
            developerConnection.set("scm:git:ssh://git@github.com/element-hq/matrix-rich-text-editor.git")
        }
    }
}
