#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

enum UniformType
#if __STDC_VERSION__ >= 202311L
  : uint64_t
#endif // __STDC_VERSION__ >= 202311L
 {
  TexelBuffer = 0,
  UBO = 1,
  Sampler = 2,
};
#if __STDC_VERSION__ >= 202311L
typedef enum UniformType UniformType;
#else
typedef uint64_t UniformType;
#endif // __STDC_VERSION__ >= 202311L

enum GpuFormat
#if __STDC_VERSION__ >= 202311L
  : uint64_t
#endif // __STDC_VERSION__ >= 202311L
 {
  None = 0,
  R8_UNORM = 1,
  R8_SNORM = 2,
  RG8_UNORM = 3,
  RG8_SNORM = 4,
  RGB8_UNORM = 5,
  RGB8_SNORM = 6,
  RGBA8_UNORM = 7,
  RGBA8_SNORM = 8,
  R16_UNORM = 9,
  R16_SNORM = 10,
  RG16_UNORM = 11,
  RG16_SNORM = 12,
  RGB16_UNORM = 13,
  RGB16_SNORM = 14,
  RGBA16_UNORM = 15,
  RGBA16_SNORM = 16,
  R8_UINT = 17,
  R8_SINT = 18,
  RG8_UINT = 19,
  RG8_SINT = 20,
  RGB8_UINT = 21,
  RGB8_SINT = 22,
  RGBA8_UINT = 23,
  RGBA8_SINT = 24,
  R16_UINT = 25,
  R16_SINT = 26,
  RG16_UINT = 27,
  RG16_SINT = 28,
  RGB16_UINT = 29,
  RGB16_SINT = 30,
  RGBA16_UINT = 31,
  RGBA16_SINT = 32,
  R32_UINT = 33,
  R32_SINT = 34,
  RG32_UINT = 35,
  RG32_SINT = 36,
  RGB32_UINT = 37,
  RGB32_SINT = 38,
  RGBA32_UINT = 39,
  RGBA32_SINT = 40,
  R16_FLOAT = 41,
  RG16_FLOAT = 42,
  RGB16_FLOAT = 43,
  RGBA16_FLOAT = 44,
  R32_FLOAT = 45,
  RG32_FLOAT = 46,
  RGB32_FLOAT = 47,
  RGBA32_FLOAT = 48,
  RGB10A2_UNORM = 49,
  RGB10A2_UINT = 50,
  RG11B10_FLOAT = 51,
  D32_FLOAT = 52,
  D32_FLOAT_S8_UINT = 53,
  D24_UNORM_S8_UINT = 54,
  D16_UNORM = 55,
  S8_UINT = 56,
};
#if __STDC_VERSION__ >= 202311L
typedef enum GpuFormat GpuFormat;
#else
typedef uint64_t GpuFormat;
#endif // __STDC_VERSION__ >= 202311L

enum PrimitiveTopology
#if __STDC_VERSION__ >= 202311L
  : uint64_t
#endif // __STDC_VERSION__ >= 202311L
 {
  Lines = 1,
  DebugLineStrip = 2,
  Points = 3,
  Tris = 4,
  TriangleStrip = 5,
  TriangleFan = 6,
  Quads = 7,
};
#if __STDC_VERSION__ >= 202311L
typedef enum PrimitiveTopology PrimitiveTopology;
#else
typedef uint64_t PrimitiveTopology;
#endif // __STDC_VERSION__ >= 202311L

typedef struct TextureView_ TextureView_;

typedef struct Texture_ Texture_;

typedef struct BlazeAttachmentDescriptor__________f32__________4 {
  const struct TextureView_ *texture_view;
  const float (*clear_value)[4];
} BlazeAttachmentDescriptor__________f32__________4;

typedef struct RawArray_BlazeAttachmentDescriptor__________f32__________4 {
  const struct BlazeAttachmentDescriptor__________f32__________4 *contents;
  uint64_t size;
} RawArray_BlazeAttachmentDescriptor__________f32__________4;

typedef struct BlazeAttachmentDescriptor_f64 {
  const struct TextureView_ *texture_view;
  const double *clear_value;
} BlazeAttachmentDescriptor_f64;

typedef struct BlazeRenderPassDescriptor {
  const struct RawArray_BlazeAttachmentDescriptor__________f32__________4 *attachments;
  const struct BlazeAttachmentDescriptor_f64 *depth_attachment;
} BlazeRenderPassDescriptor;

typedef struct BindGroupEntryDescriptor {
  UniformType type_;
  char* name;
  GpuFormat texture_format;
} BindGroupEntryDescriptor;

typedef struct RawArray_BindGroupEntryDescriptor {
  const struct BindGroupEntryDescriptor *contents;
  uint64_t size;
} RawArray_BindGroupEntryDescriptor;

typedef struct BlazeBindGroupLayout {
  const struct RawArray_BindGroupEntryDescriptor *entries;
} BlazeBindGroupLayout;

typedef struct RawArray_BlazeBindGroupLayout {
  const struct BlazeBindGroupLayout *contents;
  uint64_t size;
} RawArray_BlazeBindGroupLayout;

typedef struct BlazeColorTargetState {
  uint64_t blend_function;
} BlazeColorTargetState;

typedef struct RawArray_BlazeColorTargetState {
  const struct BlazeColorTargetState *contents;
  uint64_t size;
} RawArray_BlazeColorTargetState;

typedef struct BlazeDepthStencilState {
  uint64_t compare_function;
} BlazeDepthStencilState;

typedef struct VertexFormatElement {
  uint64_t offset;
  GpuFormat format;
  char* name;
} VertexFormatElement;

typedef struct RawArray_VertexFormatElement {
  const struct VertexFormatElement *contents;
  uint64_t size;
} RawArray_VertexFormatElement;

typedef struct VertexFormat {
  const struct RawArray_VertexFormatElement *elements;
  uint64_t vertex_size;
} VertexFormat;

typedef struct RawArray_VertexFormat {
  const struct VertexFormat *contents;
  uint64_t size;
} RawArray_VertexFormat;

typedef struct FragState {

} FragState;

typedef struct RenderPipeline {
  const struct RawArray_BlazeBindGroupLayout *bind_group_layouts;
  const struct RawArray_BlazeColorTargetState *color_target_states;
  const struct BlazeDepthStencilState *depth_stencil_state;
  const struct RawArray_VertexFormat *vertex_formats;
  char* vertex_shader;
  char* fragment_shader;
  char* directives;
  const struct FragState *frag_state;
  PrimitiveTopology primitive_topology;
} RenderPipeline;

void configure_surface(const uint8_t *wm, uint32_t width, uint32_t height, uint32_t present_mode);

void drop_surface(const uint8_t *wm);

void create_surface(const uint8_t *wm, uint64_t display, uint64_t window);

uint8_t *create_command_encoder(const uint8_t *wm);

struct TextureView_ *create_texture_view(const uint8_t *wm, const struct Texture_ *texture);

uint8_t *create_render_pass(uint8_t *encoder,
                            const struct BlazeRenderPassDescriptor *render_pass_descriptor);

uint8_t *create_buffer(const uint8_t *wm, const char *label, uint32_t usage, uint64_t size);

void write_to_buffer(const uint8_t *wm,
                     const uint8_t *buffer,
                     uint64_t start,
                     uint64_t length,
                     const uint8_t *data);

void copy_buffer_to_buffer(const uint8_t *wm,
                           uint8_t *encoder,
                           const uint8_t *src,
                           const uint8_t *dest,
                           uint64_t src_offset,
                           uint64_t dest_offset,
                           uint64_t length);

void copy_texture_to_buffer(uint8_t *encoder,
                            const struct Texture_ *source,
                            const uint8_t *dest,
                            uint64_t offset,
                            uint32_t mip,
                            uint32_t x,
                            uint32_t y,
                            uint32_t width,
                            uint32_t height);

void copy_texture_to_texture(uint8_t *encoder,
                             const struct Texture_ *source,
                             const struct Texture_ *destination,
                             uint32_t mip,
                             uint32_t dest_x,
                             uint32_t dest_y,
                             uint32_t src_x,
                             uint32_t src_y,
                             uint32_t width,
                             uint32_t height);

void bind_render_pipeline_to_pass(uint8_t *render_pass, const struct RenderPipeline *pipeline);

void set_index_buffer(uint8_t *pass, const uint8_t *buffer, bool int_indices);

void set_vertex_buffer(uint8_t *pass,
                       uint32_t slot,
                       const uint8_t *buffer,
                       uint64_t buffer_start,
                       uint64_t size);

void draw(uint8_t *pass,
          uint32_t vertex_count,
          uint32_t instance_count,
          uint32_t first_vertex,
          uint32_t first_instance);

void draw_indexed(uint8_t *pass,
                  uint32_t index_count,
                  uint32_t instance_count,
                  uint32_t first_index,
                  int32_t vertex_offset,
                  uint32_t first_instance);

void bind_texture_to_render_pass(uint8_t *render_pass,
                                 uint32_t slot,
                                 const struct TextureView_ *texture);

struct RenderPipeline *compile_render_pipeline(const uint8_t *wm,
                                               const struct RenderPipeline *render_pipeline_description);

void unmap_buffer(const uint8_t *buffer);

void drop_buffer_view(uint8_t*);

void write_buffer_with(const uint8_t *wm, const uint8_t *buffer, const uint8_t *data, uint64_t len);

uint8_t *allocate_gpu_buffer_mapped(const uint8_t *wm, uint64_t size, uint64_t usages);

uint8_t *acquire_next_texture(const uint8_t *wm);

void present_surface(const uint8_t *wm, uint8_t *surface_texture);

void copy_buffer_to_texture(uint8_t *encoder,
                            const uint8_t *buffer,
                            uint64_t buffer_start,
                            uint64_t buffer_end,
                            uint32_t source_x,
                            uint32_t source_y,
                            uint32_t source_width,
                            uint32_t source_height,
                            const struct Texture_ *destination,
                            uint32_t destination_x,
                            uint32_t destination_y,
                            uint32_t copy_width,
                            uint32_t copy_height,
                            uint32_t mip_level,
                            uint32_t depth_or_array_layers);

void write_to_texture(const uint8_t *wm,
                      const struct Texture_ *destination,
                      const uint8_t *source,
                      uint64_t source_size,
                      uint32_t mip_level,
                      uint32_t depth_or_array_layers,
                      uint32_t dest_x,
                      uint32_t dest_y,
                      uint32_t width,
                      uint32_t height);

void submit_command_encoder(const uint8_t *wm, uint8_t *encoder);

void submit_render_pass(uint8_t*);

void blit_from_texture(const uint8_t *wm,
                       const struct TextureView_ *texture_view,
                       const uint8_t *surface_texture);

uint8_t *create_buffer_init(const uint8_t *wm,
                            const char *label,
                            uint32_t usage,
                            uint8_t *data,
                            uint64_t size);

struct Texture_ *create_texture(const uint8_t *wm,
                                GpuFormat format_id,
                                uint32_t width,
                                uint32_t height,
                                uint32_t depth_or_layers,
                                uint32_t usage);

void drop_texture(struct Texture_*);

void drop_texture_view(struct TextureView_*);

void drop_buffer(uint8_t*);

uint32_t max_texture_size(const uint8_t *wm);

uint32_t min_uniform_offset_alignment(const uint8_t *wm);

uint8_t *extract_directives(const char *glsl);

void dummy(struct BlazeRenderPassDescriptor);

void thing(GpuFormat format);
