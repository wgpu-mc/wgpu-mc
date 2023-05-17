plugins {
    id("fabric-loom") version "0.11.29"
    id("fr.stardustenterprises.rust.importer") version "2.1.0"
}
base {
    val archivesBaseName: String by project
    archivesName.set(archivesBaseName)
}

loom {
    accessWidenerPath.set(file("src/main/resources/wgpu_mc.accesswidener"))
    runs {
        configureEach {
            this.isIdeConfigGenerated = true
        }
    }
}

val modVersion: String by project
version = modVersion
val mavenGroup: String by project
group = mavenGroup

dependencies {
    val minecraftVersion: String by project
    val yarnMappings: String by project
    val loaderVersion: String by project
    val fabricVersion: String by project

    // To change the versions see the gradle.properties file
    minecraft("com.mojang:minecraft:$minecraftVersion")
    mappings("net.fabricmc:yarn:$yarnMappings:v2")
    modImplementation("net.fabricmc:fabric-loader:$loaderVersion")

    // Fabric API. This is technically optional, but you probably want it anyway.
    modImplementation("net.fabricmc.fabric-api:fabric-api:$fabricVersion")

//    implementation("fr.stardustenterprises", "yanl", "0.7.1")
    rustImport(project(":wgpu-mc-rust"))
}

tasks {
    processResources {
        finalizedBy("unpackExports", "deleteExports")
    }

    jar {
        dependsOn("unpackExports", "deleteExports")
    }

    fixImport {
        enabled = false
    }

    val javaVersion = JavaVersion.VERSION_17
    withType<JavaCompile> {
        options.encoding = "UTF-8"
        sourceCompatibility = javaVersion.toString()
        targetCompatibility = javaVersion.toString()
        options.release.set(javaVersion.toString().toInt())
    }

    jar { from("LICENSE") { rename { "${it}_${base.archivesName}" } } }
    processResources {
        inputs.property("version", project.version)
        filesMatching("fabric.mod.json") { expand(mutableMapOf("version" to project.version)) }
    }
    java {
        toolchain { languageVersion.set(JavaLanguageVersion.of(javaVersion.toString())) }
        sourceCompatibility = javaVersion
        targetCompatibility = javaVersion
        withSourcesJar()
    }
}

tasks.register<Copy>("unpackExports") {
    from(zipTree(layout.buildDirectory.file("resources/main/export.zip")))
    into(layout.buildDirectory.dir("resources/main"))
    finalizedBy("deleteExports")
}

tasks.register<Delete>("deleteExports") {
    delete(layout.buildDirectory.file("resources/main/export.zip"))
}