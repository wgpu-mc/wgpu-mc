package dev.birb.wgpu.mixin.accessors;

import net.minecraft.client.util.Window;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(Window.class)
public interface WindowAccessor {

    @Accessor("height")
    void setHeight(int height);

    @Accessor("width")
    void setWidth(int width);


}
