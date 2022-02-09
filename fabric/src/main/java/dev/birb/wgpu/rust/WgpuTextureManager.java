package dev.birb.wgpu.rust;

import net.minecraft.util.Identifier;

import java.util.HashMap;

public class WgpuTextureManager {

    private static HashMap<Identifier, Integer> textures = new HashMap<>();

    public int getTextureId(Identifier id) {
        if(textures.containsKey(id)) {
            return textures.get(id);
        } else {
            int tex_id = WgpuNative.getTextureId(id.toString());
            textures.put(id, tex_id);
            return tex_id;
        }
    }

}
