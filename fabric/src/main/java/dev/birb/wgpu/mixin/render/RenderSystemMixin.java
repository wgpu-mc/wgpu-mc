package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.shaders.ShaderType;
import com.mojang.blaze3d.systems.GpuDevice;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.VertexFormat;
import dev.birb.wgpu.backend.WgpuBackend;
import net.minecraft.client.gl.DynamicUniforms;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.BuiltBuffer;
import net.minecraft.client.render.VertexFormats;
import net.minecraft.client.util.BufferAllocator;
import net.minecraft.util.Identifier;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWNativeCocoa;
import org.lwjgl.glfw.GLFWNativeWin32;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.function.BiFunction;

import static com.mojang.blaze3d.systems.RenderSystem.getDevice;

@Mixin(RenderSystem.class)
public abstract class RenderSystemMixin {
    @Shadow private static @Nullable GpuDevice DEVICE;

    @Shadow private static String apiDescription;

    @Shadow private static @Nullable DynamicUniforms dynamicUniforms;

    @Shadow private static @Nullable GpuBuffer QUAD_VERTEX_BUFFER;

    @Redirect(method = "flipFrame", at = @At(value = "INVOKE", target = "Lorg/lwjgl/glfw/GLFW;glfwSwapBuffers(J)V"))
    private static void dontSwapBuffers(long window) {

    }

    @Inject(method = "initRenderer", at = @At(value = "NEW", target = "net/minecraft/client/gl/GlBackend"), cancellable = true)
    private static void newWgpuBackend(long windowHandle, int debugVerbosity, boolean sync, BiFunction<Identifier, ShaderType, String> shaderSourceGetter, boolean renderDebugLabels, CallbackInfo ci) {
        DEVICE = new WgpuBackend(windowHandle, shaderSourceGetter);

        dynamicUniforms = new DynamicUniforms();
        apiDescription = getDevice().getImplementationInformation();

        try (BufferAllocator bufferAllocator = new BufferAllocator(VertexFormats.POSITION.getVertexSize() * 4)) {
            BufferBuilder bufferBuilder = new BufferBuilder(bufferAllocator, VertexFormat.DrawMode.QUADS, VertexFormats.POSITION);
            bufferBuilder.vertex(0.0F, 0.0F, 0.0F);
            bufferBuilder.vertex(1.0F, 0.0F, 0.0F);
            bufferBuilder.vertex(1.0F, 1.0F, 0.0F);
            bufferBuilder.vertex(0.0F, 1.0F, 0.0F);

            try (BuiltBuffer builtBuffer = bufferBuilder.end()) {
                QUAD_VERTEX_BUFFER = getDevice().createBuffer(() -> "Quad", 32, builtBuffer.getBuffer());
            }
        }

        ci.cancel();
    }

}
