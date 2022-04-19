package dev.birb.wgpu.mixin.core;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.Mouse;
import net.minecraft.client.util.InputUtil;
import net.minecraft.client.util.Window;
import org.lwjgl.glfw.GLFWDropCallback;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.Redirect;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;

@Mixin(Mouse.class)
public class MouseMixin {

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void setup(long l) {
        MinecraftClient client = MinecraftClient.getInstance();
        Mouse mouse = (Mouse) (Object) this;


    }

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void updateMouse() {

    }

    @Redirect(method = "onMouseButton", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/GlfwUtil;getTime()D"))
    public double getTime() {
        return ((double) System.currentTimeMillis()) / 1000.0D;
    }

    @Redirect(method = "onMouseButton", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;getHandle()J"))
    public long windowHandleMakeEqual(Window instance) {
        return -1;
    }

}
