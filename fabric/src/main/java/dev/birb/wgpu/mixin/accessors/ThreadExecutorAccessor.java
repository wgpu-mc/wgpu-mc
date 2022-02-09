package dev.birb.wgpu.mixin.accessors;

import net.minecraft.util.thread.ThreadExecutor;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import java.util.Queue;

@Mixin(ThreadExecutor.class)
public interface ThreadExecutorAccessor {

    @Accessor("tasks")
    Queue<Runnable> getTasks();

}
