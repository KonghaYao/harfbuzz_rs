#include "../../harfbuzz/src/hb.h"

#ifdef __cplusplus
extern "C"
{
#endif

    int hb_glyph_to_svg_path(hb_font_t *font, hb_codepoint_t glyph, char *buf, unsigned buf_size);

#ifdef __cplusplus
}
#endif