package dev.birb.wgpu.mixin.core;

import com.mojang.blaze3d.platform.FramebufferInfo;
import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.client.util.math.Vector3f;
import net.minecraft.client.util.math.Vector4f;
import net.minecraft.util.math.Matrix4f;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.opengl.*;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.ByteBuffer;
import java.nio.FloatBuffer;
import java.nio.IntBuffer;

@SuppressWarnings("OverwriteAuthorRequired")
@Mixin(GlStateManager.class)
public class GlStateManagerMixin {

    @Deprecated
    @Overwrite
    public static void pushTextureAttributes() {
    }

    @Deprecated
    @Overwrite
    public static void popAttributes() {
    }

    @Deprecated
    @Overwrite
    public static void disableAlphaTest() {
    }

    @Deprecated
    @Overwrite
    public static void enableAlphaTest() {
    }

    @Deprecated
    @Overwrite
    public static void alphaFunc(int func, float ref) {

    }

    @Deprecated
    @Overwrite
    public static void enableLighting() {
    }

    @Deprecated
    @Overwrite
    public static void disableLighting() {
    }

    @Deprecated
    @Overwrite
    public static void enableLight(int light) {
    }

    @Deprecated
    @Overwrite
    public static void enableColorMaterial() {
    }

    @Deprecated
    @Overwrite
    public static void disableColorMaterial() {
    }

    @Deprecated
    @Overwrite
    public static void colorMaterial(int face, int mode) {

    }

    @Deprecated
    @Overwrite
    public static void light(int light, int pname, FloatBuffer params) {
    }

    @Deprecated
    @Overwrite
    public static void lightModel(int pname, FloatBuffer params) {
    }

    @Deprecated
    @Overwrite
    public static void normal3f(float nx, float ny, float nz) {
    }

    @Overwrite
    public static void method_31318() {
    }

    @Overwrite
    public static void method_31319() {
    }

    @Overwrite
    public static void method_31317(int i, int j, int k, int l) {
    }

    @Overwrite
    public static void disableDepthTest() {
    }

    @Overwrite
    public static void enableDepthTest() {
    }

    @Overwrite
    public static void depthFunc(int func) {

    }

    @Overwrite
    public static void depthMask(boolean mask) {

    }

    @Overwrite
    public static void disableBlend() {
    }

    @Overwrite
    public static void enableBlend() {
    }

    @Overwrite
    public static void blendFunc(int srcFactor, int dstFactor) {

    }

    @Overwrite
    public static void blendFuncSeparate(int srcFactorRGB, int dstFactorRGB, int srcFactorAlpha, int dstFactorAlpha) {

    }

    @Overwrite
    public static void blendColor(float red, float green, float blue, float alpha) {
    }

    @Overwrite
    public static void blendEquation(int mode) {
    }

    /**
     * Configures the frame buffer and populates {@link FramebufferInfo} with the appropriate constants
     * for the current GLCapabilities.
     *
     * @return human-readable string representing the selected frame buffer technology
     * @throws IllegalStateException if no known frame buffer technology is supported
     */
    @Overwrite
    public static String initFramebufferSupport(GLCapabilities capabilities) {
        return "";
    }

    @Overwrite
    public static int getProgram(int program, int pname) {
        //TODO
        return 0;
    }

    @Overwrite
    public static void attachShader(int program, int shader) {
    }

    @Overwrite
    public static void deleteShader(int shader) {
    }

    @Overwrite
    public static int createShader(int type) {
        return 0;
    }

    @Overwrite
    public static void shaderSource(int shader, CharSequence source) {
    }

    @Overwrite
    public static void compileShader(int shader) {
    }

    @Overwrite
    public static int getShader(int shader, int pname) {
        //TODO
        return 0;
    }

    @Overwrite
    public static void useProgram(int program) {
    }

    @Overwrite
    public static int createProgram() {
        //TODO
        return 0;
    }

    @Overwrite
    public static void deleteProgram(int program) {
    }

    @Overwrite
    public static void linkProgram(int program) {
    }

    @Overwrite
    public static int getUniformLocation(int program, CharSequence name) {
        //TODO
        return 0;
    }

    @Overwrite
    public static void uniform1(int location, IntBuffer value) {
    }

    @Overwrite
    public static void uniform1(int location, int value) {
    }

    @Overwrite
    public static void uniform1(int location, FloatBuffer value) {
    }

    @Overwrite
    public static void uniform2(int location, IntBuffer value) {
    }

    @Overwrite
    public static void uniform2(int location, FloatBuffer value) {
    }

    @Overwrite
    public static void uniform3(int location, IntBuffer value) {
    }

    @Overwrite
    public static void uniform3(int location, FloatBuffer value) {
    }

    @Overwrite
    public static void uniform4(int location, IntBuffer value) {
    }

    @Overwrite
    public static void uniform4(int location, FloatBuffer value) {
    }

    @Overwrite
    public static void uniformMatrix2(int location, boolean transpose, FloatBuffer value) {
    }

    @Overwrite
    public static void uniformMatrix3(int location, boolean transpose, FloatBuffer value) {
    }

    @Overwrite
    public static void uniformMatrix4(int location, boolean transpose, FloatBuffer value) {
    }

    @Overwrite
    public static int getAttribLocation(int program, CharSequence name) {
        //TODO
        return 0;
    }

    @Overwrite
    public static int genBuffers() {
        return WgpuNative.genBuffer();
    }

    @Overwrite
    public static void bindBuffers(int target, int buffer) {
        WgpuNative.bindBuffer(target, buffer);
    }

    @Overwrite
    public static void bufferData(int target, ByteBuffer data, int usage) {
//        WgpuNative.uploadBufferData(target, data, usage);
    }

    @Overwrite
    public static void deleteBuffers(int buffer) {
        WgpuNative.deleteBuffer(buffer);
    }

    @Overwrite
    public static void copyTexSubImage2d(int i, int j, int k, int l, int m, int n, int o, int p) {
    }

    @Overwrite
    public static void bindFramebuffer(int target, int framebuffer) {
        //TODO
    }

    @Overwrite
    public static int getFramebufferDepthAttachment() {
        //TODO
        return 0;
    }

    @Overwrite
    public static void blitFramebuffer(int i, int j, int k, int l, int m, int n, int o, int p, int q, int r) {

    }

    @Overwrite
    public static void deleteFramebuffers(int framebuffers) {

    }

    @Overwrite
    public static int genFramebuffers() {
        //TODO
        return 0;
    }

    @Overwrite
    public static int checkFramebufferStatus(int target) {
        //TODO
        return 0;
    }

    @Overwrite
    public static void framebufferTexture2D(int target, int attachment, int textureTarget, int texture, int level) {

    }

    @Deprecated
    @Overwrite
    public static int getActiveBoundTexture() {
        //TODO
        return 0;
    }

    @Overwrite
    public static void activeTextureUntracked(int texture) {
    }

    @Deprecated
    @Overwrite
    public static void clientActiveTexture(int texture) {
    }

    @Deprecated
    @Overwrite
    public static void multiTexCoords2f(int texture, float s, float t) {
    }

    @Overwrite
    public static void blendFuncSeparateUntracked(int srcFactorRGB, int dstFactorRGB, int srcFactorAlpha, int dstFactorAlpha) {
    }

    @Overwrite
    public static String getShaderInfoLog(int shader, int maxLength) {
        //TODO
        return "getShaderInfoLog Stub";
    }

    @Overwrite
    public static String getProgramInfoLog(int program, int maxLength) {
        //TODO
        return "getProgramInfoLog Stub";
    }

    @Overwrite
    public static void setupOutline() {
    }

    @Overwrite
    public static void teardownOutline() {
    }

    @Overwrite
    public static void setupOverlayColor(int texture, int size) {
    }

    @Overwrite
    public static void teardownOverlayColor() {
    }

    private static void combineColor(int combineColor, int source0Color) {
    }

    private static void combineColor(int combineColor, int source0Color, int source1Color, int source2Color) {
    }

    private static void combineAlpha(int combineAlpha, int source0Alpha) {
    }

    @Overwrite
    public static void setupLevelDiffuseLighting(Vector3f vector3f, Vector3f vector3f2, Matrix4f matrix4f) {
    }

    @Overwrite
    public static void setupGuiFlatDiffuseLighting(Vector3f vector3f, Vector3f vector3f2) {
    }

    @Overwrite
    public static void setupGui3dDiffuseLighting(Vector3f vector3f, Vector3f vector3f2) {
    }

    private static FloatBuffer getBuffer(float a, float b, float c, float d) {
        //TODO
        return null;
    }

    @Overwrite
    public static void setupEndPortalTexGen() {
    }

    @Overwrite
    public static void clearTexGen() {
    }

    @Overwrite
    public static void mulTextureByProjModelView() {
    }

    @Deprecated
    @Overwrite
    public static void enableFog() {
    }

    @Deprecated
    @Overwrite
    public static void disableFog() {
    }

    @Deprecated
    @Overwrite
    public static void fogMode(int mode) {

    }

    @Deprecated
    @Overwrite
    public static void fogDensity(float density) {

    }

    @Deprecated
    @Overwrite
    public static void fogStart(float start) {

    }

    @Deprecated
    @Overwrite
    public static void fogEnd(float end) {

    }

    @Deprecated
    @Overwrite
    public static void fog(int pname, float[] params) {
    }

    @Deprecated
    @Overwrite
    public static void fogi(int pname, int param) {
    }

    @Overwrite
    public static void enableCull() {
    }

    @Overwrite
    public static void disableCull() {
    }

    @Overwrite
    public static void polygonMode(int face, int mode) {
    }

    @Overwrite
    public static void enablePolygonOffset() {
    }

    @Overwrite
    public static void disablePolygonOffset() {
    }

    @Overwrite
    public static void enableLineOffset() {
    }

    @Overwrite
    public static void disableLineOffset() {
    }

    @Overwrite
    public static void polygonOffset(float factor, float units) {

    }

    @Overwrite
    public static void enableColorLogicOp() {
    }

    @Overwrite
    public static void disableColorLogicOp() {
    }

    @Overwrite
    public static void logicOp(int op) {

    }

    @Deprecated
    @Overwrite
    public static void enableTexGen(GlStateManager.TexCoord coord) {
    }

    @Deprecated
    @Overwrite
    public static void disableTexGen(GlStateManager.TexCoord coord) {
    }

    @Deprecated
    @Overwrite
    public static void texGenMode(GlStateManager.TexCoord coord, int mode) {

    }

    @Deprecated
    @Overwrite
    public static void texGenParam(GlStateManager.TexCoord coord, int pname, FloatBuffer params) {
    }

    @Deprecated
    @Overwrite
    private static GlStateManager.TexGenCoordState getGenCoordState(GlStateManager.TexCoord coord) {
        return null;
    }

    @Overwrite
    public static void activeTexture(int texture) {
        WgpuNative.activeTexture(texture);
    }

    @Overwrite
    public static void enableTexture() {
    }

    @Overwrite
    public static void disableTexture() {
    }

    @Deprecated
    @Overwrite
    public static void texEnv(int target, int pname, int param) {
    }

    @Overwrite
    public static void texParameter(int target, int pname, float param) {
    }

    @Overwrite
    public static void texParameter(int target, int pname, int param) {
    }

    @Overwrite
    public static int getTexLevelParameter(int target, int level, int pname) {
        //TODO
        return 0;
    }

    @Overwrite
    public static int genTextures() {
        return WgpuNative.genTexture();
    }

    @Overwrite
    public static void method_30498(int[] is) {
    }

    @Overwrite
    public static void deleteTexture(int texture) {

    }

    @Overwrite
    public static void method_30499(int[] is) {
    }

    @Overwrite
    public static void bindTexture(int texture) {
        WgpuNative.bindTexture(texture);
    }

    @Overwrite
    public static void texImage2D(int target, int level, int internalFormat, int width, int height, int border, int format, int type, @Nullable IntBuffer pixels) {
        long pointer;
        if(pixels != null) {
             pointer = MemoryUtil.memAddress(pixels);
        } else {
            pointer = 0;
        }
        WgpuNative.texImage2D(target, level, internalFormat, width, height, border, format, type, pointer);
    }

    @Overwrite
    public static void texSubImage2D(int target, int level, int offsetX, int offsetY, int width, int height, int format, int type, long pixels) {
    }

    @Overwrite
    public static void getTexImage(int target, int level, int format, int type, long pixels) {
    }

    @Deprecated
    @Overwrite
    public static void shadeModel(int mode) {

    }

    @Deprecated
    @Overwrite
    public static void enableRescaleNormal() {
    }

    @Deprecated
    @Overwrite
    public static void disableRescaleNormal() {
    }

    @Overwrite
    public static void viewport(int x, int y, int width, int height) {
    }

    @Overwrite
    public static void colorMask(boolean red, boolean green, boolean blue, boolean alpha) {

    }

    @Overwrite
    public static void stencilFunc(int func, int ref, int mask) {

    }

    @Overwrite
    public static void stencilMask(int mask) {

    }

    @Overwrite
    public static void stencilOp(int sfail, int dpfail, int dppass) {

    }

    @Overwrite
    public static void clearDepth(double depth) {
    }

    @Overwrite
    public static void clearColor(float red, float green, float blue, float alpha) {
    }

    @Overwrite
    public static void clearStencil(int stencil) {
    }

    @Overwrite
    public static void clear(int mask, boolean getError) {

    }

    @Deprecated
    @Overwrite
    public static void matrixMode(int mode) {
    }

    @Deprecated
    @Overwrite
    public static void loadIdentity() {
    }

    @Deprecated
    @Overwrite
    public static void pushMatrix() {
        WgpuNative.pushMatrix();
    }

    @Deprecated
    @Overwrite
    public static void popMatrix() {
        WgpuNative.popMatrix();
    }

    @Deprecated
    @Overwrite
    public static void getFloat(int pname, FloatBuffer params) {
    }

    @Deprecated
    @Overwrite
    public static void ortho(double l, double r, double b, double t, double n, double f) {
//        WgpuNative.ortho(l, r, b, t, n, f);
    }

    @Deprecated
    @Overwrite
    public static void rotatef(float angle, float x, float y, float z) {
//        WgpuNative.rotatef(angle, x, y, z);
    }

    @Deprecated
    @Overwrite
    public static void scalef(float x, float y, float z) {
//        WgpuNative.scalef(x, y, z);
    }

    @Deprecated
    @Overwrite
    public static void scaled(double x, double y, double z) {
        //TODO
        scalef((float) x, (float) y, (float) z);
    }

    @Deprecated
    @Overwrite
    public static void translatef(float x, float y, float z) {
//        WgpuNative.translatef(x, y, z);
    }

    @Deprecated
    @Overwrite
    public static void translated(double x, double y, double z) {
        translatef((float) x, (float) y, (float) z);
    }

    @Deprecated
    @Overwrite
    public static void multMatrix(FloatBuffer matrix) {
    }

    @Deprecated
    @Overwrite
    public static void multMatrix(Matrix4f matrix) {
        //TODO
//        WgpuNative.matrix(matrix);
    }

    @Deprecated
    @Overwrite
    public static void color4f(float red, float green, float blue, float alpha) {

    }

    @Deprecated
    @Overwrite
    public static void clearCurrentColor() {
    }

    @Deprecated
    @Overwrite
    public static void normalPointer(int type, int stride, long pointer) {
    }

    @Deprecated
    @Overwrite
    public static void texCoordPointer(int size, int type, int stride, long pointer) {
        WgpuNative.texCoordPointer(size, type, stride, pointer);
    }

    @Deprecated
    @Overwrite
    public static void vertexPointer(int size, int type, int stride, long pointer) {
        WgpuNative.vertexPointer(size, type, stride, pointer);
    }

    @Deprecated
    @Overwrite
    public static void colorPointer(int size, int type, int stride, long pointer) {
        WgpuNative.colorPointer(size, type, stride, pointer);
    }

    @Overwrite
    public static void vertexAttribPointer(int index, int size, int type, boolean normalized, int stride, long pointer) {
    }

    @Deprecated
    @Overwrite
    public static void enableClientState(int cap) {
        WgpuNative.enableClientState(cap);
    }

    @Deprecated
    @Overwrite
    public static void disableClientState(int cap) {
        WgpuNative.disableClientState(cap);
    }

    @Overwrite
    public static void enableVertexAttribArray(int index) {

    }

    @Overwrite
    public static void method_22607(int index) {
    }

    @Overwrite
    public static void drawArrays(int mode, int first, int count) {
        WgpuNative.drawArray(mode, first, count);
    }

    @Overwrite
    public static void lineWidth(float width) {
    }

    @Overwrite
    public static void pixelStore(int pname, int param) {
    }

    @Overwrite
    public static void pixelTransfer(int pname, float param) {
    }

    @Overwrite
    public static void readPixels(int x, int y, int width, int height, int format, int type, ByteBuffer pixels) {
    }

    @Overwrite
    public static int getError() {
        //TODO
        return 0;
    }

    @Overwrite
    public static String getString(int name) {
        //TODO
        return "getString Stub";
    }

    @Overwrite
    public static int getInteger(int pname) {
        //TODO
        return 0;
    }

    //I *assume* whatever GL 3.0 would be doing has a wgpu equivalent
    @Overwrite
    public static boolean supportsGl30() {
        return true;
    }

}
