#include "hb.h"
#include "hb-draw.h"

extern "C"
{
    int hb_glyph_to_svg_path(hb_font_t *font, hb_codepoint_t glyph, char *buf, unsigned buf_size);
}