enableFeaturePreview("TYPESAFE_PROJECT_ACCESSORS")
enableFeaturePreview("VERSION_CATALOGS")

pluginManagement {
    repositories {
        gradlePluginPortal()
        maven("https://maven.fabricmc.net")
    }
}

rootProject.name = "wgpu-mc"

subProject("rust");
subProject("fabric");

fun subProject(name: String) {
    setupSubproject("wgpu-mc-$name") {
        projectDir = file(name)
    }
}

inline fun setupSubproject(name: String, block: ProjectDescriptor.() -> Unit) {
    include(name)
    project(":$name").apply(block)
}