package dev.birb.wgpu.mixin.disablers;

import net.minecraft.client.util.MonitorTracker;
import org.lwjgl.PointerBuffer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import java.util.function.Supplier;

@Mixin(MonitorTracker.class)
public class MonitorTrackerMixin {

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/systems/RenderSystem;assertThread(Ljava/util/function/Supplier;)V"))
    public void cancelAssertThread(Supplier<Boolean> check) {

    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lorg/lwjgl/glfw/GLFW;glfwGetMonitors()Lorg/lwjgl/PointerBuffer;"))
    public PointerBuffer cancelLWJGLThing() {
        return null;
    }

}
