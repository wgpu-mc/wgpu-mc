package dev.birb.wgpu.backend;

import com.mojang.blaze3d.pipeline.CompiledRenderPipeline;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.shaders.ShaderSource;
import com.mojang.blaze3d.shaders.ShaderType;
import dev.birb.wm.WM;
import it.unimi.dsi.fastutil.objects.Object2IntArrayMap;
import lombok.Getter;
import net.fabricmc.api.EnvType;
import net.fabricmc.api.Environment;
import net.minecraft.client.renderer.ShaderDefines;
import net.minecraft.resources.Identifier;
import org.jspecify.annotations.NonNull;

import java.lang.foreign.Arena;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.util.concurrent.ConcurrentHashMap;

public class WgpuCompiledRenderPipeline implements CompiledRenderPipeline {

    private static final ValueLayout normalizedTypeLayout = ValueLayout.JAVA_LONG.withName("NormalizedType");

    static final ConcurrentHashMap<RenderPipeline, WgpuCompiledRenderPipeline> wgpuRenderPipelines = new ConcurrentHashMap<>();
    static final ConcurrentHashMap<ShaderCompilationKey, String> shaderSourceCache = new ConcurrentHashMap<>();

    private static final MemoryLayout vertexFormatElementLayout = MemoryLayout.structLayout(
            ValueLayout.JAVA_LONG.withName("offset"),
            normalizedTypeLayout
    ).withName("VertexFormatElement {}");

    private static final MemoryLayout vertexFormatLayout = MemoryLayout.structLayout(
            ValueLayout.ADDRESS.withName("elements"),
            ValueLayout.JAVA_LONG.withName("elements_count"),
            ValueLayout.JAVA_LONG.withName("vertex_size"),
            ValueLayout.JAVA_LONG.withName("primitive")
    ).withName("VertexFormat {}");

    private static final MemoryLayout uniformDescriptionLayout = MemoryLayout.structLayout(
            ValueLayout.JAVA_LONG.withName("type_"),
            ValueLayout.ADDRESS.withName("name")
    );

    private static final MemoryLayout renderPipelineLayout = MemoryLayout.structLayout(
            ValueLayout.ADDRESS.withTargetLayout(uniformDescriptionLayout).withName("uniforms"),
            ValueLayout.JAVA_LONG.withName("uniforms_count"),
            ValueLayout.ADDRESS.withTargetLayout(vertexFormatLayout).withName("vertex_format"),
            ValueLayout.ADDRESS.withName("vert_shader"),
            ValueLayout.ADDRESS.withName("frag_shader"),
            ValueLayout.ADDRESS.withName("defines"),
            ValueLayout.JAVA_LONG.withName("defines_count"),
            ValueLayout.ADDRESS.withName("frag_state"),
            ValueLayout.JAVA_LONG.withName("depth")
    ).withName("RenderPipeline {}").withByteAlignment(8);

    @Environment(EnvType.CLIENT)
    private record ShaderCompilationKey(Identifier id, ShaderType type, ShaderDefines defines) {
        public String toString() {
            String string = this.id + " (" + this.type + ")";
            return !this.defines.isEmpty() ? string + " with " + this.defines : string;
        }
    }

    @Getter
    private final MemorySegment nativePipeline;

    @Getter
    private final ShaderSource shaderSource;

    private static String getOrSourceShader(Identifier location, ShaderType type, ShaderDefines defines, ShaderSource source) {
        return shaderSourceCache.computeIfAbsent(
                new ShaderCompilationKey(location, type, defines), r -> source.get(r.id, r.type)
        );
    }

    public WgpuCompiledRenderPipeline(@NonNull RenderPipeline pipeline, ShaderSource shaderSource) {

        this.shaderSource = shaderSource;

        try(Arena arena = Arena.ofConfined()) {
            var processedFragBuffer = MemorySegment.NULL;
            var processedVertBuffer = MemorySegment.NULL;

            var vertexFormat = pipeline.getVertexFormat();
            var elements = vertexFormat.getElements();

//            var vertexShape = new Object2IntArrayMap<String>();
//
//            for(int i=0;i<elements.size();i++) {
//                vertexShape.put(vertexFormat.getElementName(vertexFormat.getElements().get(i)), i);
//            }

            var fragSource = getOrSourceShader(pipeline.getFragmentShader(), ShaderType.FRAGMENT, pipeline.getShaderDefines(), shaderSource);
            var vertSource = getOrSourceShader(pipeline.getVertexShader(), ShaderType.VERTEX, pipeline.getShaderDefines(), shaderSource);

            var m = new Object2IntArrayMap<String>();

            for(int u=0; u<pipeline.getUniforms().size(); u++) {
                m.put(pipeline.getUniforms().get(u).name(), u);
            }

            int samplerOffset = m.size();

            for(int i=0;i<pipeline.getSamplers().size();i++) {
                var sampler = pipeline.getSamplers().get(i);
                m.put(sampler, samplerOffset + i);
            }

            var vertexFormatElements = arena.allocate(vertexFormatElementLayout, elements.size());

            for(long i=0;i<elements.size();i++) {
                var formatElement = elements.get((int) i);

                long t_ = switch(formatElement.id()) {
                    case 0 -> 1;
                    case 1 -> 8;
                    case 2 -> 4;
                    case 3, 4 -> 7;
                    case 5 -> 9;
                    case 6 -> 5;
                    default -> throw new IllegalStateException("Unexpected value: " + formatElement.id());
                };

                MemorySegment seg = vertexFormatElements.asSlice(vertexFormatElementLayout.byteSize() * i, vertexFormatElementLayout.byteSize());
                seg.set(ValueLayout.JAVA_LONG, 0, vertexFormat.getOffsetsByElement()[formatElement.id()]);
                seg.set(ValueLayout.JAVA_LONG, 8, t_);
            }

            var vertexFormatBuffer = arena.allocate(vertexFormatLayout);

            vertexFormatBuffer.set(ValueLayout.ADDRESS, 0, vertexFormatElements);
            vertexFormatBuffer.set(ValueLayout.JAVA_LONG,8, elements.size());
            vertexFormatBuffer.set(ValueLayout.JAVA_LONG, 16, vertexFormat.getVertexSize());
            vertexFormatBuffer.set(ValueLayout.JAVA_LONG, 24, switch(pipeline.getVertexFormatMode()) {
                case LINES -> 0;
                case DEBUG_LINES -> 1;
                case DEBUG_LINE_STRIP -> 2;
                case POINTS -> 3;
                case TRIANGLES -> 4;
                case TRIANGLE_STRIP -> 5;
                case TRIANGLE_FAN -> 6;
                case QUADS -> 7;
            });

            var uniformDescriptions = arena.allocate(MemoryLayout.sequenceLayout(pipeline.getUniforms().size() + pipeline.getSamplers().size(), uniformDescriptionLayout));

            for(int i=0;i<pipeline.getUniforms().size();i++) {
                var uniform = pipeline.getUniforms().get(i);

                int type = switch(uniform.type()) {
                    case TEXEL_BUFFER -> 0;
                    case UNIFORM_BUFFER -> 1;
                };

                var name = arena.allocateFrom(uniform.name());

                var uniformDescriptor = uniformDescriptions.asSlice(uniformDescriptionLayout.byteSize() * i, uniformDescriptionLayout.byteSize());
                uniformDescriptor.set(ValueLayout.JAVA_LONG, 0, type);
                uniformDescriptor.set(ValueLayout.ADDRESS, 8, name);
            }

            for(int i=0;i<pipeline.getSamplers().size();i++) {
                var sampler = pipeline.getSamplers().get(i);
                var name = arena.allocateFrom(sampler);

                var uniformDescriptor = uniformDescriptions.asSlice(uniformDescriptionLayout.byteSize() * (i + pipeline.getUniforms().size()), uniformDescriptionLayout.byteSize());

                uniformDescriptor.set(ValueLayout.JAVA_LONG, 0, 2);
                uniformDescriptor.set(ValueLayout.ADDRESS, 8, name);
            }

            var defines = pipeline.getShaderDefines();

            var definesBuffer = arena.allocate(MemoryLayout.sequenceLayout(defines.values().size(), MemoryLayout.structLayout(ValueLayout.ADDRESS, ValueLayout.ADDRESS)));
            var definesEntries = defines.values().entrySet().iterator();

            for(int i=0;i<defines.values().size();i++) {
                var define = definesEntries.next();
                MemorySegment definePair = definesBuffer.asSlice(16L * i, 16);
                definePair.set(ValueLayout.ADDRESS, 0, arena.allocateFrom(define.getKey()));
                definePair.set(ValueLayout.ADDRESS, 8, arena.allocateFrom(define.getValue()));
            }

            var fragStateBuffer = arena.allocate(8, 8);

            var renderPipelineStruct = arena.allocate(renderPipelineLayout);

            renderPipelineStruct.set(ValueLayout.ADDRESS, 0, uniformDescriptions);
            renderPipelineStruct.set(ValueLayout.JAVA_LONG, 8, pipeline.getUniforms().size() + pipeline.getSamplers().size());
            renderPipelineStruct.set(ValueLayout.ADDRESS, 8 * 2, vertexFormatBuffer);
            renderPipelineStruct.set(ValueLayout.ADDRESS, 8 * 3, arena.allocateFrom(vertSource));
            renderPipelineStruct.set(ValueLayout.ADDRESS, 8 * 4, arena.allocateFrom(fragSource));
            renderPipelineStruct.set(ValueLayout.ADDRESS, 8 * 5, definesBuffer);
            renderPipelineStruct.set(ValueLayout.JAVA_LONG, 8 * 6, defines.values().size());
            renderPipelineStruct.set(ValueLayout.ADDRESS, 8 * 7, fragStateBuffer);
            renderPipelineStruct.set(ValueLayout.JAVA_LONG, 8 * 8, pipeline.wantsDepthTexture() ? 1 : 0);

            nativePipeline = WM.compile_render_pipeline(renderPipelineStruct);
        }

    }

    @Override
    public boolean isValid() {
        return true;
    }

}
