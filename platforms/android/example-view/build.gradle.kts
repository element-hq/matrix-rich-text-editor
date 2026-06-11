plugins {
    id("com.android.application")
}

android {
    namespace = "io.element.android.wysiwyg.poc"

    compileSdk = 36

    defaultConfig {
        applicationId = "io.element.android.wysiwyg.poc"
        minSdk = 23
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"
    }

    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    buildFeatures {
        viewBinding = true
    }
}

kotlin {
    jvmToolchain(21)
}

dependencies {
    implementation(project(":library"))
    implementation(libs.androidx.core)
    implementation(libs.androidx.appcompat)
    implementation(libs.androidx.constraintlayout)
    implementation(libs.google.material)

    implementation(libs.timber)
}

