/* Test-only allocator: checks bounds, quarantines freed memory, and counts live
 * allocations. It is not a production allocator or a replacement for ASan. */
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
void *__real_malloc(size_t);
struct allocation { struct allocation *next; size_t size; int freed; max_align_t align; };
static struct allocation *allocations;
static size_t live;
static const unsigned char marker=0xa7;
static void *guard_malloc(size_t size) {
    if(size>SIZE_MAX-sizeof(struct allocation)-32) return NULL;
    struct allocation *a=__real_malloc(sizeof(*a)+size+32);
    if(!a) return NULL;
    a->next=allocations;a->size=size;a->freed=0;allocations=a;++live;
    unsigned char *p=(unsigned char *)(a+1);
    memset(p,0xcc,size);memset(p+size,marker,32);return p;
}
static void guard_free(void *p) {
    if(!p) return;
    struct allocation *a=allocations;
    while(a && (void *)(a+1)!=p) a=a->next;
    if(!a || a->freed) {fputs("heap guard: invalid/double free\n",stderr);_Exit(99);}
    for(size_t i=0;i<32;i++) if(((unsigned char *)p)[a->size+i]!=marker) {
        fputs("heap guard: allocation overrun\n",stderr);_Exit(99);
    }
    a->freed=1;--live;memset(p,0xdd,a->size);
}
static void *guard_realloc(void *p,size_t size) {
    if(!p) return guard_malloc(size);
    struct allocation *a=allocations;
    while(a && (void *)(a+1)!=p) a=a->next;
    if(!a || a->freed) {fputs("heap guard: invalid realloc\n",stderr);_Exit(99);}
    void *next=guard_malloc(size);
    if(next) {memcpy(next,p,a->size<size?a->size:size);guard_free(p);}
    return next;
}
void *__wrap_malloc(size_t size) {void *p=guard_malloc(size);return p;}
void __wrap_free(void *p) {guard_free(p);}
void *__wrap_realloc(void *p,size_t size) {void *q=guard_realloc(p,size);return q;}
__attribute__((destructor)) static void check_live(void) {

    const char *expected=getenv("FISHYC_EXPECT_LIVE");
    if(expected && live!=strtoull(expected,NULL,10)) {
        fprintf(stderr,"heap guard: %zu live allocations, expected %s\n",live,expected);
        _Exit(99);
    }

}
