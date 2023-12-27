package dev.birb.wgpu.mixin.core;

import com.mojang.blaze3d.platform.GlStateManager;
import dev.birb.wgpu.WgpuMcMod;
import dev.birb.wgpu.render.GlWmState;
import dev.birb.wgpu.render.Wgpu;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.Nullable;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.lwjgl.opengl.GL11;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.ByteBuffer;
import java.nio.FloatBuffer;
import java.nio.IntBuffer;
import java.util.List;
import java.util.function.Consumer;

import static dev.birb.wgpu.WgpuMcMod.LOGGER;

@Mixin(GlStateManager.class)
public class GlStateManagerMixin {
    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _disableScissorTest() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _enableScissorTest() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _scissorBox(int x, int y, int width, int height) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _disableDepthTest() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _enableDepthTest() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _depthFunc(int func) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _depthMask(boolean mask) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _disableBlend() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _enableBlend() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _blendFunc(int srcFactor, int dstFactor) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _blendFuncSeparate(int srcFactorRGB, int dstFactorRGB, int srcFactorAlpha, int dstFactorAlpha) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _blendEquation(int mode) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int glGetProgrami(int program, int pname) {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void glAttachShader(int program, int shader) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void glDeleteShader(int shader) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int glCreateShader(int type) {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void glShaderSource(int shader, List<String> strings) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void glCompileShader(int shader) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int glGetShaderi(int shader, int pname) {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUseProgram(int program) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int glCreateProgram() {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void glDeleteProgram(int program) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void glLinkProgram(int program) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int _glGetUniformLocation(int program, CharSequence name) {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniform1(int location, IntBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniform1i(int location, int value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniform1(int location, FloatBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniform2(int location, IntBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniform2(int location, FloatBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniform3(int location, IntBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniform3(int location, FloatBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniform4(int location, IntBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniform4(int location, FloatBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniformMatrix2(int location, boolean transpose, FloatBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniformMatrix3(int location, boolean transpose, FloatBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUniformMatrix4(int location, boolean transpose, FloatBuffer value) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int _glGetAttribLocation(int program, CharSequence name) {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glBindAttribLocation(int program, int index, CharSequence name) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int _glGenBuffers() {
        return -1;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int _glGenVertexArrays() {
//        return WgpuNative.gen
        return 1;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glBindBuffer(int target, int buffer) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glBindVertexArray(int array) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glBufferData(int target, ByteBuffer data, int usage) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glBufferData(int target, long size, int usage) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @SuppressWarnings("DataFlowIssue")
    @Overwrite(remap = false)
    @Nullable
    public static ByteBuffer mapBuffer(int target, int access) {
        return ByteBuffer.allocate(0);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glUnmapBuffer(int target) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glDeleteBuffers(int buffer) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glCopyTexSubImage2D(int target, int level, int xOffset, int yOffset, int x, int y, int width, int height) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glDeleteVertexArrays(int array) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glBindFramebuffer(int target, int framebuffer) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glBlitFrameBuffer(int srcX0, int srcY0, int srcX1, int srcY1, int dstX0, int dstY0, int dstX1, int dstY1, int mask, int filter) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glBindRenderbuffer(int target, int renderbuffer) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glDeleteRenderbuffers(int renderbuffer) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glDeleteFramebuffers(int framebuffer) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int glGenFramebuffers() {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int glGenRenderbuffers() {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glRenderbufferStorage(int target, int internalFormat, int width, int height) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glFramebufferRenderbuffer(int target, int attachment, int renderbufferTarget, int renderbuffer) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int glCheckFramebufferStatus(int target) {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glFramebufferTexture2D(int target, int attachment, int textureTarget, int texture, int level) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int getBoundFramebuffer() {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void glActiveTexture(int texture) {
        GlWmState.setActiveTexture(texture);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void glBlendFuncSeparate(int srcFactorRGB, int dstFactorRGB, int srcFactorAlpha, int dstFactorAlpha) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static String glGetShaderInfoLog(int shader, int maxLength) {
        return "Shader info log stub";
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static String glGetProgramInfoLog(int program, int maxLength) {
        return "Program info log stub";
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void setupLevelDiffuseLighting(Vector3f vec3f, Vector3f vec3f2, Matrix4f matrix4f) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void setupGuiFlatDiffuseLighting(Vector3f vec3f, Vector3f vec3f2) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void setupGui3DDiffuseLighting(Vector3f vec3f, Vector3f vec3f2) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _enableCull() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _disableCull() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _polygonMode(int face, int mode) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _enablePolygonOffset() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _disablePolygonOffset() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _polygonOffset(float factor, float units) {


    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _enableColorLogicOp() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _disableColorLogicOp() {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _logicOp(int op) {


    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _activeTexture(int texture) {


    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _texParameter(int target, int pname, float param) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _texParameter(int target, int pname, int param) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int _getTexLevelParameter(int target, int level, int pname) {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int _genTexture() {
        GlWmState.getGeneratedTextures().add(new GlWmState.WmTexture());
        return GlWmState.getGeneratedTextures().size() - 1;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _genTextures(int[] textures) {
        for (int i = 0; i < textures.length; i++) {
            textures[i] = _genTexture();
        }
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _deleteTexture(int texture) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _deleteTextures(int[] textures) {

    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _bindTexture(int texture) {
        GlWmState.getTextureSlots().put(GlWmState.getActiveTexture(), texture);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int _getActiveTexture() {
        return GlWmState.getActiveTexture();
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _texImage2D(int target, int level, int internalFormat, int width, int height, int border, int format, int type, @Nullable IntBuffer pixels) {
        if (level != 0) return;

        int texId = GlWmState.getTextureSlots().get(GlWmState.getActiveTexture());
        GlWmState.WmTexture texture = GlWmState.getGeneratedTextures().get(texId);

        if(width < texture.getWidth() || height < texture.getHeight()) {
            WgpuMcMod.LOGGER.debug("_texImage2D tried to make a texture smaller?");
            return;
        }

        texture.setWidth(width);
        texture.setHeight(height);

        long ptr = 0;
        if (pixels != null) {
            ptr = MemoryUtil.memAddress(pixels);
        }
        WgpuNative.texImage2D(texId, target, level, internalFormat, width, height, border, format, type, ptr);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _texSubImage2D(int target, int level, int offsetX, int offsetY, int width, int height, int format, int type, long pixels) {
        //we do not care about mip maps
        if (level != 0) return;

        if (format != GL11.GL_RGBA && format != 0x80E1) return;

        int texId = GlWmState.getTextureSlots().get(GlWmState.getActiveTexture());
        GlWmState.WmTexture texture = GlWmState.getGeneratedTextures().get(texId);

        int unpackRowLength = GlWmState.getPixelStore().getOrDefault(GL11.GL_UNPACK_ROW_LENGTH, 0);
        int unpackSkipPixels = GlWmState.getPixelStore().getOrDefault(GL11.GL_UNPACK_SKIP_PIXELS, 0);
        int unpackSkipRows = GlWmState.getPixelStore().getOrDefault(GL11.GL_UNPACK_SKIP_ROWS, 0);
        int unpackAlignment = GlWmState.getPixelStore().getOrDefault(GL11.GL_UNPACK_ALIGNMENT, 4);

        if (width + offsetX <= texture.getWidth() && height + offsetY <= texture.getHeight()) {
            int[] pixelArray = new int[width * height];
            long pixelSize = 4L; //TODO support more formats..?
            for (int y = 0; y < height; y++) {
                for (int x = 0; x < width; x++) {
                    int currentX = x + unpackSkipPixels;
                    int currentY = (y + unpackSkipRows) * (unpackRowLength > 0 ? unpackRowLength : width);

//<<<<<<< HEAD
                    //TODO row_byte_offset proper impl || let row_byte_offset = if pixel_size >= unpack_alignment
                    long offset = (currentX + currentY) * pixelSize;
                    pixelArray[x + y * width] = MemoryUtil.memGetInt(pixels + offset);
//=======
//            long pixel_size = 4L; //TODO support more formats..?
//            for (int y = 0; y < height; y++) {
//                for (int x = 0; x < width; x++) {
//                    int current_x = x + unpack_skip_pixels;
//                    int current_y = (y + unpack_skip_rows) *
//                            (unpack_row_length > 0 ? unpack_row_length : width);
//
//                    //TODO row_byte_offset proper impl || let row_byte_offset = if pixel_size >= unpack_alignment
//                    long offset = (current_x + current_y) * pixel_size;
//
////                    intBuf.put(x + y * width, MemoryUtil.memGetInt(pixels+offset));
//                    pixel_array[x + y * width] = MemoryUtil.memGetInt(pixels + offset);
//>>>>>>> e61f847689629ff02bf2135cc266992744d3c54e
                }
            }

            Wgpu.incrementTexSubImageCount();
            WgpuNative.subImage2D(texId, target, level, offsetX, offsetY, width, height, format, type, pixelArray, unpackRowLength, unpackSkipPixels, unpackSkipRows, unpackAlignment);
        } else {
            throw new IllegalArgumentException("Attempted to map a texture that was too large onto a smaller texture");
        }
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _getTexImage(int target, int level, int format, int type, long pixels) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _viewport(int x, int y, int width, int height) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _colorMask(boolean red, boolean green, boolean blue, boolean alpha) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _stencilFunc(int func, int ref, int mask) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _stencilMask(int mask) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _stencilOp(int sfail, int dpfail, int dppass) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _clearDepth(double depth) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _clearColor(float red, float green, float blue, float alpha) {
        WgpuNative.clearColor(red, green, blue);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _clearStencil(int stencil) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _clear(int mask, boolean getError) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _glDrawPixels(int width, int height, int format, int type, long pixels) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _vertexAttribPointer(int index, int size, int type, boolean normalized, int stride, long pointer) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _vertexAttribIPointer(int index, int size, int type, int stride, long pointer) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _enableVertexAttribArray(int index) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _disableVertexAttribArray(int index) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _drawElements(int mode, int first, int type, long indices) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _pixelStore(int pname, int param) {
        GlWmState.getPixelStore().put(pname, param);
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _readPixels(int x, int y, int width, int height, int format, int type, ByteBuffer pixels) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static void _readPixels(int x, int y, int width, int height, int format, int type, long pixels) {
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int _getError() {
        return 0;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static String _getString(int name) {
        return "get string stub";
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static int _getInteger(int pname) {
        return -1;
    }

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite
    public static void upload(int level, int offsetX, int offsetY, int width, int height, NativeImage.Format format, IntBuffer pixels, Consumer<IntBuffer> closer) {

    }


}
