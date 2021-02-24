package cloud.birb.wgpu.mixin;
import net.minecraft.client.WindowSettings;
import net.minecraft.client.util.Window;
import net.minecraft.client.util.WindowProvider;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(WindowProvider.class)
public class WindowProviderMixin {

    @Inject(method = "createWindow", at = @At("HEAD"), cancellable = true)
    public void createWindow(WindowSettings settings, String videoMode, String title, CallbackInfoReturnable<Window> cir) {
        cir.setReturnValue(null);
    }

}
