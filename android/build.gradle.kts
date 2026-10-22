// Versões dos plugins compartilhadas pelos módulos. O Gradle Wrapper (gradlew) fixa o Gradle 8.13.
plugins {
    id("com.android.application") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.21" apply false
}
