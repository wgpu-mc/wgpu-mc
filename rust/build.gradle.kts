plugins {
    id("fr.stardustenterprises.rust.wrapper") version "2.1.0"
}

rust {
    command = "cargo"

    environment = mapOf("RUSTUP_TOOLCHAIN" to "nightly")

    outputs = mapOf("" to System.mapLibraryName("wgpu_mc_jni"))

    outputDirectory = "META-INF/natives"

    profile = "release"
}