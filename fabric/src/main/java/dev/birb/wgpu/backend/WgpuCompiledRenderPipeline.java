package dev.birb.wgpu.backend;

import com.mojang.blaze3d.pipeline.CompiledRenderPipeline;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.shaders.ShaderSource;
import com.mojang.blaze3d.shaders.ShaderType;
import com.mojang.blaze3d.shaders.UniformType;
import dev.birb.wgpu.helper.GpuFormatHelper;
import dev.birb.wm.*;
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
import java.util.Objects;
import java.util.concurrent.ConcurrentHashMap;

public class WgpuCompiledRenderPipeline implements CompiledRenderPipeline {

    private static final ValueLayout normalizedTypeLayout = ValueLayout.JAVA_LONG.withName("NormalizedType");

    static final ConcurrentHashMap<RenderPipeline, WgpuCompiledRenderPipeline> wgpuRenderPipelines = new ConcurrentHashMap<>();
    static final ConcurrentHashMap<ShaderCompilationKey, String> shaderSourceCache = new ConcurrentHashMap<>();

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

    public WgpuCompiledRenderPipeline(WgpuDevice device, @NonNull RenderPipeline pipeline, ShaderSource shaderSource) {

        this.shaderSource = shaderSource;

        try(Arena arena = Arena.ofConfined()) {
            var vertexFormats = VertexFormat.allocateArray(pipeline.getVertexFormatBindings().length, arena);

            int actualVertexFormatBufferCount = 0;

            for(int v=0;v<pipeline.getVertexFormatBindings().length;v++) {
                var vertexFormat = pipeline.getVertexFormatBindings()[v];

                if(vertexFormat == null) {
                    actualVertexFormatBufferCount = v;
                    break;
                }

                var elements = Objects.requireNonNull(vertexFormat).getElements();

                var vertexFormatElements = VertexFormatElement.allocateArray(elements.size(), arena);

                for(long i=0;i<elements.size();i++) {
                    var formatElement = elements.get((int) i);

                    MemorySegment seg = VertexFormatElement.asSlice(vertexFormatElements, i);
                    VertexFormatElement.offset(seg, formatElement.offset());
                    VertexFormatElement.format(seg, GpuFormatHelper.gpuFormatToRustEnum(formatElement.format()));
                    VertexFormatElement.name(seg, arena.allocateFrom(formatElement.name()));
                }

                var vertexFormatSeg = VertexFormat.asSlice(vertexFormats, v);

                var raw_array = RawArray_VertexFormatElement.allocate(arena);
                RawArray_VertexFormatElement.size(raw_array, elements.size());
                RawArray_VertexFormatElement.contents(raw_array, vertexFormatElements);

                VertexFormat.elements(vertexFormatSeg, raw_array);
                VertexFormat.vertex_size(vertexFormatSeg, vertexFormat.getVertexSize());
            }

            var fragSource = getOrSourceShader(pipeline.getFragmentShader(), ShaderType.FRAGMENT, pipeline.getShaderDefines(), shaderSource);
            var vertSource = getOrSourceShader(pipeline.getVertexShader(), ShaderType.VERTEX, pipeline.getShaderDefines(), shaderSource);

            var bind_group_layouts_array = BlazeBindGroupLayout.allocateArray(pipeline.getBindGroupLayouts().size(), arena);

            var bind_group_layouts_raw_array = RawArray_BlazeBindGroupLayout.allocate(arena);
            RawArray_BlazeBindGroupLayout.contents(bind_group_layouts_raw_array, bind_group_layouts_array);
            RawArray_BlazeBindGroupLayout.size(bind_group_layouts_raw_array, pipeline.getBindGroupLayouts().size());

            for(int i=0;i<pipeline.getBindGroupLayouts().size();i++) {
                var layout = pipeline.getBindGroupLayouts().get(i);
                var entryCount = layout.getSamplers().size() + layout.getUniforms().size();

                var bind_group_descriptor_seg = BlazeBindGroupLayout.asSlice(bind_group_layouts_array, i);

                var bind_group_entries = BindGroupEntryDescriptor.allocateArray(entryCount, arena);
                var raw_array_entries = RawArray_BindGroupEntryDescriptor.allocate(arena);
                RawArray_BindGroupEntryDescriptor.size(raw_array_entries, entryCount);
                RawArray_BindGroupEntryDescriptor.contents(raw_array_entries, bind_group_entries);

                BlazeBindGroupLayout.entries(bind_group_descriptor_seg, raw_array_entries);

                for(var u=0;u<layout.getUniforms().size();u++) {
                    var uniform = layout.getUniforms().get(u);

                    var descriptor = BindGroupEntryDescriptor.asSlice(bind_group_entries, u);
                    BindGroupEntryDescriptor.name(descriptor, arena.allocateFrom(uniform.name()));
                    BindGroupEntryDescriptor.type_(descriptor, switch(uniform.type()) {
                        case TEXEL_BUFFER -> 0;
                        case UNIFORM_BUFFER -> 1;
                    });
                    if(uniform.type() == UniformType.TEXEL_BUFFER) {
                        BindGroupEntryDescriptor.texture_format(descriptor, GpuFormatHelper.gpuFormatToRustEnum(Objects.requireNonNull(uniform.gpuFormat())));
                    }
                }

                for(var s=0;s<layout.getSamplers().size();s++) {
                    var sampler = layout.getSamplers().get(s);

                    var descriptor = BindGroupEntryDescriptor.asSlice(bind_group_entries, s + layout.getUniforms().size());
                    BindGroupEntryDescriptor.name(descriptor, arena.allocateFrom(sampler));
                    BindGroupEntryDescriptor.type_(descriptor, 2);
                }
            }

            var vertexFormatsRawArray = RawArray_VertexFormat.allocate(arena);
            RawArray_VertexFormat.contents(vertexFormatsRawArray, vertexFormats);
            RawArray_VertexFormat.size(vertexFormatsRawArray, actualVertexFormatBufferCount);

            var defines_raw_array = RawArray_______________FfiStr__________2.allocate(arena);
            var defines = arena.allocate(MemoryLayout.sequenceLayout(2, ValueLayout.ADDRESS), pipeline.getShaderDefines().values().size());
            RawArray_______________FfiStr__________2.contents(defines_raw_array, defines);
            RawArray_______________FfiStr__________2.size(defines_raw_array, pipeline.getShaderDefines().values().size());

            var directives = pipeline.getShaderDefines().asSourceDirectives();

            var colorTargetCount = pipeline.getColorTargetStates().length;

            var rawArrayColorTargetState = RawArray_BlazeColorTargetState.allocate(arena);
            var colorTargetStates = BlazeColorTargetState.allocateArray(colorTargetCount, arena);

            RawArray_BlazeColorTargetState.size(rawArrayColorTargetState, colorTargetCount);
            RawArray_BlazeColorTargetState.contents(rawArrayColorTargetState, colorTargetStates);

            for(int i=0;i<colorTargetCount;i++) {
                var colorTargetState = pipeline.getColorTargetStates()[i];
                var slice = BlazeColorTargetState.asSlice(colorTargetStates, i);

            }

            var depthTargetState = MemorySegment.NULL;

            if(pipeline.getDepthStencilState() != null) {
//                depthTargetState = BlazeDepthStencilState.allocate(arena);

            }

            var renderPipelineStruct = dev.birb.wm.RenderPipeline.allocate(arena);
            dev.birb.wm.RenderPipeline.bind_group_layouts(renderPipelineStruct, bind_group_layouts_raw_array);
            dev.birb.wm.RenderPipeline.vertex_formats(renderPipelineStruct, vertexFormatsRawArray);
            dev.birb.wm.RenderPipeline.vertex_shader(renderPipelineStruct, arena.allocateFrom(vertSource));
            dev.birb.wm.RenderPipeline.fragment_shader(renderPipelineStruct, arena.allocateFrom(fragSource));
//            dev.birb.wm.RenderPipeline.defines(renderPipelineStruct, defines_raw_array);
            dev.birb.wm.RenderPipeline.directives(renderPipelineStruct, arena.allocateFrom(directives));
            dev.birb.wm.RenderPipeline.depth_stencil_state(renderPipelineStruct, depthTargetState);
            dev.birb.wm.RenderPipeline.color_target_states(renderPipelineStruct, rawArrayColorTargetState);
            dev.birb.wm.RenderPipeline.primitive_topology(renderPipelineStruct, switch(pipeline.getPrimitiveTopology()) {
                case LINES, DEBUG_LINES -> 1;
                case DEBUG_LINE_STRIP -> 2;
                case POINTS -> 3;
                case TRIANGLES -> 4;
                case TRIANGLE_STRIP -> 5;
                case TRIANGLE_FAN -> 6;
                case QUADS -> 7;
            });

            nativePipeline = WM.compile_render_pipeline(
                    device.getWm(),
                    renderPipelineStruct
            );
        }

    }

    @Override
    public boolean isValid() {
        return true;
    }

}
