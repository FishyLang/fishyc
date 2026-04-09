; ModuleID = 'fishy_module'
source_filename = "fishy_module"

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.2 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.3 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.4 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

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
  %call1 = call i32 (ptr, ...) @printf(ptr @global_str.2, ptr %load)
  %call2 = call i32 (ptr, ...) @printf(ptr @global_str.3)
  call void @exit(i32 1)
  ret void
}

define i64 @main() {
bb49:
  %res = alloca i64, align 8
  store i64 0, ptr %res, align 4
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 0, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 200, ptr %gep1, align 4
  %ptr2int = ptrtoint ptr %data_ptr to i64
  %add = add i64 %ptr2int, 0
  store i64 %add, ptr %res, align 4
  %isOk = alloca i64, align 8
  store i64 0, ptr %isOk, align 4
  %load = load i64, ptr %res, align 4
  %call = call i1 @Result_i64_i64_is_ok.1(i64 %load)
  %zext = zext i1 %call to i64
  store i64 %zext, ptr %isOk, align 4
  %load2 = load i64, ptr %isOk, align 4
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.4, i64 %load2)
  ret i64 0
}

declare i1 @Result_i64_i64_is_ok(i64)

define i1 @Result_i64_i64_is_ok.1(i64 %0) {
bb120:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %load = load i64, ptr %self, align 4
  %match_res = alloca i1, align 1
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb122, label %bb123

bb121:                                            ; preds = %bb125, %bb124, %bb122
  %load2 = load i1, ptr %match_res, align 1
  %zext3 = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %zext3 to i1
  ret i1 %ret_trunc

bb122:                                            ; preds = %bb120
  store i1 true, ptr %match_res, align 1
  br label %bb121

bb123:                                            ; preds = %bb120
  %cmpeq4 = icmp eq i64 %load1, 1
  %zext5 = zext i1 %cmpeq4 to i64
  %trunc6 = trunc i64 %zext5 to i1
  br i1 %trunc6, label %bb124, label %bb125

bb124:                                            ; preds = %bb123
  store i1 false, ptr %match_res, align 1
  br label %bb121

bb125:                                            ; preds = %bb123
  br label %bb121

bb196:                                            ; No predecessors!
  %self7 = alloca i64, align 8
  store i64 %0, ptr %self7, align 4
  %load8 = load i64, ptr %self7, align 4
  %match_res9 = alloca i1, align 1
  %inttoptr10 = inttoptr i64 %load8 to ptr
  %gep11 = getelementptr i64, ptr %inttoptr10, i64 0
  %load12 = load i64, ptr %gep11, align 4
  %cmpeq13 = icmp eq i64 %load12, 0
  %zext14 = zext i1 %cmpeq13 to i64
  %trunc15 = trunc i64 %zext14 to i1
  br i1 %trunc15, label %bb198, label %bb199

bb197:                                            ; preds = %bb201, %bb200, %bb198
  %load16 = load i1, ptr %match_res9, align 1
  %zext17 = zext i1 %load16 to i64
  %ret_trunc18 = trunc i64 %zext17 to i1
  ret i1 %ret_trunc18

bb198:                                            ; preds = %bb196
  store i1 true, ptr %match_res9, align 1
  br label %bb197

bb199:                                            ; preds = %bb196
  %cmpeq19 = icmp eq i64 %load12, 1
  %zext20 = zext i1 %cmpeq19 to i64
  %trunc21 = trunc i64 %zext20 to i1
  br i1 %trunc21, label %bb200, label %bb201

bb200:                                            ; preds = %bb199
  store i1 false, ptr %match_res9, align 1
  br label %bb197

bb201:                                            ; preds = %bb199
  br label %bb197
}

define void @__global_init() {
bb0:
  ret void
}
