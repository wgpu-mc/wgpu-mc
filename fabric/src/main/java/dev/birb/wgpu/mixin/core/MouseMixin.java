package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.Mouse;
import net.minecraft.client.util.GlfwUtil;
import net.minecraft.client.util.InputUtil;
import net.minecraft.client.util.SmoothUtil;
import net.minecraft.client.util.Window;
import org.checkerframework.checker.units.qual.A;
import org.lwjgl.glfw.GLFWDropCallback;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.Redirect;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;

@Mixin(Mouse.class)
public abstract class MouseMixin {

    @Shadow private boolean cursorLocked;

    @Shadow private double cursorDeltaX;

    @Shadow @Final private SmoothUtil cursorXSmoother;

    @Shadow @Final private SmoothUtil cursorYSmoother;

    @Shadow private double cursorDeltaY;

    @Shadow @Final private MinecraftClient client;

    @Shadow public abstract boolean isCursorLocked();

    @Shadow private double lastMouseUpdateTime;

    /**
     * @author wgpu-mc
     */
    @Overwrite
    public void setup(long l) {
        MinecraftClient client = MinecraftClient.getInstance();
        Mouse mouse = (Mouse) (Object) this;


    }

    @Redirect(method = "*", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/GlfwUtil;getTime()D"))
    public double getTime() {
        return ((double) System.currentTimeMillis()) / 1000.0D;
    }

    @Redirect(method = "onMouseButton", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/util/Window;getHandle()J"))
    public long windowHandleMakeEqual(Window instance) {
        return -1;
    }

    @Overwrite
    public void lockCursor() {
        this.cursorLocked = true;
        WgpuNative.setCursorLocked(true);
    }

    @Overwrite
    public void unlockCursor() {
        this.cursorLocked = false;
        WgpuNative.setCursorLocked(false);
    }

}
