// Módulo do aplicativo. Compile com "compile.bat android" (APK de debug).
// API_BASE_URL vira BuildConfig.API_BASE_URL; padrão 10.0.2.2 = localhost do PC visto pelo emulador.
// Para outra URL, no cmd: "set API_BASE_URL=https://api.seu-dominio.com" e depois "compile android".
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}
android {
    namespace = "br.com.motoclube.coins"
    compileSdk = 35
    defaultConfig {
        applicationId = "br.com.motoclube.coins"
        minSdk = 26
        targetSdk = 35
        versionCode = 2
        versionName = "0.2"
        buildConfigField("String", "API_BASE_URL", "\"${project.findProperty("API_BASE_URL") ?: "http://10.0.2.2:5000"}\"")
    }
    buildFeatures { compose = true; buildConfig = true }
    compileOptions { sourceCompatibility = JavaVersion.VERSION_17; targetCompatibility = JavaVersion.VERSION_17 }
    kotlinOptions { jvmTarget = "17" }
}
dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.12.01"))
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.foundation:foundation")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")
}
