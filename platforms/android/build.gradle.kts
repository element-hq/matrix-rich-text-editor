import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.rust.android)
    alias(libs.plugins.sonarqube)
}

val launchTask = gradle
        .startParameter
        .taskRequests
        .toString()
        .lowercase()

if (launchTask.contains("coverage")) {
    apply(from = "coverage.gradle")
}

subprojects {
    tasks.withType<KotlinCompile>().configureEach {
        val buildDirPath = project.layout.buildDirectory.get().asFile.absolutePath
        compilerOptions {
            if (project.findProperty("composeCompilerReports") == "true") {
                freeCompilerArgs.addAll(
                        listOf(
                                "-P",
                                "plugin:androidx.compose.compiler.plugins.kotlin:reportsDestination=$buildDirPath/compose_compiler"
                        )
                )
            }
            if (project.findProperty("composeCompilerMetrics") == "true") {
                freeCompilerArgs.addAll(
                        listOf(
                                "-P",
                                "plugin:androidx.compose.compiler.plugins.kotlin:metricsDestination=$buildDirPath/compose_compiler"
                        )
                )
            }
        }
    }
}


