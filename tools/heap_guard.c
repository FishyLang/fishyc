/* Test-only linker wrapper: checks allocation tails and quarantines freed blocks.
 * This is deliberately not a general-purpose allocator or an ASan replacement.
 * Link generated objects with -Wl,--wrap=malloc -Wl,--wrap=free. */
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
void *__real_malloc(size_t);
struct allocation { struct allocation *next; size_t size; int freed; max_align_t align; };
static struct allocation *allocations;
static const unsigned char marker=0xa7;
void *__wrap_malloc(size_t size) {
    if(size>SIZE_MAX-sizeof(struct allocation)-32) return NULL;
    struct allocation *a=__real_malloc(sizeof(*a)+size+32);
    if(!a) return NULL;
    a->next=allocations;a->size=size;a->freed=0;allocations=a;
    unsigned char *p=(unsigned char *)(a+1);
    memset(p,0xcc,size);memset(p+size,marker,32);return p;
}
void __wrap_free(void *p) {
    if(!p) return;
    struct allocation *a=allocations;
    while(a && (void *)(a+1)!=p) a=a->next;
    if(!a || a->freed) {fputs("heap guard: invalid/double free\n",stderr);exit(99);}
    for(size_t i=0;i<32;i++) if(((unsigned char *)p)[a->size+i]!=marker) {
        fputs("heap guard: allocation overrun\n",stderr);exit(99);
    }
    a->freed=1;memset(p,0xdd,a->size); /* quarantine: deterministic stale reads */
}
