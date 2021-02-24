package cloud.birb.wgpu.mixin;

import net.minecraft.client.font.FontManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import java.util.function.Consumer;

@Mixin(FontManager.class)
public class FontManagerMixin {

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/util/Util;make(Ljava/lang/Object;Ljava/util/function/Consumer;)Ljava/lang/Object;"))
    public Object cancelMake(Object object, Consumer<Object> initializer) {
        return null;
    }

}
