// Don't bring in all of stdio.h, we'll use libc::FILE as needed
typedef struct _IO_FILE FILE;

#include <babeltrace2/babeltrace.h>
#include "common/metadata/decoder.h"
#include "common/msg-iter/msg-iter.h"
#include "lib/graph/component-class.h"
#include "lib/graph/component.h"
