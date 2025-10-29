#ifndef HTML_TO_MARKDOWN_RB_STRINGS_H
#define HTML_TO_MARKDOWN_RB_STRINGS_H

#include <string.h>

#ifdef _WIN32
#  include <ctype.h>
#  ifndef strcasecmp
#    define strcasecmp _stricmp
#  endif
#  ifndef strncasecmp
#    define strncasecmp _strnicmp
#  endif
#endif

#endif /* HTML_TO_MARKDOWN_RB_STRINGS_H */
