/* Allocation accounting for this prototype only. Reuse the unchanged guard;
 * its retained records let us count requests without changing ARC/runtime. */
#include "../ownership/heap_guard.c"

uint64_t prototype_allocations(void) {
    uint64_t count = 0;
    for (struct allocation *a = allocations; a; a = a->next) ++count;
    return count;
}
uint64_t prototype_live(void) { return live; }
uint64_t prototype_live_bytes(void) {
    uint64_t bytes = 0;
    for (struct allocation *a = allocations; a; a = a->next)
        if (!a->freed) bytes += a->size;
    return bytes;
}

/* Optional source instrumentation counts method entries, never ARC operations. */
static uint64_t work_entries;
void prototype_tick(void) { ++work_entries; }
uint64_t prototype_work(void) { return work_entries; }
