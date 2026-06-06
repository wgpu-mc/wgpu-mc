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

enum NormalizedType
#if __STDC_VERSION__ >= 202311L
  : uint64_t
#endif // __STDC_VERSION__ >= 202311L
 {
  F32x3 = 1,
  U8x8 = 2,
  U8x4 = 3,
  F32x2 = 4,
  F32 = 5,
  F32x4 = 6,
  I16x2 = 7,
  U8x4Norm = 8,
  S8x3Norm = 9,
};
#if __STDC_VERSION__ >= 202311L
typedef enum NormalizedType NormalizedType;
#else
typedef uint64_t NormalizedType;
#endif // __STDC_VERSION__ >= 202311L

typedef struct TextureView_ TextureView_;

typedef struct Texture_ Texture_;

typedef struct UniformDescriptor {
  UniformType type_;
  const char *name;
} UniformDescriptor;

typedef struct VertexFormatElement {
  uint64_t offset;
  NormalizedType type_;
} VertexFormatElement;

typedef struct VertexFormat {
  const struct VertexFormatElement *elements;
  uint64_t elements_count;
  uint64_t vertex_size;
  uint64_t primitive;
} VertexFormat;

typedef struct FragState {

} FragState;

typedef struct RenderPipeline {
  const struct UniformDescriptor *uniforms;
  uint64_t uniforms_count;
  const struct VertexFormat *vertex_format;
  const char *vertex_shader;
  const char *fragment_shader;
  const char *const (*defines)[2];
  uint64_t defines_count;
  const struct FragState *frag_state;
  uint64_t depth;
} RenderPipeline;

uint8_t *create_command_encoder(void);

struct TextureView_ *create_texture_view(const struct Texture_ *texture);

uint8_t *create_render_pass(uint8_t *encoder,
                            const struct TextureView_ *color_texture,
                            bool clear,
                            uint32_t clear_color,
                            const struct TextureView_ *depth_texture,
                            bool clear_depth,
                            double depth);

void write_mapped_buffer(const uint8_t *buffer, uint8_t *data, uint64_t size);

uint8_t *create_buffer(const char *label, uint32_t usage, uint64_t size);

void write_to_buffer(const uint8_t *buffer, uint64_t start, uint64_t length, const uint8_t *data);

void copy_buffer_to_buffer(uint8_t *encoder,
                           const uint8_t *src,
                           const uint8_t *dest,
                           uint64_t src_offset,
                           uint64_t dest_offset,
                           uint64_t length);

void bind_render_pipeline_to_pass(uint8_t *render_pass, const struct RenderPipeline *pipeline);

void bind_texture_to_render_pass(uint8_t *render_pass,
                                 uint32_t slot,
                                 const struct TextureView_ *texture);

struct RenderPipeline *compile_render_pipeline(const struct RenderPipeline *render_pipeline_description);

void present_texture(uint8_t *encoder, const struct TextureView_ *texture_view);

uint8_t *create_buffer_init(const char *label, uint32_t usage, uint8_t *data, uint64_t size);

struct Texture_ *create_texture(uint32_t format_id,
                                uint32_t width,
                                uint32_t height,
                                uint32_t depth_or_layers,
                                uint32_t usage);

void drop_texture(struct Texture_*);

void drop_texture_view(struct TextureView_*);

void drop_buffer(uint8_t*);

uint32_t max_texture_size(void);

uint32_t min_uniform_offset_alignment(void);

uint8_t *extract_directives(const char *glsl);
