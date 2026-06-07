package dev.birb.wgpu.backend;

import com.mojang.blaze3d.pipeline.CompiledRenderPipeline;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.shaders.ShaderSource;
import com.mojang.blaze3d.shaders.ShaderType;
import com.mojang.blaze3d.shaders.UniformType;
import dev.birb.wgpu.helper.GpuFormatHelper;
import dev.birb.wm.*;
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
import java.util.Objects;
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
            ValueLayout.JAVA_LONG.withName("vertex_size")
    ).withName("VertexFormat {}");

    private static final MemoryLayout uniformDescriptionLayout = MemoryLayout.structLayout(
            ValueLayout.JAVA_LONG.withName("type_"),
            ValueLayout.ADDRESS.withName("name"),
            ValueLayout.JAVA_LONG.withName("format")
    );

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

            var vertexFormats = VertexFormat.allocateArray(pipeline.getVertexFormatBindings().length, arena);

            for(int v=0;v<pipeline.getVertexFormatBindings().length;v++) {
                var vertexFormat = pipeline.getVertexFormatBindings()[v];

                var elements = Objects.requireNonNull(vertexFormat).getElements();

                var vertexFormatElements = arena.allocate(vertexFormatElementLayout, elements.size());

                for(long i=0;i<elements.size();i++) {
                    var formatElement = elements.get((int) i);

                    MemorySegment seg = vertexFormatElements.asSlice(vertexFormatElementLayout.byteSize() * i, vertexFormatElementLayout.byteSize());
                    seg.set(ValueLayout.JAVA_LONG, 0, formatElement.offset());
                    seg.set(ValueLayout.JAVA_LONG, 8, GpuFormatHelper.gpuFormatToRustEnum(formatElement.format()));
                }

                var vertexFormatBuffer = arena.allocate(vertexFormatLayout);

                vertexFormatBuffer.set(ValueLayout.ADDRESS, 0, vertexFormatElements);
                vertexFormatBuffer.set(ValueLayout.JAVA_LONG,8, elements.size());
                vertexFormatBuffer.set(ValueLayout.JAVA_LONG, 16, vertexFormat.getVertexSize());
            }


//            var vertexShape = new Object2IntArrayMap<String>();
//
//            for(int i=0;i<elements.size();i++) {
//                vertexShape.put(vertexFormat.getElementName(vertexFormat.getElements().get(i)), i);
//            }

            var fragSource = getOrSourceShader(pipeline.getFragmentShader(), ShaderType.FRAGMENT, pipeline.getShaderDefines(), shaderSource);
            var vertSource = getOrSourceShader(pipeline.getVertexShader(), ShaderType.VERTEX, pipeline.getShaderDefines(), shaderSource);

            var m = new Object2IntArrayMap<String>();

            var bind_group_descriptors = arena.allocate(BlazeBindGroupLayout.layout(), pipeline.getBindGroupLayouts().size());

            var raw_array_bind_group_descriptors = RawArray_BlazeBindGroupLayout.allocate(arena);
            RawArray_BlazeBindGroupLayout.contents(raw_array_bind_group_descriptors, bind_group_descriptors);
            RawArray_BlazeBindGroupLayout.size(raw_array_bind_group_descriptors, pipeline.getBindGroupLayouts().size());

            for(int i=0;i<pipeline.getBindGroupLayouts().size();i++) {
                var layout = pipeline.getBindGroupLayouts().get(i);
                var entryCount = layout.getSamplers().size() + layout.getUniforms().size();

                var bind_group_descriptor_seg = BlazeBindGroupLayout.asSlice(bind_group_descriptors, i);

                var bind_group_entries = BindGroupEntryDescriptor.allocateArray(entryCount, arena);
                var raw_array_entries = RawArray_BindGroupEntryDescriptor.allocate(arena);
                RawArray_BindGroupEntryDescriptor.size(raw_array_entries, entryCount);
                RawArray_BindGroupEntryDescriptor.contents(raw_array_entries, bind_group_entries);

                BlazeBindGroupLayout.entries(bind_group_descriptor_seg, raw_array_entries);

                for(var u=0;u<layout.getUniforms().size();u++) {
                    var uniform = layout.getUniforms().get(u);

                    var descriptor = BindGroupEntryDescriptor.asSlice(bind_group_entries, u);
                    BindGroupEntryDescriptor.type_(descriptor, switch(uniform.type()) {
                        case TEXEL_BUFFER -> 0;
                        case UNIFORM_BUFFER -> 1;
                    });
                    if(uniform.type() == UniformType.TEXEL_BUFFER) {
                        BindGroupEntryDescriptor.texture_format(descriptor, GpuFormatHelper.gpuFormatToRustEnum(Objects.requireNonNull(uniform.gpuFormat())));
                    }
                }
            }

            var vertexFormatsRawArray = RawArray_VertexFormat.allocateArray(pipeline.getVertexFormatBindings().length, arena);
            RawArray_VertexFormat.contents(vertexFormatsRawArray, vertexFormats);
            RawArray_VertexFormat.size(vertexFormatsRawArray, pipeline.getVertexFormatBindings().length);

            var defines_raw_array = RawArray_______________FfiStr__________2.allocate(arena);
            var defines = arena.allocate(MemoryLayout.sequenceLayout(2, ValueLayout.ADDRESS), pipeline.getShaderDefines().values().size());
            RawArray_______________FfiStr__________2.contents(defines_raw_array, defines);
            RawArray_______________FfiStr__________2.size(defines_raw_array, pipeline.getShaderDefines().values().size());

            int d = 0;
            for(var entry : pipeline.getShaderDefines().values().entrySet()) {
                var slice = defines.asSlice(d * ValueLayout.ADDRESS.byteSize() * 2, ValueLayout.ADDRESS.byteSize() * 2);
                slice.set(ValueLayout.ADDRESS, 0, arena.allocateFrom(entry.getKey()));
                slice.set(ValueLayout.ADDRESS, ValueLayout.ADDRESS.byteSize(), arena.allocateFrom(entry.getValue()));
                d++;
            }

            var colorTargetCount = pipeline.getColorTargetStates().length;

            var rawArrayColorTargetState = RawArray_BlazeColorTargetState.allocate(arena);
            var colorTargetStates = BlazeColorTargetState.allocateArray(colorTargetCount, arena);

            RawArray_BlazeColorTargetState.size(rawArrayColorTargetState, colorTargetCount);
            RawArray_BlazeColorTargetState.contents(rawArrayColorTargetState, colorTargetStates);

            for(int i=0;i<colorTargetCount;i++) {
                var colorTargetState = pipeline.getColorTargetStates()[i];
                var slice = BlazeColorTargetState.asSlice(colorTargetStates, i);
                
                //todo
            }
            
            var depthTargetState = MemorySegment.NULL;
            
            if(pipeline.getDepthStencilState() != null) {
                depthTargetState = BlazeDepthStencilState.allocate(arena);
            }

            var renderPipelineStruct = dev.birb.wm.RenderPipeline.allocate(arena);
            dev.birb.wm.RenderPipeline.bind_group_layouts(renderPipelineStruct, raw_array_bind_group_descriptors);
            dev.birb.wm.RenderPipeline.vertex_formats(renderPipelineStruct, vertexFormatsRawArray);
            dev.birb.wm.RenderPipeline.vertex_shader(renderPipelineStruct, arena.allocateFrom(vertSource));
            dev.birb.wm.RenderPipeline.fragment_shader(renderPipelineStruct, arena.allocateFrom(fragSource));
            dev.birb.wm.RenderPipeline.defines(renderPipelineStruct, defines_raw_array);
            dev.birb.wm.RenderPipeline.depth_stencil_state(renderPipelineStruct, depthTargetState);
            dev.birb.wm.RenderPipeline.color_target_states(renderPipelineStruct, rawArrayColorTargetState);

            System.out.println(pipeline.getLocation());
            nativePipeline = WM.compile_render_pipeline(renderPipelineStruct);
        }

    }

    @Override
    public boolean isValid() {
        return true;
    }

}
