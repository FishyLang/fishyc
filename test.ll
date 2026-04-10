; ModuleID = 'fishy_module'
source_filename = "fishy_module"

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.1 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.2 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1

declare ptr @malloc(i64)

declare void @free(ptr)

declare ptr @realloc(ptr, i64)

declare ptr @memcpy(ptr, ptr, i64)

declare i64 @strlen(ptr)

declare ptr @strcat(ptr, ptr)

declare i32 @printf(ptr, ...)

declare ptr @fopen(ptr, ptr)

declare i32 @fclose(ptr)

declare i32 @fputs(ptr, ptr)

declare i32 @fprintf(ptr, ptr, ...)

declare void @exit(i32)

define void @panic(ptr %0) {
bb1:
  %message = alloca ptr, align 8
  store ptr %0, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str)
  %load = load ptr, ptr %message, align 8
  %call1 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load)
  %call2 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  ret void
}

define i64 @main() {
bb49:
  ret i64 1
}

define void @__global_init() {
bb0:
  ret void
}
