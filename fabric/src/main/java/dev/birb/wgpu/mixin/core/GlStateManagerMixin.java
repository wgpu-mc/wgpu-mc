package dev.birb.wgpu.mixin.core;

import com.mojang.blaze3d.platform.GlStateManager;
import dev.birb.wgpu.render.GlWmState;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.util.math.Matrix4f;
import net.minecraft.util.math.Vec3f;
import net.minecraft.util.math.Vector4f;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.opengl.GL11;
import org.lwjgl.opengl.GL30;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.ByteBuffer;
import java.nio.FloatBuffer;
import java.nio.IntBuffer;
import java.util.List;

@SuppressWarnings("OverwriteAuthorRequired")
@Mixin(GlStateManager.class)
public class GlStateManagerMixin {

    private static void combineColor(int combineColor, int source0Color) {
    }

    private static void combineColor(int combineColor, int source0Color, int source1Color, int source2Color) {
    }

    private static void combineAlpha(int combineAlpha, int source0Alpha) {
    }

    private static FloatBuffer getBuffer(float a, float b, float c, float d) {
        return null;
    }

//    @Overwrite(remap = false)
//    @Overwrite(remap = false)
//    public static void _bindTexture(int texture) {
//        WgpuNative.bindTexture(texture);
//    }
//
//    @Overwrite(remap = false)
//    @Overwrite(remap = false)
//    public static void _texImage2D(int target, int level, int internalFormat, int width, int height, int border, int format, int type, @Nullable IntBuffer pixels) {
//        long pointer;
//        if(pixels != null) {
//
//        } else {
//
//        }
//        WgpuNative.texImage2D(target, level, internalFormat, width, height, border, format, type, pointer);
//    }
//
//    @Overwrite(remap = false)
//    @Overwrite(remap = false)
//    public static void _texSubImage2D(int target, int level, int offsetX, int offsetY, int width, int height, int format, int type, long pixels) {
//        WgpuNative.texSubImage2D(target, level, offsetX, offsetY, width, height, format, type, pixels);
//    }
//
//    @Overwrite(remap = false)
//    @Overwrite(remap = false)
//    public static void _getTexImage(int target, int level, int format, int type, long pixels) {
//    }

    @Overwrite(remap = false)
    public static void _disableScissorTest() {
    }

    @Overwrite(remap = false)
    public static void _enableScissorTest() {
    }

    @Overwrite(remap = false)
    public static void _scissorBox(int x, int y, int width, int height) {
    }

    @Overwrite(remap = false)
    public static void _disableDepthTest() {
    }

    @Overwrite(remap = false)
    public static void _enableDepthTest() {
    }

    @Overwrite(remap = false)
    public static void _depthFunc(int func) {


    }

    @Overwrite(remap = false)
    public static void _depthMask(boolean mask) {


    }

    @Overwrite(remap = false)
    public static void _disableBlend() {
    }

    @Overwrite(remap = false)
    public static void _enableBlend() {
    }

    @Overwrite(remap = false)
    public static void _blendFunc(int srcFactor, int dstFactor) {


    }

    @Overwrite(remap = false)
    public static void _blendFuncSeparate(int srcFactorRGB, int dstFactorRGB, int srcFactorAlpha, int dstFactorAlpha) {


    }

    @Overwrite(remap = false)
    public static void _blendEquation(int mode) {
    }

    @Overwrite(remap = false)
    public static int glGetProgrami(int program, int pname) {
        return 0;
    }

    @Overwrite(remap = false)
    public static void glAttachShader(int program, int shader) {
    }

    @Overwrite(remap = false)
    public static void glDeleteShader(int shader) {
    }

    @Overwrite(remap = false)
    public static int glCreateShader(int type) {
        return 0;
    }

    @Overwrite(remap = false)
    public static void glShaderSource(int shader, List<String> strings) {
    }

    @Overwrite(remap = false)
    public static void glCompileShader(int shader) {
    }

    @Overwrite(remap = false)
    public static int glGetShaderi(int shader, int pname) {
        return 0;
    }

    @Overwrite(remap = false)
    public static void _glUseProgram(int program) {
    }

    @Overwrite(remap = false)
    public static int glCreateProgram() {
        return 0;
    }

    @Overwrite(remap = false)
    public static void glDeleteProgram(int program) {
    }

    @Overwrite(remap = false)
    public static void glLinkProgram(int program) {
    }

    @Overwrite(remap = false)
    public static int _glGetUniformLocation(int program, CharSequence name) {
        return 0;
    }

    @Overwrite(remap = false)
    public static void _glUniform1(int location, IntBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniform1i(int location, int value) {
    }

    @Overwrite(remap = false)
    public static void _glUniform1(int location, FloatBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniform2(int location, IntBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniform2(int location, FloatBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniform3(int location, IntBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniform3(int location, FloatBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniform4(int location, IntBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniform4(int location, FloatBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniformMatrix2(int location, boolean transpose, FloatBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniformMatrix3(int location, boolean transpose, FloatBuffer value) {
    }

    @Overwrite(remap = false)
    public static void _glUniformMatrix4(int location, boolean transpose, FloatBuffer value) {
    }

    @Overwrite(remap = false)
    public static int _glGetAttribLocation(int program, CharSequence name) {
        return 0;
    }

    @Overwrite(remap = false)
    public static void _glBindAttribLocation(int program, int index, CharSequence name) {
    }

    @Overwrite(remap = false)
    public static int _glGenBuffers() {
        return -1;
    }

    @Overwrite(remap = false)
    public static int _glGenVertexArrays() {
//        return WgpuNative.gen
        return 1;
    }

    @Overwrite(remap = false)
    public static void _glBindBuffer(int target, int buffer) {
    }

    @Overwrite(remap = false)
    public static void _glBindVertexArray(int array) {
    }

    @Overwrite(remap = false)
    public static void _glBufferData(int target, ByteBuffer data, int usage) {
    }

    @Overwrite(remap = false)
    public static void _glBufferData(int target, long size, int usage) {
    }

    @Nullable
    @Overwrite(remap = false)
    public static ByteBuffer mapBuffer(int target, int access) {
        return ByteBuffer.allocate(0);
    }

    @Overwrite(remap = false)
    public static void _glUnmapBuffer(int target) {
    }

    @Overwrite(remap = false)
    public static void _glDeleteBuffers(int buffer) {

    }

    @Overwrite(remap = false)
    public static void _glCopyTexSubImage2D(int target, int level, int xOffset, int yOffset, int x, int y, int width, int height) {
    }

    @Overwrite(remap = false)
    public static void _glDeleteVertexArrays(int array) {
    }

    @Overwrite(remap = false)
    public static void _glBindFramebuffer(int target, int framebuffer) {
    }

    @Overwrite(remap = false)
    public static void _glBlitFrameBuffer(int srcX0, int srcY0, int srcX1, int srcY1, int dstX0, int dstY0, int dstX1, int dstY1, int mask, int filter) {
    }

    @Overwrite(remap = false)
    public static void _glBindRenderbuffer(int target, int renderbuffer) {
    }

    @Overwrite(remap = false)
    public static void _glDeleteRenderbuffers(int renderbuffer) {
    }

    @Overwrite(remap = false)
    public static void _glDeleteFramebuffers(int framebuffer) {
    }

    @Overwrite(remap = false)
    public static int glGenFramebuffers() {
        return 0;
    }

    @Overwrite(remap = false)
    public static int glGenRenderbuffers() {
        return 0;
    }

    @Overwrite(remap = false)
    public static void _glRenderbufferStorage(int target, int internalFormat, int width, int height) {
    }

    @Overwrite(remap = false)
    public static void _glFramebufferRenderbuffer(int target, int attachment, int renderbufferTarget, int renderbuffer) {
    }

    @Overwrite(remap = false)
    public static int glCheckFramebufferStatus(int target) {
        return 0;
    }

    @Overwrite(remap = false)
    public static void _glFramebufferTexture2D(int target, int attachment, int textureTarget, int texture, int level) {
    }

    @Overwrite(remap = false)
    public static int getBoundFramebuffer() {
        return 0;
    }

    @Overwrite(remap = false)
    public static void glActiveTexture(int texture) {
        GlWmState.activeTexture = texture;
    }

    @Overwrite(remap = false)
    public static void glBlendFuncSeparate(int srcFactorRGB, int dstFactorRGB, int srcFactorAlpha, int dstFactorAlpha) {
    }

    @Overwrite(remap = false)
    public static String glGetShaderInfoLog(int shader, int maxLength) {
        return "Shader info log stub";
    }

    @Overwrite(remap = false)
    public static String glGetProgramInfoLog(int program, int maxLength) {
        return "Program info log stub";
    }

    @Overwrite(remap = false)
    public static void setupLevelDiffuseLighting(Vec3f vec3f, Vec3f vec3f2, Matrix4f matrix4f) {
        Vector4f vector4f2 = new Vector4f(vec3f2);
    }

    @Overwrite(remap = false)
    public static void setupGuiFlatDiffuseLighting(Vec3f vec3f, Vec3f vec3f2) {
//        matrix4f.multiply(Matrix4f.scale(1.0F, -1.0F, 1.0F));
    }

    @Overwrite(remap = false)
    public static void setupGui3DDiffuseLighting(Vec3f vec3f, Vec3f vec3f2) {
//        matrix4f.multiply(Vec3f.POSITIVE_Y.getDegreesQuaternion(62.0F));
//        setupLevelDiffuseLighting(vec3f, vec3f2, matrix4f);
    }

    @Overwrite(remap = false)
    public static void _enableCull() {
    }

    @Overwrite(remap = false)
    public static void _disableCull() {
    }

    @Overwrite(remap = false)
    public static void _polygonMode(int face, int mode) {
    }

    @Overwrite(remap = false)
    public static void _enablePolygonOffset() {
    }

    @Overwrite(remap = false)
    public static void _disablePolygonOffset() {
    }

    @Overwrite(remap = false)
    public static void _polygonOffset(float factor, float units) {


    }

    @Overwrite(remap = false)
    public static void _enableColorLogicOp() {
    }

    @Overwrite(remap = false)
    public static void _disableColorLogicOp() {
    }

    @Overwrite(remap = false)
    public static void _logicOp(int op) {


    }

    @Overwrite(remap = false)
    public static void _activeTexture(int texture) {


    }

    @Overwrite(remap = false)
    public static void _enableTexture() {
    }

    @Overwrite(remap = false)
    public static void _disableTexture() {
    }

    @Overwrite(remap = false)
    public static void _texParameter(int target, int pname, float param) {
    }

    @Overwrite(remap = false)
    public static void _texParameter(int target, int pname, int param) {
    }

    @Overwrite(remap = false)
    public static int _getTexLevelParameter(int target, int level, int pname) {
        return 0;
    }

    @Overwrite(remap = false)
    public static int _genTexture() {
        GlWmState.generatedTextures.add(new GlWmState.WmTexture());
        return GlWmState.generatedTextures.size() - 1;
    }

    @Overwrite(remap = false)
    public static void _genTextures(int[] textures) {
        for(int i = 0; i < textures.length; i++) {
            textures[i] = _genTexture();
        }
    }

    @Overwrite(remap = false)
    public static void _deleteTexture(int texture) {
//        int var2 = var1.length;
//        WgpuNative
    }

    @Overwrite(remap = false)
    public static void _deleteTextures(int[] textures) {

    }

    @Overwrite(remap = false)
    public static void _bindTexture(int texture) {
//        WgpuNative.bindTexture(0, texture);
        GlWmState.textureSlots.put(GlWmState.activeTexture, texture);
    }

    @Overwrite(remap = false)
    public static int _getTextureId(int texture) {
        return GlWmState.textureSlots.get(texture);
    }

    @Overwrite(remap = false)
    public static int _getActiveTexture() {
        return GlWmState.activeTexture;
//        return WgpuNative.get
    }

    @Overwrite(remap = false)
    public static void _texImage2D(int target, int level, int internalFormat, int width, int height, int border, int format, int type, @Nullable IntBuffer pixels) {
        if(level != 0) return;

        int texId = GlWmState.textureSlots.get(GlWmState.activeTexture);
        GlWmState.WmTexture texture = GlWmState.generatedTextures.get(texId);

        if(width < texture.width || height < texture.height) {
            System.out.println("Tried to make texture smaller?");
            return;
        }

        texture.width = width;
        texture.height = height;

        long ptr = 0;
        if(pixels != null) {
            ptr = MemoryUtil.memAddress(pixels);
        }
        WgpuNative.texImage2D(texId, target, level, internalFormat, width, height, border, format, type, ptr);
    }

    @Overwrite(remap = false)
    public static void _texSubImage2D(int target, int level, int offsetX, int offsetY, int width, int height, int format, int type, long pixels) {
        //we do not care about mip maps
        if(level != 0) 
            return;

        if(format != GL11.GL_RGBA && format != 0x80E1)
            return;

        int texId = GlWmState.textureSlots.get(GlWmState.activeTexture);
        GlWmState.WmTexture texture = GlWmState.generatedTextures.get(texId);

        int unpack_row_length = GlWmState.pixelStore.getOrDefault(GL30.GL_UNPACK_ROW_LENGTH, 0);
        int unpack_skip_pixels = GlWmState.pixelStore.getOrDefault(GL30.GL_UNPACK_SKIP_PIXELS, 0);
        int unpack_skip_rows = GlWmState.pixelStore.getOrDefault(GL30.GL_UNPACK_SKIP_ROWS, 0);
        int unpack_alignment = GlWmState.pixelStore.getOrDefault(GL30.GL_UNPACK_ALIGNMENT, 4);

        if(width + offsetX <= texture.width && height + offsetY <= texture.height) {
            int[] pixel_array = new int[width*height];

            long pixel_size = 4L; //TODO support more formats..?
            for(int y = 0; y < height; y++) {
                for(int x = 0; x < width; x++){
                    int current_x = x + unpack_skip_pixels;
                    int current_y = (y + unpack_skip_rows) * 
                        (unpack_row_length > 0 ? unpack_row_length : width);
                    
                    //TODO row_byte_offset proper impl || let row_byte_offset = if pixel_size >= unpack_alignment
                    long offset = (current_x + current_y) * pixel_size;
                    pixel_array[x + y * width] = MemoryUtil.memGetInt(pixels+offset);
                }
            }
            
             WgpuNative.subImage2D(
                 texId,
                 target,
                 level,
                 offsetX,
                 offsetY,
                 width,
                 height,
                 format,
                 type,
                 pixel_array,
                 unpack_row_length,
                 unpack_skip_pixels,
                 unpack_skip_rows,
                 unpack_alignment
             );
        } else {
            throw new RuntimeException("Attempted to map a texture that was too large onto a smaller texture");
        }
    }

    @Overwrite(remap = false)
    public static void _getTexImage(int target, int level, int format, int type, long pixels) {
    }

    @Overwrite(remap = false)
    public static void _viewport(int x, int y, int width, int height) {
//        GlStateManager.Viewport.INSTANCE.width = width;
    }

    @Overwrite(remap = false)
    public static void _colorMask(boolean red, boolean green, boolean blue, boolean alpha) {


    }

    @Overwrite(remap = false)
    public static void _stencilFunc(int func, int ref, int mask) {


    }

    @Overwrite(remap = false)
    public static void _stencilMask(int mask) {


    }

    @Overwrite(remap = false)
    public static void _stencilOp(int sfail, int dpfail, int dppass) {


    }

    @Overwrite(remap = false)
    public static void _clearDepth(double depth) {
    }

    @Overwrite(remap = false)
    public static void _clearColor(float red, float green, float blue, float alpha) {
        WgpuNative.clearColor(red, green, blue);
    }

    @Overwrite(remap = false)
    public static void _clearStencil(int stencil) {
    }

    @Overwrite(remap = false)
    public static void _clear(int mask, boolean getError) {


    }

    @Overwrite(remap = false)
    public static void _glDrawPixels(int width, int height, int format, int type, long pixels) {
    }

    @Overwrite(remap = false)
    public static void _vertexAttribPointer(int index, int size, int type, boolean normalized, int stride, long pointer) {
    }

    @Overwrite(remap = false)
    public static void _vertexAttribIPointer(int index, int size, int type, int stride, long pointer) {
    }

    @Overwrite(remap = false)
    public static void _enableVertexAttribArray(int index) {
    }

    @Overwrite(remap = false)
    public static void _disableVertexAttribArray(int index) {
    }

    @Overwrite(remap = false)
    public static void _drawElements(int mode, int first, int type, long indices) {
    }

    @Overwrite(remap = false)
    public static void _pixelStore(int pname, int param) {
        GlWmState.pixelStore.put(pname, param);
    }

    @Overwrite(remap = false)
    public static void _readPixels(int x, int y, int width, int height, int format, int type, ByteBuffer pixels) {
    }

    @Overwrite(remap = false)
    public static void _readPixels(int x, int y, int width, int height, int format, int type, long pixels) {
    }

    @Overwrite(remap = false)
    public static int _getError() {
        return 0;
    }

    @Overwrite(remap = false)
    public static String _getString(int name) {
        return "get string stub";
    }

    @Overwrite(remap = false)
    public static int _getInteger(int pname) {
        return -1;
    }




}
