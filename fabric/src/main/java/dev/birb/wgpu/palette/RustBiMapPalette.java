package dev.birb.wgpu.palette;

import net.minecraft.network.PacketByteBuf;
import net.minecraft.world.chunk.Palette;

import java.util.function.Predicate;

public class RustBiMapPalette<T> implements Palette<T> {

    @Override
    public int index(T object) {
        return 0;
    }

    @Override
    public boolean hasAny(Predicate<T> predicate) {
        return false;
    }

    @Override
    public T get(int id) {
        return null;
    }

    @Override
    public void readPacket(PacketByteBuf buf) {

    }

    @Override
    public void writePacket(PacketByteBuf buf) {

    }

    @Override
    public int getPacketSize() {
        return 0;
    }

    @Override
    public int getSize() {
        return 0;
    }

    @Override
    public Palette<T> copy() {
        return null;
    }
}
