package dev.birb.wgpu.render;

import lombok.Getter;
import lombok.Setter;

import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class GlWmState {

    @Getter
    private static final List<WmTexture> generatedTextures = new ArrayList<>();
    @Getter
    private static final Map<Integer, Integer> textureSlots = new HashMap<>();
    @Getter
    private static final Map<Integer, Integer> pixelStore = new HashMap<>();

    @Getter
    @Setter
    private static int activeTexture = 0;

    @Getter
    @Setter
    public static class WmTexture {
        private ByteBuffer buffer;
        private int width;
        private int height;

        public WmTexture() {
            this(null);
        }

        public WmTexture(ByteBuffer buffer) {
            this.buffer = buffer;
        }
    }
}
