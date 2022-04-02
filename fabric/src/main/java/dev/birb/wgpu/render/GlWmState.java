package dev.birb.wgpu.render;

import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;

public class GlWmState {

    public static List<WmTexture> generatedTextures = new ArrayList<>();
    public static HashMap<Integer, Integer> textureSlots = new HashMap<>();
    public static int activeTexture = 0;

    public static class WmTexture {
        public ByteBuffer buffer;
        public int width;
        public int height;

        public WmTexture() {
            this(null);
        }

        public WmTexture(ByteBuffer buffer) {
            this.buffer = buffer;
        }

    }

}
