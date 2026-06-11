import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.sonarqube)
}

val launchTask = gradle
        .startParameter
        .taskRequests
        .toString()
        .lowercase()

if (launchTask.contains("coverage", ignoreCase = true)) {
    apply(plugin = "jacoco")
}

val commonExcludes = listOf(
    // Bindings
    "uniffi/**/",
    // Mockk
    "io/mockk/**/",
)

val unitTestExcludes = listOf(
    // Views
    "**/*View.*",
    "**/*EditText*",
    "**/*TextView*",

    // UI helpers, rendering
    "**/*InterceptInputConnection*",
    "**/*EditorEditTextAttributeReader*",
    "**/internal/view/**/*",
    "**/view/**/*",

    // Compose
    "**/compose/*RichTextEditorKt*",
)

val instrumentationTestExcludes = mutableListOf<String>()

tasks.register<JacocoReport>("generateCoverageReport") {
    outputs.upToDateWhen { false }
    val projects = collectProjects { it.name.contains("library") }
    val executionDataPaths = listOf(
        "**/build/**/*.exec",
        "**/build/outputs/code_coverage/**/coverage.ec",
    )
    val excludes = commonExcludes
    initializeReport(this, projects, excludes, executionDataPaths)
}

tasks.register<JacocoReport>("generateUnitTestCoverageReport") {
    outputs.upToDateWhen { false }
    val projects = collectProjects { listOf("library", "library-compose").contains(it.name) }
    val executionDataPaths = listOf(
        "**/build/outputs/unit_test_code_coverage/**/*.exec",
    )
    val excludes = commonExcludes + unitTestExcludes
    initializeReport(this, projects, excludes, executionDataPaths)
}

tasks.register<JacocoReport>("generateInstrumentationTestCoverageReport") {
    outputs.upToDateWhen { false }
    val projects = collectProjects { listOf("library", "library-compose").contains(it.name) }
    val executionDataPaths = listOf(
        "**/build/outputs/code_coverage/*AndroidTest/connected/**/coverage.ec",
    )
    val excludes = commonExcludes + instrumentationTestExcludes
    initializeReport(this, projects, excludes, executionDataPaths)
}

fun initializeReport(report: JacocoReport, projects: List<Project>, classExcludes: List<String>, executionDataPaths: List<String>) {
    report.executionData(
        fileTree(rootProject.rootDir.absolutePath).include(
            *executionDataPaths.toTypedArray()
        )
    )
    report.reports {
        xml.required = true
        html.required = true
        csv.required = false
    }

    val androidSourceDirs = mutableListOf<String>()
    val androidClassDirs = mutableListOf<String>()

    projects.forEach { project ->
        when {
            project.plugins.hasPlugin("com.android.application") -> {
                androidClassDirs.add("${project.layout.buildDirectory.asFile.get()}/intermediates/built_in_kotlinc/debug/compileDebugKotlin/classes/")
                androidSourceDirs.add("${project.projectDir}/src/main/kotlin")
                androidSourceDirs.add("${project.projectDir}/src/main/java")
            }
            project.plugins.hasPlugin("com.android.library") -> {
                androidClassDirs.add("${project.layout.buildDirectory.asFile.get()}/intermediates/built_in_kotlinc/debug/compileDebugKotlin/classes/")
                androidSourceDirs.add("${project.projectDir}/src/main/kotlin")
                androidSourceDirs.add("${project.projectDir}/src/main/java")
            }
            else -> Unit
        }
    }

    report.sourceDirectories.setFrom(report.sourceDirectories + files(androidSourceDirs))
    val classFiles = androidClassDirs.flatMap { files(it).files }
    report.classDirectories.setFrom(files((report.classDirectories.files + classFiles).map {
        fileTree(baseDir = it) {
            setExcludes(classExcludes)
        }
    }))
}

fun collectProjects(predicate: (Project) -> Boolean): List<Project> {
    return subprojects.filter { it.buildFile.isFile() && predicate(it) }
}

tasks.register<GradleBuild>("unitTestsWithCoverage") {
    startParameter.projectProperties["coverage"] = "true"
    tasks = listOf(":library:testDebugUnitTest", "library-compose:testDebugUnitTest")
}

tasks.register<GradleBuild>("instrumentationTestsWithCoverage") {
    startParameter.projectProperties["coverage"] = "true"
    tasks = listOf(":library:uninstallDebugAndroidTest", ":library-compose:uninstallDebugAndroidTest", ":library:connectedDebugAndroidTest", ":library-compose:connectedDebugAndroidTest")
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


