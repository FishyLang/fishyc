; ModuleID = 'fishy_module'
source_filename = "fishy_module"

%Vec_str = type { ptr, i64, i64 }
%Point3_f32 = type { float, float, float }

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.1 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.2 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.3 = private unnamed_addr constant [40 x i8] c"Null pointer dereference (Array Write)!\00", align 1
@global_str.4 = private unnamed_addr constant [60 x i8] c"Array index out of bounds! Invalid memory access prevented.\00", align 1
@global_str.5 = private unnamed_addr constant [39 x i8] c"Null pointer dereference (Array Read)!\00", align 1
@global_str.6 = private unnamed_addr constant [41 x i8] c"Null pointer dereference on Method Call!\00", align 1
@global_str.7 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.8 = private unnamed_addr constant [5 x i8] c"John\00", align 1
@global_str.9 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.10 = private unnamed_addr constant [2 x i8] c"h\00", align 1
@global_str.11 = private unnamed_addr constant [2 x i8] c"J\00", align 1
@global_str.12 = private unnamed_addr constant [2 x i8] c"n\00", align 1
@global_str.13 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@global_str.14 = private unnamed_addr constant [59 x i8] c"write down your favorite fruits separated by a dash (-): \0A\00", align 1
@global_str.15 = private unnamed_addr constant [2 x i8] c"-\00", align 1
@global_str.16 = private unnamed_addr constant [19 x i8] c"\0Ayour fruits are:\0A\00", align 1
@global_str.17 = private unnamed_addr constant [6 x i8] c"- %s\0A\00", align 1
@global_str.18 = private unnamed_addr constant [38 x i8] c"Called Result#unwrap on an Err value\0A\00", align 1
@global_str.19 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.20 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.21 = private unnamed_addr constant [33 x i8] c"out of memory during Vec#reserve\00", align 1
@global_str.22 = private unnamed_addr constant [34 x i8] c"index out of bounds in Vec#insert\00", align 1
@global_str.23 = private unnamed_addr constant [29 x i8] c"cannot pop from an empty Vec\00", align 1
@global_str.24 = private unnamed_addr constant [34 x i8] c"index out of bounds in Vec#remove\00", align 1
@global_str.25 = private unnamed_addr constant [40 x i8] c"index out of bounds for Vec#swap_remove\00", align 1
@global_str.26 = private unnamed_addr constant [31 x i8] c"index out of bounds in Vec#get\00", align 1
@global_str.27 = private unnamed_addr constant [31 x i8] c"index out of bounds in Vec#set\00", align 1

declare ptr @malloc(i64)

declare void @free(ptr)

declare ptr @realloc(ptr, i64)

declare ptr @memcpy(ptr, ptr, i64)

declare i32 @printf(ptr, ...)

declare ptr @fopen(ptr, ptr)

declare i32 @fclose(ptr)

declare i32 @fputs(ptr, ptr)

declare i32 @fprintf(ptr, ptr, ...)

declare void @exit(i32)

declare i64 @getchar()

declare i64 @strlen(ptr)

declare ptr @strcat(ptr, ptr)

declare i64 @strstr(ptr, ptr)

declare i64 @strcmp(ptr, ptr)

declare ptr @strncpy(ptr, ptr, i64)

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

define i64 @Result_Ok(i64 %0) {
bb2:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 0, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Result_Err(i64 %0) {
bb3:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 1, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Option_Some(i64 %0) {
bb4:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 0, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Option_None() {
bb5:
  %arr_alloc = call ptr @malloc(i64 24)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 1, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 1, ptr %gep, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define ptr @read_line(i64 %0) {
bb6:
  %buf_size = alloca i64, align 8
  store i64 %0, ptr %buf_size, align 4
  %buffer = alloca ptr, align 8
  store ptr null, ptr %buffer, align 8
  %load = load i64, ptr %buf_size, align 4
  %call = call ptr @malloc(i64 %load)
  store ptr %call, ptr %buffer, align 8
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 4
  store i64 0, ptr %i, align 4
  %reading = alloca i1, align 1
  store i1 false, ptr %reading, align 1
  store i1 true, ptr %reading, align 1
  br label %bb7

bb7:                                              ; preds = %bb11, %bb6
  %load1 = load i1, ptr %reading, align 1
  %load_zext = zext i1 %load1 to i64
  %cond_true = icmp ne i64 %load_zext, 0
  br i1 %cond_true, label %bb8, label %bb9

bb8:                                              ; preds = %bb7
  %c = alloca i64, align 8
  store i64 0, ptr %c, align 4
  %call2 = call i64 @getchar()
  store i64 %call2, ptr %c, align 4
  %load3 = load i64, ptr %c, align 4
  %cmpeq = icmp eq i64 %load3, -1
  %zext = zext i1 %cmpeq to i64
  %cond_true4 = icmp ne i64 %zext, 0
  br i1 %cond_true4, label %bb10, label %bb12

bb9:                                              ; preds = %bb7
  %load5 = load ptr, ptr %buffer, align 8
  %ptr2int = ptrtoint ptr %load5 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext6 = zext i1 %cmpne to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb23, label %bb24

bb10:                                             ; preds = %bb8
  store i1 false, ptr %reading, align 1
  br label %bb11

bb11:                                             ; preds = %bb14, %bb10
  br label %bb7

bb12:                                             ; preds = %bb8
  %load8 = load i64, ptr %c, align 4
  %cmpeq9 = icmp eq i64 %load8, 10
  %zext10 = zext i1 %cmpeq9 to i64
  %cond_true11 = icmp ne i64 %zext10, 0
  br i1 %cond_true11, label %bb13, label %bb15

bb13:                                             ; preds = %bb12
  store i1 false, ptr %reading, align 1
  br label %bb14

bb14:                                             ; preds = %bb17, %bb13
  br label %bb11

bb15:                                             ; preds = %bb12
  %load12 = load i64, ptr %i, align 4
  %load13 = load i64, ptr %buf_size, align 4
  %sub = sub i64 %load13, 1
  %cmplt = icmp ult i64 %load12, %sub
  %zext14 = zext i1 %cmplt to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb16, label %bb17

bb16:                                             ; preds = %bb15
  %load16 = load ptr, ptr %buffer, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpne18 = icmp ne i64 %ptr2int17, 0
  %zext19 = zext i1 %cmpne18 to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb18, label %bb19

bb17:                                             ; preds = %bb22, %bb15
  br label %bb14

bb18:                                             ; preds = %bb16
  %load21 = load i64, ptr %i, align 4
  %gep = getelementptr i64, ptr %load16, i64 -1
  %load22 = load i64, ptr %gep, align 4
  %cmplt23 = icmp slt i64 %load21, %load22
  %zext24 = zext i1 %cmplt23 to i64
  %cond_true25 = icmp ne i64 %zext24, 0
  br i1 %cond_true25, label %bb20, label %bb21

bb19:                                             ; preds = %bb16
  call void @panic(ptr @global_str.3)
  unreachable

bb20:                                             ; preds = %bb18
  %cmpge = icmp sge i64 %load21, 0
  %zext26 = zext i1 %cmpge to i64
  %cond_true27 = icmp ne i64 %zext26, 0
  br i1 %cond_true27, label %bb22, label %bb21

bb21:                                             ; preds = %bb20, %bb18
  call void @panic(ptr @global_str.4)
  unreachable

bb22:                                             ; preds = %bb20
  %load28 = load i64, ptr %c, align 4
  %cast_trunc = trunc i64 %load28 to i8
  %gep29 = getelementptr i8, ptr %load16, i64 %load21
  store i8 %cast_trunc, ptr %gep29, align 1
  %load30 = load i64, ptr %i, align 4
  %add = add i64 %load30, 1
  store i64 %add, ptr %i, align 4
  br label %bb17

bb23:                                             ; preds = %bb9
  %load31 = load i64, ptr %i, align 4
  %gep32 = getelementptr i64, ptr %load5, i64 -1
  %load33 = load i64, ptr %gep32, align 4
  %cmplt34 = icmp slt i64 %load31, %load33
  %zext35 = zext i1 %cmplt34 to i64
  %cond_true36 = icmp ne i64 %zext35, 0
  br i1 %cond_true36, label %bb25, label %bb26

bb24:                                             ; preds = %bb9
  call void @panic(ptr @global_str.3)
  unreachable

bb25:                                             ; preds = %bb23
  %cmpge37 = icmp sge i64 %load31, 0
  %zext38 = zext i1 %cmpge37 to i64
  %cond_true39 = icmp ne i64 %zext38, 0
  br i1 %cond_true39, label %bb27, label %bb26

bb26:                                             ; preds = %bb25, %bb23
  call void @panic(ptr @global_str.4)
  unreachable

bb27:                                             ; preds = %bb25
  %gep40 = getelementptr i8, ptr %load5, i64 %load31
  store i8 0, ptr %gep40, align 1
  %final_str = alloca ptr, align 8
  store ptr null, ptr %final_str, align 8
  %load41 = load i64, ptr %i, align 4
  %add42 = add i64 %load41, 1
  %call43 = call ptr @malloc(i64 %add42)
  store ptr %call43, ptr %final_str, align 8
  %j = alloca i64, align 8
  store i64 0, ptr %j, align 4
  store i64 0, ptr %j, align 4
  br label %bb28

bb28:                                             ; preds = %bb40, %bb27
  %load44 = load i64, ptr %j, align 4
  %load45 = load i64, ptr %i, align 4
  %cmplt46 = icmp ult i64 %load44, %load45
  %zext47 = zext i1 %cmplt46 to i64
  %cond_true48 = icmp ne i64 %zext47, 0
  br i1 %cond_true48, label %bb29, label %bb30

bb29:                                             ; preds = %bb28
  %load49 = load ptr, ptr %final_str, align 8
  %ptr2int50 = ptrtoint ptr %load49 to i64
  %cmpne51 = icmp ne i64 %ptr2int50, 0
  %zext52 = zext i1 %cmpne51 to i64
  %cond_true53 = icmp ne i64 %zext52, 0
  br i1 %cond_true53, label %bb31, label %bb32

bb30:                                             ; preds = %bb28
  %load54 = load ptr, ptr %final_str, align 8
  %ptr2int55 = ptrtoint ptr %load54 to i64
  %cmpne56 = icmp ne i64 %ptr2int55, 0
  %zext57 = zext i1 %cmpne56 to i64
  %cond_true58 = icmp ne i64 %zext57, 0
  br i1 %cond_true58, label %bb41, label %bb42

bb31:                                             ; preds = %bb29
  %load59 = load i64, ptr %j, align 4
  %gep60 = getelementptr i64, ptr %load49, i64 -1
  %load61 = load i64, ptr %gep60, align 4
  %cmplt62 = icmp slt i64 %load59, %load61
  %zext63 = zext i1 %cmplt62 to i64
  %cond_true64 = icmp ne i64 %zext63, 0
  br i1 %cond_true64, label %bb33, label %bb34

bb32:                                             ; preds = %bb29
  call void @panic(ptr @global_str.3)
  unreachable

bb33:                                             ; preds = %bb31
  %cmpge65 = icmp sge i64 %load59, 0
  %zext66 = zext i1 %cmpge65 to i64
  %cond_true67 = icmp ne i64 %zext66, 0
  br i1 %cond_true67, label %bb35, label %bb34

bb34:                                             ; preds = %bb33, %bb31
  call void @panic(ptr @global_str.4)
  unreachable

bb35:                                             ; preds = %bb33
  %load68 = load ptr, ptr %buffer, align 8
  %ptr2int69 = ptrtoint ptr %load68 to i64
  %cmpne70 = icmp ne i64 %ptr2int69, 0
  %zext71 = zext i1 %cmpne70 to i64
  %cond_true72 = icmp ne i64 %zext71, 0
  br i1 %cond_true72, label %bb36, label %bb37

bb36:                                             ; preds = %bb35
  %load73 = load i64, ptr %j, align 4
  %gep74 = getelementptr i64, ptr %load68, i64 -1
  %load75 = load i64, ptr %gep74, align 4
  %cmplt76 = icmp slt i64 %load73, %load75
  %zext77 = zext i1 %cmplt76 to i64
  %cond_true78 = icmp ne i64 %zext77, 0
  br i1 %cond_true78, label %bb38, label %bb39

bb37:                                             ; preds = %bb35
  call void @panic(ptr @global_str.5)
  unreachable

bb38:                                             ; preds = %bb36
  %cmpge79 = icmp sge i64 %load73, 0
  %zext80 = zext i1 %cmpge79 to i64
  %cond_true81 = icmp ne i64 %zext80, 0
  br i1 %cond_true81, label %bb40, label %bb39

bb39:                                             ; preds = %bb38, %bb36
  call void @panic(ptr @global_str.4)
  unreachable

bb40:                                             ; preds = %bb38
  %gep82 = getelementptr i8, ptr %load68, i64 %load73
  %load83 = load i8, ptr %gep82, align 1
  %load_zext84 = zext i8 %load83 to i64
  %gep85 = getelementptr i8, ptr %load49, i64 %load59
  %trunc = trunc i64 %load_zext84 to i8
  store i8 %trunc, ptr %gep85, align 1
  %load86 = load i64, ptr %j, align 4
  %add87 = add i64 %load86, 1
  store i64 %add87, ptr %j, align 4
  br label %bb28

bb41:                                             ; preds = %bb30
  %load88 = load i64, ptr %i, align 4
  %gep89 = getelementptr i64, ptr %load54, i64 -1
  %load90 = load i64, ptr %gep89, align 4
  %cmplt91 = icmp slt i64 %load88, %load90
  %zext92 = zext i1 %cmplt91 to i64
  %cond_true93 = icmp ne i64 %zext92, 0
  br i1 %cond_true93, label %bb43, label %bb44

bb42:                                             ; preds = %bb30
  call void @panic(ptr @global_str.3)
  unreachable

bb43:                                             ; preds = %bb41
  %cmpge94 = icmp sge i64 %load88, 0
  %zext95 = zext i1 %cmpge94 to i64
  %cond_true96 = icmp ne i64 %zext95, 0
  br i1 %cond_true96, label %bb45, label %bb44

bb44:                                             ; preds = %bb43, %bb41
  call void @panic(ptr @global_str.4)
  unreachable

bb45:                                             ; preds = %bb43
  %gep97 = getelementptr i8, ptr %load54, i64 %load88
  store i8 0, ptr %gep97, align 1
  %load98 = load ptr, ptr %buffer, align 8
  call void @free(ptr %load98)
  store ptr null, ptr %buffer, align 8
  %load99 = load ptr, ptr %final_str, align 8
  ret ptr %load99
}

define i8 @u8_min() {
bb46:
  ret i8 0
}

define i8 @u8_max() {
bb47:
  ret i8 -1
}

define i8 @u8_clamp(i8 %0, i8 %1, i8 %2) {
bb48:
  %self = alloca i8, align 1
  store i8 %0, ptr %self, align 1
  %min = alloca i8, align 1
  store i8 %1, ptr %min, align 1
  %max = alloca i8, align 1
  store i8 %2, ptr %max, align 1
  %load = load i8, ptr %self, align 1
  %load_zext = zext i8 %load to i64
  %load1 = load i8, ptr %min, align 1
  %load_zext2 = zext i8 %load1 to i64
  %cmplt = icmp ult i64 %load_zext, %load_zext2
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb49, label %bb50

bb49:                                             ; preds = %bb48
  %load3 = load i8, ptr %min, align 1
  %load_zext4 = zext i8 %load3 to i64
  %ret_trunc = trunc i64 %load_zext4 to i8
  ret i8 %ret_trunc

bb50:                                             ; preds = %bb48
  %load5 = load i8, ptr %self, align 1
  %load_zext6 = zext i8 %load5 to i64
  %load7 = load i8, ptr %max, align 1
  %load_zext8 = zext i8 %load7 to i64
  %cmpgt = icmp ugt i64 %load_zext6, %load_zext8
  %zext9 = zext i1 %cmpgt to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb51, label %bb52

bb51:                                             ; preds = %bb50
  %load11 = load i8, ptr %max, align 1
  %load_zext12 = zext i8 %load11 to i64
  %ret_trunc13 = trunc i64 %load_zext12 to i8
  ret i8 %ret_trunc13

bb52:                                             ; preds = %bb50
  %load14 = load i8, ptr %self, align 1
  %load_zext15 = zext i8 %load14 to i64
  %ret_trunc16 = trunc i64 %load_zext15 to i8
  ret i8 %ret_trunc16
}

define i16 @u16_min() {
bb53:
  ret i16 0
}

define i16 @u16_max() {
bb54:
  ret i16 -1
}

define i16 @u16_clamp(i16 %0, i16 %1, i16 %2) {
bb55:
  %self = alloca i16, align 2
  store i16 %0, ptr %self, align 2
  %min = alloca i16, align 2
  store i16 %1, ptr %min, align 2
  %max = alloca i16, align 2
  store i16 %2, ptr %max, align 2
  %load = load i16, ptr %self, align 2
  %load_zext = zext i16 %load to i64
  %load1 = load i16, ptr %min, align 2
  %load_zext2 = zext i16 %load1 to i64
  %cmplt = icmp ult i64 %load_zext, %load_zext2
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb56, label %bb57

bb56:                                             ; preds = %bb55
  %load3 = load i16, ptr %min, align 2
  %load_zext4 = zext i16 %load3 to i64
  %ret_trunc = trunc i64 %load_zext4 to i16
  ret i16 %ret_trunc

bb57:                                             ; preds = %bb55
  %load5 = load i16, ptr %self, align 2
  %load_zext6 = zext i16 %load5 to i64
  %load7 = load i16, ptr %max, align 2
  %load_zext8 = zext i16 %load7 to i64
  %cmpgt = icmp ugt i64 %load_zext6, %load_zext8
  %zext9 = zext i1 %cmpgt to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb58, label %bb59

bb58:                                             ; preds = %bb57
  %load11 = load i16, ptr %max, align 2
  %load_zext12 = zext i16 %load11 to i64
  %ret_trunc13 = trunc i64 %load_zext12 to i16
  ret i16 %ret_trunc13

bb59:                                             ; preds = %bb57
  %load14 = load i16, ptr %self, align 2
  %load_zext15 = zext i16 %load14 to i64
  %ret_trunc16 = trunc i64 %load_zext15 to i16
  ret i16 %ret_trunc16
}

define i32 @u32_min() {
bb60:
  ret i32 0
}

define i32 @u32_max() {
bb61:
  ret i32 -1
}

define i32 @u32_clamp(i32 %0, i32 %1, i32 %2) {
bb62:
  %self = alloca i32, align 4
  store i32 %0, ptr %self, align 4
  %min = alloca i32, align 4
  store i32 %1, ptr %min, align 4
  %max = alloca i32, align 4
  store i32 %2, ptr %max, align 4
  %load = load i32, ptr %self, align 4
  %load_zext = zext i32 %load to i64
  %load1 = load i32, ptr %min, align 4
  %load_zext2 = zext i32 %load1 to i64
  %cmplt = icmp ult i64 %load_zext, %load_zext2
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb63, label %bb64

bb63:                                             ; preds = %bb62
  %load3 = load i32, ptr %min, align 4
  %load_zext4 = zext i32 %load3 to i64
  %ret_trunc = trunc i64 %load_zext4 to i32
  ret i32 %ret_trunc

bb64:                                             ; preds = %bb62
  %load5 = load i32, ptr %self, align 4
  %load_zext6 = zext i32 %load5 to i64
  %load7 = load i32, ptr %max, align 4
  %load_zext8 = zext i32 %load7 to i64
  %cmpgt = icmp ugt i64 %load_zext6, %load_zext8
  %zext9 = zext i1 %cmpgt to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb65, label %bb66

bb65:                                             ; preds = %bb64
  %load11 = load i32, ptr %max, align 4
  %load_zext12 = zext i32 %load11 to i64
  %ret_trunc13 = trunc i64 %load_zext12 to i32
  ret i32 %ret_trunc13

bb66:                                             ; preds = %bb64
  %load14 = load i32, ptr %self, align 4
  %load_zext15 = zext i32 %load14 to i64
  %ret_trunc16 = trunc i64 %load_zext15 to i32
  ret i32 %ret_trunc16
}

define i64 @u64_min() {
bb67:
  ret i64 0
}

define i64 @u64_max() {
bb68:
  ret i64 -1
}

define i64 @u64_clamp(i64 %0, i64 %1, i64 %2) {
bb69:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %min = alloca i64, align 8
  store i64 %1, ptr %min, align 4
  %max = alloca i64, align 8
  store i64 %2, ptr %max, align 4
  %load = load i64, ptr %self, align 4
  %load1 = load i64, ptr %min, align 4
  %cmplt = icmp ult i64 %load, %load1
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb70, label %bb71

bb70:                                             ; preds = %bb69
  %load2 = load i64, ptr %min, align 4
  ret i64 %load2

bb71:                                             ; preds = %bb69
  %load3 = load i64, ptr %self, align 4
  %load4 = load i64, ptr %max, align 4
  %cmpgt = icmp ugt i64 %load3, %load4
  %zext5 = zext i1 %cmpgt to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb72, label %bb73

bb72:                                             ; preds = %bb71
  %load7 = load i64, ptr %max, align 4
  ret i64 %load7

bb73:                                             ; preds = %bb71
  %load8 = load i64, ptr %self, align 4
  ret i64 %load8
}

define i8 @i8_min() {
bb74:
  ret i8 -128
}

define i8 @i8_max() {
bb75:
  ret i8 127
}

define i8 @i8_abs(i8 %0) {
bb76:
  %self = alloca i8, align 1
  store i8 %0, ptr %self, align 1
  %load = load i8, ptr %self, align 1
  %load_sext = sext i8 %load to i64
  %cmplt = icmp slt i64 %load_sext, 0
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb77, label %bb78

bb77:                                             ; preds = %bb76
  %load1 = load i8, ptr %self, align 1
  %load_sext2 = sext i8 %load1 to i64
  %sub = sub i64 0, %load_sext2
  %ret_trunc = trunc i64 %sub to i8
  ret i8 %ret_trunc

bb78:                                             ; preds = %bb76
  %load3 = load i8, ptr %self, align 1
  %load_sext4 = sext i8 %load3 to i64
  %ret_trunc5 = trunc i64 %load_sext4 to i8
  ret i8 %ret_trunc5
}

define i8 @i8_clamp(i8 %0, i8 %1, i8 %2) {
bb79:
  %self = alloca i8, align 1
  store i8 %0, ptr %self, align 1
  %min = alloca i8, align 1
  store i8 %1, ptr %min, align 1
  %max = alloca i8, align 1
  store i8 %2, ptr %max, align 1
  %load = load i8, ptr %self, align 1
  %load_sext = sext i8 %load to i64
  %load1 = load i8, ptr %min, align 1
  %load_sext2 = sext i8 %load1 to i64
  %cmplt = icmp slt i64 %load_sext, %load_sext2
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb80, label %bb81

bb80:                                             ; preds = %bb79
  %load3 = load i8, ptr %min, align 1
  %load_sext4 = sext i8 %load3 to i64
  %ret_trunc = trunc i64 %load_sext4 to i8
  ret i8 %ret_trunc

bb81:                                             ; preds = %bb79
  %load5 = load i8, ptr %self, align 1
  %load_sext6 = sext i8 %load5 to i64
  %load7 = load i8, ptr %max, align 1
  %load_sext8 = sext i8 %load7 to i64
  %cmpgt = icmp sgt i64 %load_sext6, %load_sext8
  %zext9 = zext i1 %cmpgt to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb82, label %bb83

bb82:                                             ; preds = %bb81
  %load11 = load i8, ptr %max, align 1
  %load_sext12 = sext i8 %load11 to i64
  %ret_trunc13 = trunc i64 %load_sext12 to i8
  ret i8 %ret_trunc13

bb83:                                             ; preds = %bb81
  %load14 = load i8, ptr %self, align 1
  %load_sext15 = sext i8 %load14 to i64
  %ret_trunc16 = trunc i64 %load_sext15 to i8
  ret i8 %ret_trunc16
}

define i16 @i16_min() {
bb84:
  ret i16 -32768
}

define i16 @i16_max() {
bb85:
  ret i16 32767
}

define i16 @i16_abs(i16 %0) {
bb86:
  %self = alloca i16, align 2
  store i16 %0, ptr %self, align 2
  %load = load i16, ptr %self, align 2
  %load_sext = sext i16 %load to i64
  %cmplt = icmp slt i64 %load_sext, 0
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb87, label %bb88

bb87:                                             ; preds = %bb86
  %load1 = load i16, ptr %self, align 2
  %load_sext2 = sext i16 %load1 to i64
  %sub = sub i64 0, %load_sext2
  %ret_trunc = trunc i64 %sub to i16
  ret i16 %ret_trunc

bb88:                                             ; preds = %bb86
  %load3 = load i16, ptr %self, align 2
  %load_sext4 = sext i16 %load3 to i64
  %ret_trunc5 = trunc i64 %load_sext4 to i16
  ret i16 %ret_trunc5
}

define i16 @i16_clamp(i16 %0, i16 %1, i16 %2) {
bb89:
  %self = alloca i16, align 2
  store i16 %0, ptr %self, align 2
  %min = alloca i16, align 2
  store i16 %1, ptr %min, align 2
  %max = alloca i16, align 2
  store i16 %2, ptr %max, align 2
  %load = load i16, ptr %self, align 2
  %load_sext = sext i16 %load to i64
  %load1 = load i16, ptr %min, align 2
  %load_sext2 = sext i16 %load1 to i64
  %cmplt = icmp slt i64 %load_sext, %load_sext2
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb90, label %bb91

bb90:                                             ; preds = %bb89
  %load3 = load i16, ptr %min, align 2
  %load_sext4 = sext i16 %load3 to i64
  %ret_trunc = trunc i64 %load_sext4 to i16
  ret i16 %ret_trunc

bb91:                                             ; preds = %bb89
  %load5 = load i16, ptr %self, align 2
  %load_sext6 = sext i16 %load5 to i64
  %load7 = load i16, ptr %max, align 2
  %load_sext8 = sext i16 %load7 to i64
  %cmpgt = icmp sgt i64 %load_sext6, %load_sext8
  %zext9 = zext i1 %cmpgt to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb92, label %bb93

bb92:                                             ; preds = %bb91
  %load11 = load i16, ptr %max, align 2
  %load_sext12 = sext i16 %load11 to i64
  %ret_trunc13 = trunc i64 %load_sext12 to i16
  ret i16 %ret_trunc13

bb93:                                             ; preds = %bb91
  %load14 = load i16, ptr %self, align 2
  %load_sext15 = sext i16 %load14 to i64
  %ret_trunc16 = trunc i64 %load_sext15 to i16
  ret i16 %ret_trunc16
}

define i32 @i32_min() {
bb94:
  ret i32 -2147483648
}

define i32 @i32_max() {
bb95:
  ret i32 2147483647
}

define i32 @i32_abs(i32 %0) {
bb96:
  %self = alloca i32, align 4
  store i32 %0, ptr %self, align 4
  %load = load i32, ptr %self, align 4
  %load_sext = sext i32 %load to i64
  %cmplt = icmp slt i64 %load_sext, 0
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb97, label %bb98

bb97:                                             ; preds = %bb96
  %load1 = load i32, ptr %self, align 4
  %load_sext2 = sext i32 %load1 to i64
  %sub = sub i64 0, %load_sext2
  %ret_trunc = trunc i64 %sub to i32
  ret i32 %ret_trunc

bb98:                                             ; preds = %bb96
  %load3 = load i32, ptr %self, align 4
  %load_sext4 = sext i32 %load3 to i64
  %ret_trunc5 = trunc i64 %load_sext4 to i32
  ret i32 %ret_trunc5
}

define i32 @i32_clamp(i32 %0, i32 %1, i32 %2) {
bb99:
  %self = alloca i32, align 4
  store i32 %0, ptr %self, align 4
  %min = alloca i32, align 4
  store i32 %1, ptr %min, align 4
  %max = alloca i32, align 4
  store i32 %2, ptr %max, align 4
  %load = load i32, ptr %self, align 4
  %load_sext = sext i32 %load to i64
  %load1 = load i32, ptr %min, align 4
  %load_sext2 = sext i32 %load1 to i64
  %cmplt = icmp slt i64 %load_sext, %load_sext2
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb100, label %bb101

bb100:                                            ; preds = %bb99
  %load3 = load i32, ptr %min, align 4
  %load_sext4 = sext i32 %load3 to i64
  %ret_trunc = trunc i64 %load_sext4 to i32
  ret i32 %ret_trunc

bb101:                                            ; preds = %bb99
  %load5 = load i32, ptr %self, align 4
  %load_sext6 = sext i32 %load5 to i64
  %load7 = load i32, ptr %max, align 4
  %load_sext8 = sext i32 %load7 to i64
  %cmpgt = icmp sgt i64 %load_sext6, %load_sext8
  %zext9 = zext i1 %cmpgt to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb102, label %bb103

bb102:                                            ; preds = %bb101
  %load11 = load i32, ptr %max, align 4
  %load_sext12 = sext i32 %load11 to i64
  %ret_trunc13 = trunc i64 %load_sext12 to i32
  ret i32 %ret_trunc13

bb103:                                            ; preds = %bb101
  %load14 = load i32, ptr %self, align 4
  %load_sext15 = sext i32 %load14 to i64
  %ret_trunc16 = trunc i64 %load_sext15 to i32
  ret i32 %ret_trunc16
}

define i64 @i64_min() {
bb104:
  ret i64 -9223372036854775808
}

define i64 @i64_max() {
bb105:
  ret i64 9223372036854775807
}

define i64 @i64_abs(i64 %0) {
bb106:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %load = load i64, ptr %self, align 4
  %cmplt = icmp slt i64 %load, 0
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb107, label %bb108

bb107:                                            ; preds = %bb106
  %load1 = load i64, ptr %self, align 4
  %sub = sub i64 0, %load1
  ret i64 %sub

bb108:                                            ; preds = %bb106
  %load2 = load i64, ptr %self, align 4
  ret i64 %load2
}

define i64 @i64_clamp(i64 %0, i64 %1, i64 %2) {
bb109:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %min = alloca i64, align 8
  store i64 %1, ptr %min, align 4
  %max = alloca i64, align 8
  store i64 %2, ptr %max, align 4
  %load = load i64, ptr %self, align 4
  %load1 = load i64, ptr %min, align 4
  %cmplt = icmp slt i64 %load, %load1
  %zext = zext i1 %cmplt to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb110, label %bb111

bb110:                                            ; preds = %bb109
  %load2 = load i64, ptr %min, align 4
  ret i64 %load2

bb111:                                            ; preds = %bb109
  %load3 = load i64, ptr %self, align 4
  %load4 = load i64, ptr %max, align 4
  %cmpgt = icmp sgt i64 %load3, %load4
  %zext5 = zext i1 %cmpgt to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb112, label %bb113

bb112:                                            ; preds = %bb111
  %load7 = load i64, ptr %max, align 4
  ret i64 %load7

bb113:                                            ; preds = %bb111
  %load8 = load i64, ptr %self, align 4
  ret i64 %load8
}

define half @f16_min() {
bb114:
  ret half 0xHFBFF
}

define half @f16_max() {
bb115:
  ret half 0xH7BFF
}

define i64 @string_len(ptr %0) {
bb116:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %call = call i64 @strlen(ptr %load)
  ret i64 %call
}

define i1 @string_is_empty(ptr %0) {
bb117:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb118, label %bb119

bb118:                                            ; preds = %bb117
  %call = call i64 @string_len(ptr %load)
  %cmpeq = icmp eq i64 %call, 0
  %zext1 = zext i1 %cmpeq to i64
  %ret_trunc = trunc i64 %zext1 to i1
  ret i1 %ret_trunc

bb119:                                            ; preds = %bb117
  call void @panic(ptr @global_str.6)
  unreachable
}

define i1 @string_contains(ptr %0, ptr %1) {
bb120:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %sub = alloca ptr, align 8
  store ptr %1, ptr %sub, align 8
  %load = load ptr, ptr %sub, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb121, label %bb122

bb121:                                            ; preds = %bb120
  %call = call i64 @string_len(ptr %load)
  %cmpeq = icmp eq i64 %call, 0
  %zext1 = zext i1 %cmpeq to i64
  %cond_true2 = icmp ne i64 %zext1, 0
  br i1 %cond_true2, label %bb123, label %bb124

bb122:                                            ; preds = %bb120
  call void @panic(ptr @global_str.6)
  unreachable

bb123:                                            ; preds = %bb121
  ret i1 false

bb124:                                            ; preds = %bb121
  %load3 = load ptr, ptr %self, align 8
  %load4 = load ptr, ptr %sub, align 8
  %call5 = call i64 @strstr(ptr %load3, ptr %load4)
  %cmpne6 = icmp ne i64 %call5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %ret_trunc = trunc i64 %zext7 to i1
  ret i1 %ret_trunc
}

define i1 @string_starts_with(ptr %0, ptr %1) {
bb125:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %prefix = alloca ptr, align 8
  store ptr %1, ptr %prefix, align 8
  %p_len = alloca i64, align 8
  store i64 0, ptr %p_len, align 4
  %load = load ptr, ptr %prefix, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb126, label %bb127

bb126:                                            ; preds = %bb125
  %call = call i64 @string_len(ptr %load)
  store i64 %call, ptr %p_len, align 4
  %load1 = load ptr, ptr %self, align 8
  %ptr2int2 = ptrtoint ptr %load1 to i64
  %cmpne3 = icmp ne i64 %ptr2int2, 0
  %zext4 = zext i1 %cmpne3 to i64
  %cond_true5 = icmp ne i64 %zext4, 0
  br i1 %cond_true5, label %bb128, label %bb129

bb127:                                            ; preds = %bb125
  call void @panic(ptr @global_str.6)
  unreachable

bb128:                                            ; preds = %bb126
  %call6 = call i64 @string_len(ptr %load1)
  %load7 = load i64, ptr %p_len, align 4
  %cmplt = icmp ult i64 %call6, %load7
  %zext8 = zext i1 %cmplt to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb130, label %bb131

bb129:                                            ; preds = %bb126
  call void @panic(ptr @global_str.6)
  unreachable

bb130:                                            ; preds = %bb128
  ret i1 false

bb131:                                            ; preds = %bb128
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 4
  store i64 0, ptr %i, align 4
  br label %bb132

bb132:                                            ; preds = %bb146, %bb131
  %load10 = load i64, ptr %i, align 4
  %load11 = load i64, ptr %p_len, align 4
  %cmplt12 = icmp slt i64 %load10, %load11
  %zext13 = zext i1 %cmplt12 to i64
  %cond_true14 = icmp ne i64 %zext13, 0
  br i1 %cond_true14, label %bb133, label %bb134

bb133:                                            ; preds = %bb132
  %load15 = load ptr, ptr %self, align 8
  %ptr2int16 = ptrtoint ptr %load15 to i64
  %cmpne17 = icmp ne i64 %ptr2int16, 0
  %zext18 = zext i1 %cmpne17 to i64
  %cond_true19 = icmp ne i64 %zext18, 0
  br i1 %cond_true19, label %bb135, label %bb136

bb134:                                            ; preds = %bb132
  ret i1 true

bb135:                                            ; preds = %bb133
  %load20 = load i64, ptr %i, align 4
  %gep = getelementptr i64, ptr %load15, i64 -1
  %load21 = load i64, ptr %gep, align 4
  %cmplt22 = icmp slt i64 %load20, %load21
  %zext23 = zext i1 %cmplt22 to i64
  %cond_true24 = icmp ne i64 %zext23, 0
  br i1 %cond_true24, label %bb137, label %bb138

bb136:                                            ; preds = %bb133
  call void @panic(ptr @global_str.5)
  unreachable

bb137:                                            ; preds = %bb135
  %cmpge = icmp sge i64 %load20, 0
  %zext25 = zext i1 %cmpge to i64
  %cond_true26 = icmp ne i64 %zext25, 0
  br i1 %cond_true26, label %bb139, label %bb138

bb138:                                            ; preds = %bb137, %bb135
  call void @panic(ptr @global_str.4)
  unreachable

bb139:                                            ; preds = %bb137
  %gep27 = getelementptr i8, ptr %load15, i64 %load20
  %load28 = load i8, ptr %gep27, align 1
  %load_sext = sext i8 %load28 to i64
  %load29 = load ptr, ptr %prefix, align 8
  %ptr2int30 = ptrtoint ptr %load29 to i64
  %cmpne31 = icmp ne i64 %ptr2int30, 0
  %zext32 = zext i1 %cmpne31 to i64
  %cond_true33 = icmp ne i64 %zext32, 0
  br i1 %cond_true33, label %bb140, label %bb141

bb140:                                            ; preds = %bb139
  %load34 = load i64, ptr %i, align 4
  %gep35 = getelementptr i64, ptr %load29, i64 -1
  %load36 = load i64, ptr %gep35, align 4
  %cmplt37 = icmp slt i64 %load34, %load36
  %zext38 = zext i1 %cmplt37 to i64
  %cond_true39 = icmp ne i64 %zext38, 0
  br i1 %cond_true39, label %bb142, label %bb143

bb141:                                            ; preds = %bb139
  call void @panic(ptr @global_str.5)
  unreachable

bb142:                                            ; preds = %bb140
  %cmpge40 = icmp sge i64 %load34, 0
  %zext41 = zext i1 %cmpge40 to i64
  %cond_true42 = icmp ne i64 %zext41, 0
  br i1 %cond_true42, label %bb144, label %bb143

bb143:                                            ; preds = %bb142, %bb140
  call void @panic(ptr @global_str.4)
  unreachable

bb144:                                            ; preds = %bb142
  %gep43 = getelementptr i8, ptr %load29, i64 %load34
  %load44 = load i8, ptr %gep43, align 1
  %load_sext45 = sext i8 %load44 to i64
  %cmpne46 = icmp ne i64 %load_sext, %load_sext45
  %zext47 = zext i1 %cmpne46 to i64
  %cond_true48 = icmp ne i64 %zext47, 0
  br i1 %cond_true48, label %bb145, label %bb146

bb145:                                            ; preds = %bb144
  ret i1 false

bb146:                                            ; preds = %bb144
  %load49 = load i64, ptr %i, align 4
  %add = add i64 %load49, 1
  store i64 %add, ptr %i, align 4
  br label %bb132
}

define i1 @string_ends_with(ptr %0, ptr %1) {
bb147:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %suffix = alloca ptr, align 8
  store ptr %1, ptr %suffix, align 8
  %s_len = alloca i64, align 8
  store i64 0, ptr %s_len, align 4
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb148, label %bb149

bb148:                                            ; preds = %bb147
  %call = call i64 @string_len(ptr %load)
  store i64 %call, ptr %s_len, align 4
  %sub_len = alloca i64, align 8
  store i64 0, ptr %sub_len, align 4
  %load1 = load ptr, ptr %suffix, align 8
  %ptr2int2 = ptrtoint ptr %load1 to i64
  %cmpne3 = icmp ne i64 %ptr2int2, 0
  %zext4 = zext i1 %cmpne3 to i64
  %cond_true5 = icmp ne i64 %zext4, 0
  br i1 %cond_true5, label %bb150, label %bb151

bb149:                                            ; preds = %bb147
  call void @panic(ptr @global_str.6)
  unreachable

bb150:                                            ; preds = %bb148
  %call6 = call i64 @string_len(ptr %load1)
  store i64 %call6, ptr %sub_len, align 4
  %load7 = load i64, ptr %sub_len, align 4
  %load8 = load i64, ptr %s_len, align 4
  %cmpgt = icmp ugt i64 %load7, %load8
  %zext9 = zext i1 %cmpgt to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb152, label %bb153

bb151:                                            ; preds = %bb148
  call void @panic(ptr @global_str.6)
  unreachable

bb152:                                            ; preds = %bb150
  ret i1 false

bb153:                                            ; preds = %bb150
  %offset = alloca i64, align 8
  store i64 0, ptr %offset, align 4
  %load11 = load i64, ptr %s_len, align 4
  %load12 = load i64, ptr %sub_len, align 4
  %sub = sub i64 %load11, %load12
  store i64 %sub, ptr %offset, align 4
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 4
  store i64 0, ptr %i, align 4
  br label %bb154

bb154:                                            ; preds = %bb168, %bb153
  %load13 = load i64, ptr %i, align 4
  %load14 = load i64, ptr %sub_len, align 4
  %cmplt = icmp slt i64 %load13, %load14
  %zext15 = zext i1 %cmplt to i64
  %cond_true16 = icmp ne i64 %zext15, 0
  br i1 %cond_true16, label %bb155, label %bb156

bb155:                                            ; preds = %bb154
  %load17 = load ptr, ptr %self, align 8
  %ptr2int18 = ptrtoint ptr %load17 to i64
  %cmpne19 = icmp ne i64 %ptr2int18, 0
  %zext20 = zext i1 %cmpne19 to i64
  %cond_true21 = icmp ne i64 %zext20, 0
  br i1 %cond_true21, label %bb157, label %bb158

bb156:                                            ; preds = %bb154
  ret i1 true

bb157:                                            ; preds = %bb155
  %load22 = load i64, ptr %offset, align 4
  %load23 = load i64, ptr %i, align 4
  %add = add i64 %load22, %load23
  %gep = getelementptr i64, ptr %load17, i64 -1
  %load24 = load i64, ptr %gep, align 4
  %cmplt25 = icmp slt i64 %add, %load24
  %zext26 = zext i1 %cmplt25 to i64
  %cond_true27 = icmp ne i64 %zext26, 0
  br i1 %cond_true27, label %bb159, label %bb160

bb158:                                            ; preds = %bb155
  call void @panic(ptr @global_str.5)
  unreachable

bb159:                                            ; preds = %bb157
  %cmpge = icmp sge i64 %add, 0
  %zext28 = zext i1 %cmpge to i64
  %cond_true29 = icmp ne i64 %zext28, 0
  br i1 %cond_true29, label %bb161, label %bb160

bb160:                                            ; preds = %bb159, %bb157
  call void @panic(ptr @global_str.4)
  unreachable

bb161:                                            ; preds = %bb159
  %gep30 = getelementptr i8, ptr %load17, i64 %add
  %load31 = load i8, ptr %gep30, align 1
  %load_sext = sext i8 %load31 to i64
  %load32 = load ptr, ptr %suffix, align 8
  %ptr2int33 = ptrtoint ptr %load32 to i64
  %cmpne34 = icmp ne i64 %ptr2int33, 0
  %zext35 = zext i1 %cmpne34 to i64
  %cond_true36 = icmp ne i64 %zext35, 0
  br i1 %cond_true36, label %bb162, label %bb163

bb162:                                            ; preds = %bb161
  %load37 = load i64, ptr %i, align 4
  %gep38 = getelementptr i64, ptr %load32, i64 -1
  %load39 = load i64, ptr %gep38, align 4
  %cmplt40 = icmp slt i64 %load37, %load39
  %zext41 = zext i1 %cmplt40 to i64
  %cond_true42 = icmp ne i64 %zext41, 0
  br i1 %cond_true42, label %bb164, label %bb165

bb163:                                            ; preds = %bb161
  call void @panic(ptr @global_str.5)
  unreachable

bb164:                                            ; preds = %bb162
  %cmpge43 = icmp sge i64 %load37, 0
  %zext44 = zext i1 %cmpge43 to i64
  %cond_true45 = icmp ne i64 %zext44, 0
  br i1 %cond_true45, label %bb166, label %bb165

bb165:                                            ; preds = %bb164, %bb162
  call void @panic(ptr @global_str.4)
  unreachable

bb166:                                            ; preds = %bb164
  %gep46 = getelementptr i8, ptr %load32, i64 %load37
  %load47 = load i8, ptr %gep46, align 1
  %load_sext48 = sext i8 %load47 to i64
  %cmpne49 = icmp ne i64 %load_sext, %load_sext48
  %zext50 = zext i1 %cmpne49 to i64
  %cond_true51 = icmp ne i64 %zext50, 0
  br i1 %cond_true51, label %bb167, label %bb168

bb167:                                            ; preds = %bb166
  ret i1 false

bb168:                                            ; preds = %bb166
  %load52 = load i64, ptr %i, align 4
  %add53 = add i64 %load52, 1
  store i64 %add53, ptr %i, align 4
  br label %bb154
}

define i1 @string_equals(ptr %0, ptr %1) {
bb169:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %other = alloca ptr, align 8
  store ptr %1, ptr %other, align 8
  %load = load ptr, ptr %self, align 8
  %load1 = load ptr, ptr %other, align 8
  %call = call i64 @strcmp(ptr %load, ptr %load1)
  %cmpeq = icmp eq i64 %call, 0
  %zext = zext i1 %cmpeq to i64
  %ret_trunc = trunc i64 %zext to i1
  ret i1 %ret_trunc
}

define ptr @string_substring(ptr %0, i64 %1, i64 %2) {
bb170:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %start = alloca i64, align 8
  store i64 %1, ptr %start, align 4
  %end = alloca i64, align 8
  store i64 %2, ptr %end, align 4
  %s_len = alloca i64, align 8
  store i64 0, ptr %s_len, align 4
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb171, label %bb172

bb171:                                            ; preds = %bb170
  %call = call i64 @string_len(ptr %load)
  store i64 %call, ptr %s_len, align 4
  %safe_end = alloca i64, align 8
  store i64 0, ptr %safe_end, align 4
  %load1 = load i64, ptr %end, align 4
  store i64 %load1, ptr %safe_end, align 4
  %load2 = load i64, ptr %safe_end, align 4
  %load3 = load i64, ptr %s_len, align 4
  %cmpgt = icmp ugt i64 %load2, %load3
  %zext4 = zext i1 %cmpgt to i64
  %cond_true5 = icmp ne i64 %zext4, 0
  br i1 %cond_true5, label %bb173, label %bb174

bb172:                                            ; preds = %bb170
  call void @panic(ptr @global_str.6)
  unreachable

bb173:                                            ; preds = %bb171
  %load6 = load i64, ptr %s_len, align 4
  store i64 %load6, ptr %safe_end, align 4
  br label %bb174

bb174:                                            ; preds = %bb173, %bb171
  %load7 = load i64, ptr %start, align 4
  %load8 = load i64, ptr %safe_end, align 4
  %cmpge = icmp uge i64 %load7, %load8
  %zext9 = zext i1 %cmpge to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb175, label %bb176

bb175:                                            ; preds = %bb174
  %empty = alloca ptr, align 8
  store ptr null, ptr %empty, align 8
  %call11 = call ptr @malloc(i64 1)
  store ptr %call11, ptr %empty, align 8
  %load12 = load ptr, ptr %empty, align 8
  %ptr2int13 = ptrtoint ptr %load12 to i64
  %cmpne14 = icmp ne i64 %ptr2int13, 0
  %zext15 = zext i1 %cmpne14 to i64
  %cond_true16 = icmp ne i64 %zext15, 0
  br i1 %cond_true16, label %bb177, label %bb178

bb176:                                            ; preds = %bb174
  %len = alloca i64, align 8
  store i64 0, ptr %len, align 4
  %load17 = load i64, ptr %safe_end, align 4
  %load18 = load i64, ptr %start, align 4
  %sub = sub i64 %load17, %load18
  store i64 %sub, ptr %len, align 4
  %dest = alloca ptr, align 8
  store ptr null, ptr %dest, align 8
  %load19 = load i64, ptr %len, align 4
  %add = add i64 %load19, 1
  %call20 = call ptr @malloc(i64 %add)
  store ptr %call20, ptr %dest, align 8
  %src_offset = alloca ptr, align 8
  store ptr null, ptr %src_offset, align 8
  %load21 = load ptr, ptr %self, align 8
  %ptrtoint = ptrtoint ptr %load21 to i64
  %load22 = load i64, ptr %start, align 4
  %add23 = add i64 %ptrtoint, %load22
  %inttoptr = inttoptr i64 %add23 to ptr
  store ptr %inttoptr, ptr %src_offset, align 8
  %load24 = load ptr, ptr %dest, align 8
  %load25 = load ptr, ptr %src_offset, align 8
  %load26 = load i64, ptr %len, align 4
  %call27 = call ptr @strncpy(ptr %load24, ptr %load25, i64 %load26)
  %load28 = load ptr, ptr %dest, align 8
  %ptr2int29 = ptrtoint ptr %load28 to i64
  %cmpne30 = icmp ne i64 %ptr2int29, 0
  %zext31 = zext i1 %cmpne30 to i64
  %cond_true32 = icmp ne i64 %zext31, 0
  br i1 %cond_true32, label %bb182, label %bb183

bb177:                                            ; preds = %bb175
  %gep = getelementptr i64, ptr %load12, i64 -1
  %load33 = load i64, ptr %gep, align 4
  %cmplt = icmp slt i64 0, %load33
  %zext34 = zext i1 %cmplt to i64
  %cond_true35 = icmp ne i64 %zext34, 0
  br i1 %cond_true35, label %bb179, label %bb180

bb178:                                            ; preds = %bb175
  call void @panic(ptr @global_str.3)
  unreachable

bb179:                                            ; preds = %bb177
  br i1 true, label %bb181, label %bb180

bb180:                                            ; preds = %bb179, %bb177
  call void @panic(ptr @global_str.4)
  unreachable

bb181:                                            ; preds = %bb179
  %gep36 = getelementptr i8, ptr %load12, i64 0
  store i8 0, ptr %gep36, align 1
  %load37 = load ptr, ptr %empty, align 8
  ret ptr %load37

bb182:                                            ; preds = %bb176
  %load38 = load i64, ptr %len, align 4
  %gep39 = getelementptr i64, ptr %load28, i64 -1
  %load40 = load i64, ptr %gep39, align 4
  %cmplt41 = icmp slt i64 %load38, %load40
  %zext42 = zext i1 %cmplt41 to i64
  %cond_true43 = icmp ne i64 %zext42, 0
  br i1 %cond_true43, label %bb184, label %bb185

bb183:                                            ; preds = %bb176
  call void @panic(ptr @global_str.3)
  unreachable

bb184:                                            ; preds = %bb182
  %cmpge44 = icmp sge i64 %load38, 0
  %zext45 = zext i1 %cmpge44 to i64
  %cond_true46 = icmp ne i64 %zext45, 0
  br i1 %cond_true46, label %bb186, label %bb185

bb185:                                            ; preds = %bb184, %bb182
  call void @panic(ptr @global_str.4)
  unreachable

bb186:                                            ; preds = %bb184
  %gep47 = getelementptr i8, ptr %load28, i64 %load38
  store i8 0, ptr %gep47, align 1
  %load48 = load ptr, ptr %dest, align 8
  ret ptr %load48
}

define i64 @string_split(ptr %0, ptr %1) {
bb187:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %delim = alloca ptr, align 8
  store ptr %1, ptr %delim, align 8
  %result = alloca ptr, align 8
  store ptr null, ptr %result, align 8
  %struct_alloc = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 24, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %call = call ptr @malloc(i64 128)
  %gep = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 0
  store ptr %call, ptr %gep, align 8
  %gep1 = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 1
  store i64 0, ptr %gep1, align 4
  %gep2 = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 2
  store i64 10, ptr %gep2, align 4
  store ptr %data_ptr, ptr %result, align 8
  %s_len = alloca i64, align 8
  store i64 0, ptr %s_len, align 4
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb188, label %bb189

bb188:                                            ; preds = %bb187
  %call3 = call i64 @string_len(ptr %load)
  store i64 %call3, ptr %s_len, align 4
  %d_len = alloca i64, align 8
  store i64 0, ptr %d_len, align 4
  %load4 = load ptr, ptr %delim, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb190, label %bb191

bb189:                                            ; preds = %bb187
  call void @panic(ptr @global_str.6)
  unreachable

bb190:                                            ; preds = %bb188
  %call9 = call i64 @string_len(ptr %load4)
  store i64 %call9, ptr %d_len, align 4
  %load10 = load i64, ptr %d_len, align 4
  %cmpeq = icmp eq i64 %load10, 0
  %zext11 = zext i1 %cmpeq to i64
  %cond_true12 = icmp ne i64 %zext11, 0
  br i1 %cond_true12, label %bb192, label %bb193

bb191:                                            ; preds = %bb188
  call void @panic(ptr @global_str.6)
  unreachable

bb192:                                            ; preds = %bb190
  %load13 = load ptr, ptr %self, align 8
  %load14 = load ptr, ptr %result, align 8
  %ptr2int15 = ptrtoint ptr %load14 to i64
  %cmpne16 = icmp ne i64 %ptr2int15, 0
  %zext17 = zext i1 %cmpne16 to i64
  %cond_true18 = icmp ne i64 %zext17, 0
  br i1 %cond_true18, label %bb194, label %bb195

bb193:                                            ; preds = %bb190
  %start = alloca i64, align 8
  store i64 0, ptr %start, align 4
  store i64 0, ptr %start, align 4
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 4
  store i64 0, ptr %i, align 4
  br label %bb198

bb194:                                            ; preds = %bb192
  call void @Vec_str_push(ptr %load14, ptr %load13)
  %load19 = load ptr, ptr %result, align 8
  %load20 = load ptr, ptr %result, align 8
  %ptr2int21 = ptrtoint ptr %load20 to i64
  %cmpne22 = icmp ne i64 %ptr2int21, 0
  %zext23 = zext i1 %cmpne22 to i64
  %cond_true24 = icmp ne i64 %zext23, 0
  br i1 %cond_true24, label %bb196, label %bb197

bb195:                                            ; preds = %bb192
  call void @panic(ptr @global_str.6)
  unreachable

bb196:                                            ; preds = %bb194
  %is_not_null = icmp ne ptr %load20, null
  br i1 %is_not_null, label %arc.release.do, label %arc.release.cont

bb197:                                            ; preds = %arc.release.cont, %bb194
  %ret_cast_int = ptrtoint ptr %load19 to i64
  ret i64 %ret_cast_int

bb198:                                            ; preds = %bb217, %bb193
  %load25 = load i64, ptr %i, align 4
  %load26 = load i64, ptr %d_len, align 4
  %add = add i64 %load25, %load26
  %load27 = load i64, ptr %s_len, align 4
  %cmple = icmp ule i64 %add, %load27
  %zext28 = zext i1 %cmple to i64
  %cond_true29 = icmp ne i64 %zext28, 0
  br i1 %cond_true29, label %bb199, label %bb200

bb199:                                            ; preds = %bb198
  %match_found = alloca i1, align 1
  store i1 false, ptr %match_found, align 1
  store i1 true, ptr %match_found, align 1
  %j = alloca i64, align 8
  store i64 0, ptr %j, align 4
  store i64 0, ptr %j, align 4
  br label %bb201

bb200:                                            ; preds = %bb198
  %load30 = load i64, ptr %start, align 4
  %load31 = load i64, ptr %s_len, align 4
  %load32 = load ptr, ptr %self, align 8
  %ptr2int33 = ptrtoint ptr %load32 to i64
  %cmpne34 = icmp ne i64 %ptr2int33, 0
  %zext35 = zext i1 %cmpne34 to i64
  %cond_true36 = icmp ne i64 %zext35, 0
  br i1 %cond_true36, label %bb223, label %bb224

bb201:                                            ; preds = %bb215, %bb199
  %load37 = load i64, ptr %j, align 4
  %load38 = load i64, ptr %d_len, align 4
  %cmplt = icmp ult i64 %load37, %load38
  %zext39 = zext i1 %cmplt to i64
  %cond_true40 = icmp ne i64 %zext39, 0
  br i1 %cond_true40, label %bb202, label %bb203

bb202:                                            ; preds = %bb201
  %load41 = load ptr, ptr %self, align 8
  %ptr2int42 = ptrtoint ptr %load41 to i64
  %cmpne43 = icmp ne i64 %ptr2int42, 0
  %zext44 = zext i1 %cmpne43 to i64
  %cond_true45 = icmp ne i64 %zext44, 0
  br i1 %cond_true45, label %bb204, label %bb205

bb203:                                            ; preds = %bb201
  %load46 = load i1, ptr %match_found, align 1
  %load_zext = zext i1 %load46 to i64
  %cond_true47 = icmp ne i64 %load_zext, 0
  br i1 %cond_true47, label %bb216, label %bb218

bb204:                                            ; preds = %bb202
  %load48 = load i64, ptr %i, align 4
  %load49 = load i64, ptr %j, align 4
  %add50 = add i64 %load48, %load49
  %gep51 = getelementptr i64, ptr %load41, i64 -1
  %load52 = load i64, ptr %gep51, align 4
  %cmplt53 = icmp slt i64 %add50, %load52
  %zext54 = zext i1 %cmplt53 to i64
  %cond_true55 = icmp ne i64 %zext54, 0
  br i1 %cond_true55, label %bb206, label %bb207

bb205:                                            ; preds = %bb202
  call void @panic(ptr @global_str.5)
  unreachable

bb206:                                            ; preds = %bb204
  %cmpge = icmp sge i64 %add50, 0
  %zext56 = zext i1 %cmpge to i64
  %cond_true57 = icmp ne i64 %zext56, 0
  br i1 %cond_true57, label %bb208, label %bb207

bb207:                                            ; preds = %bb206, %bb204
  call void @panic(ptr @global_str.4)
  unreachable

bb208:                                            ; preds = %bb206
  %gep58 = getelementptr i8, ptr %load41, i64 %add50
  %load59 = load i8, ptr %gep58, align 1
  %load_sext = sext i8 %load59 to i64
  %load60 = load ptr, ptr %delim, align 8
  %ptr2int61 = ptrtoint ptr %load60 to i64
  %cmpne62 = icmp ne i64 %ptr2int61, 0
  %zext63 = zext i1 %cmpne62 to i64
  %cond_true64 = icmp ne i64 %zext63, 0
  br i1 %cond_true64, label %bb209, label %bb210

bb209:                                            ; preds = %bb208
  %load65 = load i64, ptr %j, align 4
  %gep66 = getelementptr i64, ptr %load60, i64 -1
  %load67 = load i64, ptr %gep66, align 4
  %cmplt68 = icmp slt i64 %load65, %load67
  %zext69 = zext i1 %cmplt68 to i64
  %cond_true70 = icmp ne i64 %zext69, 0
  br i1 %cond_true70, label %bb211, label %bb212

bb210:                                            ; preds = %bb208
  call void @panic(ptr @global_str.5)
  unreachable

bb211:                                            ; preds = %bb209
  %cmpge71 = icmp sge i64 %load65, 0
  %zext72 = zext i1 %cmpge71 to i64
  %cond_true73 = icmp ne i64 %zext72, 0
  br i1 %cond_true73, label %bb213, label %bb212

bb212:                                            ; preds = %bb211, %bb209
  call void @panic(ptr @global_str.4)
  unreachable

bb213:                                            ; preds = %bb211
  %gep74 = getelementptr i8, ptr %load60, i64 %load65
  %load75 = load i8, ptr %gep74, align 1
  %load_sext76 = sext i8 %load75 to i64
  %cmpne77 = icmp ne i64 %load_sext, %load_sext76
  %zext78 = zext i1 %cmpne77 to i64
  %cond_true79 = icmp ne i64 %zext78, 0
  br i1 %cond_true79, label %bb214, label %bb215

bb214:                                            ; preds = %bb213
  store i1 false, ptr %match_found, align 1
  br label %bb215

bb215:                                            ; preds = %bb214, %bb213
  %load80 = load i64, ptr %j, align 4
  %add81 = add i64 %load80, 1
  store i64 %add81, ptr %j, align 4
  br label %bb201

bb216:                                            ; preds = %bb203
  %load82 = load i64, ptr %start, align 4
  %load83 = load i64, ptr %i, align 4
  %load84 = load ptr, ptr %self, align 8
  %ptr2int85 = ptrtoint ptr %load84 to i64
  %cmpne86 = icmp ne i64 %ptr2int85, 0
  %zext87 = zext i1 %cmpne86 to i64
  %cond_true88 = icmp ne i64 %zext87, 0
  br i1 %cond_true88, label %bb219, label %bb220

bb217:                                            ; preds = %bb221, %bb218
  br label %bb198

bb218:                                            ; preds = %bb203
  %load89 = load i64, ptr %i, align 4
  %add90 = add i64 %load89, 1
  store i64 %add90, ptr %i, align 4
  br label %bb217

bb219:                                            ; preds = %bb216
  %call91 = call ptr @string_substring(ptr %load84, i64 %load82, i64 %load83)
  %load92 = load ptr, ptr %result, align 8
  %ptr2int93 = ptrtoint ptr %load92 to i64
  %cmpne94 = icmp ne i64 %ptr2int93, 0
  %zext95 = zext i1 %cmpne94 to i64
  %cond_true96 = icmp ne i64 %zext95, 0
  br i1 %cond_true96, label %bb221, label %bb222

bb220:                                            ; preds = %bb216
  call void @panic(ptr @global_str.6)
  unreachable

bb221:                                            ; preds = %bb219
  call void @Vec_str_push(ptr %load92, ptr %call91)
  %load97 = load i64, ptr %i, align 4
  %load98 = load i64, ptr %d_len, align 4
  %add99 = add i64 %load97, %load98
  store i64 %add99, ptr %i, align 4
  %load100 = load i64, ptr %i, align 4
  store i64 %load100, ptr %start, align 4
  br label %bb217

bb222:                                            ; preds = %bb219
  call void @panic(ptr @global_str.6)
  unreachable

bb223:                                            ; preds = %bb200
  %call101 = call ptr @string_substring(ptr %load32, i64 %load30, i64 %load31)
  %load102 = load ptr, ptr %result, align 8
  %ptr2int103 = ptrtoint ptr %load102 to i64
  %cmpne104 = icmp ne i64 %ptr2int103, 0
  %zext105 = zext i1 %cmpne104 to i64
  %cond_true106 = icmp ne i64 %zext105, 0
  br i1 %cond_true106, label %bb225, label %bb226

bb224:                                            ; preds = %bb200
  call void @panic(ptr @global_str.6)
  unreachable

bb225:                                            ; preds = %bb223
  call void @Vec_str_push(ptr %load102, ptr %call101)
  %load107 = load ptr, ptr %result, align 8
  %load108 = load ptr, ptr %result, align 8
  %ptr2int109 = ptrtoint ptr %load108 to i64
  %cmpne110 = icmp ne i64 %ptr2int109, 0
  %zext111 = zext i1 %cmpne110 to i64
  %cond_true112 = icmp ne i64 %zext111, 0
  br i1 %cond_true112, label %bb227, label %bb228

bb226:                                            ; preds = %bb223
  call void @panic(ptr @global_str.6)
  unreachable

bb227:                                            ; preds = %bb225
  %is_not_null113 = icmp ne ptr %load108, null
  br i1 %is_not_null113, label %arc.release.do114, label %arc.release.cont115

bb228:                                            ; preds = %arc.release.cont115, %bb225
  %ret_cast_int122 = ptrtoint ptr %load107 to i64
  ret i64 %ret_cast_int122

arc.release.do:                                   ; preds = %bb196
  %ref_ptr = getelementptr i64, ptr %load20, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = sub i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  %is_zero = icmp eq i64 %new_count, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb196
  br label %bb197

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont

arc.release.do114:                                ; preds = %bb227
  %ref_ptr116 = getelementptr i64, ptr %load108, i64 -2
  %current_count117 = load i64, ptr %ref_ptr116, align 4
  %new_count118 = sub i64 %current_count117, 1
  store i64 %new_count118, ptr %ref_ptr116, align 4
  %is_zero119 = icmp eq i64 %new_count118, 0
  br i1 %is_zero119, label %arc.free120, label %arc.end121

arc.release.cont115:                              ; preds = %arc.end121, %bb227
  br label %bb228

arc.free120:                                      ; preds = %arc.release.do114
  call void @free(ptr %ref_ptr116)
  br label %arc.end121

arc.end121:                                       ; preds = %arc.free120, %arc.release.do114
  br label %arc.release.cont115
}

define ptr @string_concat(ptr %0, ptr %1) {
bb229:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %other = alloca ptr, align 8
  store ptr %1, ptr %other, align 8
  %len1 = alloca i64, align 8
  store i64 0, ptr %len1, align 4
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb230, label %bb231

bb230:                                            ; preds = %bb229
  %call = call i64 @string_len(ptr %load)
  store i64 %call, ptr %len1, align 4
  %len2 = alloca i64, align 8
  store i64 0, ptr %len2, align 4
  %load1 = load ptr, ptr %other, align 8
  %ptr2int2 = ptrtoint ptr %load1 to i64
  %cmpne3 = icmp ne i64 %ptr2int2, 0
  %zext4 = zext i1 %cmpne3 to i64
  %cond_true5 = icmp ne i64 %zext4, 0
  br i1 %cond_true5, label %bb232, label %bb233

bb231:                                            ; preds = %bb229
  call void @panic(ptr @global_str.6)
  unreachable

bb232:                                            ; preds = %bb230
  %call6 = call i64 @string_len(ptr %load1)
  store i64 %call6, ptr %len2, align 4
  %new_str = alloca ptr, align 8
  store ptr null, ptr %new_str, align 8
  %load7 = load i64, ptr %len1, align 4
  %load8 = load i64, ptr %len2, align 4
  %add = add i64 %load7, %load8
  %add9 = add i64 %add, 1
  %call10 = call ptr @malloc(i64 %add9)
  store ptr %call10, ptr %new_str, align 8
  %load11 = load ptr, ptr %new_str, align 8
  %load12 = load ptr, ptr %self, align 8
  %load13 = load i64, ptr %len1, align 4
  %call14 = call ptr @strncpy(ptr %load11, ptr %load12, i64 %load13)
  %offset_ptr = alloca ptr, align 8
  store ptr null, ptr %offset_ptr, align 8
  %load15 = load ptr, ptr %new_str, align 8
  %ptrtoint = ptrtoint ptr %load15 to i64
  %load16 = load i64, ptr %len1, align 4
  %add17 = add i64 %ptrtoint, %load16
  %inttoptr = inttoptr i64 %add17 to ptr
  store ptr %inttoptr, ptr %offset_ptr, align 8
  %load18 = load ptr, ptr %offset_ptr, align 8
  %load19 = load ptr, ptr %other, align 8
  %load20 = load i64, ptr %len2, align 4
  %call21 = call ptr @strncpy(ptr %load18, ptr %load19, i64 %load20)
  %load22 = load ptr, ptr %new_str, align 8
  %ptr2int23 = ptrtoint ptr %load22 to i64
  %cmpne24 = icmp ne i64 %ptr2int23, 0
  %zext25 = zext i1 %cmpne24 to i64
  %cond_true26 = icmp ne i64 %zext25, 0
  br i1 %cond_true26, label %bb234, label %bb235

bb233:                                            ; preds = %bb230
  call void @panic(ptr @global_str.6)
  unreachable

bb234:                                            ; preds = %bb232
  %load27 = load i64, ptr %len1, align 4
  %load28 = load i64, ptr %len2, align 4
  %add29 = add i64 %load27, %load28
  %gep = getelementptr i64, ptr %load22, i64 -1
  %load30 = load i64, ptr %gep, align 4
  %cmplt = icmp slt i64 %add29, %load30
  %zext31 = zext i1 %cmplt to i64
  %cond_true32 = icmp ne i64 %zext31, 0
  br i1 %cond_true32, label %bb236, label %bb237

bb235:                                            ; preds = %bb232
  call void @panic(ptr @global_str.3)
  unreachable

bb236:                                            ; preds = %bb234
  %cmpge = icmp sge i64 %add29, 0
  %zext33 = zext i1 %cmpge to i64
  %cond_true34 = icmp ne i64 %zext33, 0
  br i1 %cond_true34, label %bb238, label %bb237

bb237:                                            ; preds = %bb236, %bb234
  call void @panic(ptr @global_str.4)
  unreachable

bb238:                                            ; preds = %bb236
  %gep35 = getelementptr i8, ptr %load22, i64 %add29
  store i8 0, ptr %gep35, align 1
  %load36 = load ptr, ptr %new_str, align 8
  ret ptr %load36
}

define i64 @main() {
bb239:
  %point3 = alloca ptr, align 8
  store ptr null, ptr %point3, align 8
  %struct_alloc = call ptr @malloc(i64 28)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 12, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %gep = getelementptr %Point3_f32, ptr %data_ptr, i64 0, i32 0
  store float 0x40019999A0000000, ptr %gep, align 4
  %gep1 = getelementptr %Point3_f32, ptr %data_ptr, i64 0, i32 1
  store float 0x40119999A0000000, ptr %gep1, align 4
  %gep2 = getelementptr %Point3_f32, ptr %data_ptr, i64 0, i32 2
  store float 0x401A666660000000, ptr %gep2, align 4
  store ptr %data_ptr, ptr %point3, align 8
  %load = load ptr, ptr %point3, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb240, label %bb241

bb240:                                            ; preds = %bb239
  %call = call float @Point3_f32_add3(ptr %load)
  %vararg_fpext = fpext float %call to double
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.7, double %vararg_fpext)
  %load4 = load ptr, ptr %point3, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb242, label %bb243

bb241:                                            ; preds = %bb239
  call void @panic(ptr @global_str.6)
  unreachable

bb242:                                            ; preds = %bb240
  %call9 = call float @Point3_f32_getX(ptr %load4)
  %vararg_fpext10 = fpext float %call9 to double
  %call11 = call i32 (ptr, ...) @printf(ptr @global_str.7, double %vararg_fpext10)
  %load12 = load ptr, ptr %point3, align 8
  %ptr2int13 = ptrtoint ptr %load12 to i64
  %cmpne14 = icmp ne i64 %ptr2int13, 0
  %zext15 = zext i1 %cmpne14 to i64
  %cond_true16 = icmp ne i64 %zext15, 0
  br i1 %cond_true16, label %bb244, label %bb245

bb243:                                            ; preds = %bb240
  call void @panic(ptr @global_str.6)
  unreachable

bb244:                                            ; preds = %bb242
  %call17 = call float @Point3_f32_getY(ptr %load12)
  %vararg_fpext18 = fpext float %call17 to double
  %call19 = call i32 (ptr, ...) @printf(ptr @global_str.7, double %vararg_fpext18)
  %load20 = load ptr, ptr %point3, align 8
  %ptr2int21 = ptrtoint ptr %load20 to i64
  %cmpne22 = icmp ne i64 %ptr2int21, 0
  %zext23 = zext i1 %cmpne22 to i64
  %cond_true24 = icmp ne i64 %zext23, 0
  br i1 %cond_true24, label %bb246, label %bb247

bb245:                                            ; preds = %bb242
  call void @panic(ptr @global_str.6)
  unreachable

bb246:                                            ; preds = %bb244
  %call25 = call float @Point3_f32_getZ(ptr %load20)
  %vararg_fpext26 = fpext float %call25 to double
  %call27 = call i32 (ptr, ...) @printf(ptr @global_str.7, double %vararg_fpext26)
  %call28 = call i32 (ptr, ...) @printf(ptr @global_str.7, double 0x3FF1980000000000)
  %call29 = call i32 (ptr, ...) @printf(ptr @global_str.7, double 0x3FF19999A0000000)
  %call30 = call i32 (ptr, ...) @printf(ptr @global_str.7, double 1.100000e+00)
  %a = alloca i64, align 8
  store i64 0, ptr %a, align 4
  store i64 1, ptr %a, align 4
  %call31 = call half @f16_min()
  %vararg_fpext32 = fpext half %call31 to double
  %call33 = call i32 (ptr, ...) @printf(ptr @global_str.7, double %vararg_fpext32)
  %str1 = alloca ptr, align 8
  store ptr null, ptr %str1, align 8
  store ptr @global_str.8, ptr %str1, align 8
  %load34 = load ptr, ptr %str1, align 8
  %ptr2int35 = ptrtoint ptr %load34 to i64
  %cmpne36 = icmp ne i64 %ptr2int35, 0
  %zext37 = zext i1 %cmpne36 to i64
  %cond_true38 = icmp ne i64 %zext37, 0
  br i1 %cond_true38, label %bb248, label %bb249

bb247:                                            ; preds = %bb244
  call void @panic(ptr @global_str.6)
  unreachable

bb248:                                            ; preds = %bb246
  %call39 = call i64 @string_len(ptr %load34)
  %call40 = call i32 (ptr, ...) @printf(ptr @global_str.9, i64 %call39)
  %load41 = load ptr, ptr %str1, align 8
  %ptr2int42 = ptrtoint ptr %load41 to i64
  %cmpne43 = icmp ne i64 %ptr2int42, 0
  %zext44 = zext i1 %cmpne43 to i64
  %cond_true45 = icmp ne i64 %zext44, 0
  br i1 %cond_true45, label %bb250, label %bb251

bb249:                                            ; preds = %bb246
  call void @panic(ptr @global_str.6)
  unreachable

bb250:                                            ; preds = %bb248
  %call46 = call i1 @string_is_empty(ptr %load41)
  %call47 = call i32 (ptr, ...) @printf(ptr @global_str.9, i1 %call46)
  %load48 = load ptr, ptr %str1, align 8
  %ptr2int49 = ptrtoint ptr %load48 to i64
  %cmpne50 = icmp ne i64 %ptr2int49, 0
  %zext51 = zext i1 %cmpne50 to i64
  %cond_true52 = icmp ne i64 %zext51, 0
  br i1 %cond_true52, label %bb252, label %bb253

bb251:                                            ; preds = %bb248
  call void @panic(ptr @global_str.6)
  unreachable

bb252:                                            ; preds = %bb250
  %call53 = call i1 @string_contains(ptr %load48, ptr @global_str.10)
  %call54 = call i32 (ptr, ...) @printf(ptr @global_str.9, i1 %call53)
  %load55 = load ptr, ptr %str1, align 8
  %ptr2int56 = ptrtoint ptr %load55 to i64
  %cmpne57 = icmp ne i64 %ptr2int56, 0
  %zext58 = zext i1 %cmpne57 to i64
  %cond_true59 = icmp ne i64 %zext58, 0
  br i1 %cond_true59, label %bb254, label %bb255

bb253:                                            ; preds = %bb250
  call void @panic(ptr @global_str.6)
  unreachable

bb254:                                            ; preds = %bb252
  %call60 = call i1 @string_starts_with(ptr %load55, ptr @global_str.11)
  %call61 = call i32 (ptr, ...) @printf(ptr @global_str.9, i1 %call60)
  %load62 = load ptr, ptr %str1, align 8
  %ptr2int63 = ptrtoint ptr %load62 to i64
  %cmpne64 = icmp ne i64 %ptr2int63, 0
  %zext65 = zext i1 %cmpne64 to i64
  %cond_true66 = icmp ne i64 %zext65, 0
  br i1 %cond_true66, label %bb256, label %bb257

bb255:                                            ; preds = %bb252
  call void @panic(ptr @global_str.6)
  unreachable

bb256:                                            ; preds = %bb254
  %call67 = call i1 @string_ends_with(ptr %load62, ptr @global_str.12)
  %call68 = call i32 (ptr, ...) @printf(ptr @global_str.9, i1 %call67)
  %load69 = load ptr, ptr %str1, align 8
  %load70 = load ptr, ptr %str1, align 8
  %ptr2int71 = ptrtoint ptr %load70 to i64
  %cmpne72 = icmp ne i64 %ptr2int71, 0
  %zext73 = zext i1 %cmpne72 to i64
  %cond_true74 = icmp ne i64 %zext73, 0
  br i1 %cond_true74, label %bb258, label %bb259

bb257:                                            ; preds = %bb254
  call void @panic(ptr @global_str.6)
  unreachable

bb258:                                            ; preds = %bb256
  %call75 = call i1 @string_equals(ptr %load70, ptr %load69)
  %call76 = call i32 (ptr, ...) @printf(ptr @global_str.9, i1 %call75)
  %load77 = load ptr, ptr %str1, align 8
  %ptr2int78 = ptrtoint ptr %load77 to i64
  %cmpne79 = icmp ne i64 %ptr2int78, 0
  %zext80 = zext i1 %cmpne79 to i64
  %cond_true81 = icmp ne i64 %zext80, 0
  br i1 %cond_true81, label %bb260, label %bb261

bb259:                                            ; preds = %bb256
  call void @panic(ptr @global_str.6)
  unreachable

bb260:                                            ; preds = %bb258
  %call82 = call ptr @string_substring(ptr %load77, i64 0, i64 3)
  %call83 = call i32 (ptr, ...) @printf(ptr @global_str.13, ptr %call82)
  %n1 = alloca i8, align 1
  store i8 0, ptr %n1, align 1
  store i8 -2, ptr %n1, align 1
  %n2 = alloca i8, align 1
  store i8 0, ptr %n2, align 1
  store i8 1, ptr %n2, align 1
  %load84 = load i8, ptr %n1, align 1
  %load_zext = zext i8 %load84 to i64
  %load85 = load i8, ptr %n2, align 1
  %load_zext86 = zext i8 %load85 to i64
  %add = add i64 %load_zext, %load_zext86
  %call87 = call i32 (ptr, ...) @printf(ptr @global_str.9, i64 %add)
  %load88 = load ptr, ptr %str1, align 8
  %load89 = load ptr, ptr %str1, align 8
  %ptr2int90 = ptrtoint ptr %load89 to i64
  %cmpne91 = icmp ne i64 %ptr2int90, 0
  %zext92 = zext i1 %cmpne91 to i64
  %cond_true93 = icmp ne i64 %zext92, 0
  br i1 %cond_true93, label %bb262, label %bb263

bb261:                                            ; preds = %bb258
  call void @panic(ptr @global_str.6)
  unreachable

bb262:                                            ; preds = %bb260
  %call94 = call ptr @string_concat(ptr %load89, ptr %load88)
  %call95 = call i32 (ptr, ...) @printf(ptr @global_str.13, ptr %call94)
  %call96 = call i32 (ptr, ...) @printf(ptr @global_str.14)
  %stdin = alloca ptr, align 8
  store ptr null, ptr %stdin, align 8
  %call97 = call ptr @read_line(i64 128)
  store ptr %call97, ptr %stdin, align 8
  %fruits = alloca i64, align 8
  store i64 0, ptr %fruits, align 4
  %load98 = load ptr, ptr %stdin, align 8
  %ptr2int99 = ptrtoint ptr %load98 to i64
  %cmpne100 = icmp ne i64 %ptr2int99, 0
  %zext101 = zext i1 %cmpne100 to i64
  %cond_true102 = icmp ne i64 %zext101, 0
  br i1 %cond_true102, label %bb264, label %bb265

bb263:                                            ; preds = %bb260
  call void @panic(ptr @global_str.6)
  unreachable

bb264:                                            ; preds = %bb262
  %call103 = call i64 @string_split(ptr %load98, ptr @global_str.15)
  store i64 %call103, ptr %fruits, align 4
  %call104 = call i32 (ptr, ...) @printf(ptr @global_str.16)
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 4
  store i64 0, ptr %i, align 4
  br label %bb266

bb265:                                            ; preds = %bb262
  call void @panic(ptr @global_str.6)
  unreachable

bb266:                                            ; preds = %bb271, %bb264
  %load105 = load i64, ptr %i, align 4
  %load106 = load i64, ptr %fruits, align 4
  %cmpne107 = icmp ne i64 %load106, 0
  %zext108 = zext i1 %cmpne107 to i64
  %cond_true109 = icmp ne i64 %zext108, 0
  br i1 %cond_true109, label %bb269, label %bb270

bb267:                                            ; preds = %bb269
  %load110 = load i64, ptr %i, align 4
  %load111 = load i64, ptr %fruits, align 4
  %cmpne112 = icmp ne i64 %load111, 0
  %zext113 = zext i1 %cmpne112 to i64
  %cond_true114 = icmp ne i64 %zext113, 0
  br i1 %cond_true114, label %bb271, label %bb272

bb268:                                            ; preds = %bb269
  %load115 = load i64, ptr %fruits, align 4
  %cmpne116 = icmp ne i64 %load115, 0
  %zext117 = zext i1 %cmpne116 to i64
  %cond_true118 = icmp ne i64 %zext117, 0
  br i1 %cond_true118, label %bb273, label %bb274

bb269:                                            ; preds = %bb266
  %auto_cast_ptr = inttoptr i64 %load106 to ptr
  %call119 = call i64 @Vec_str_length(ptr %auto_cast_ptr)
  %cmplt = icmp slt i64 %load105, %call119
  %zext120 = zext i1 %cmplt to i64
  %cond_true121 = icmp ne i64 %zext120, 0
  br i1 %cond_true121, label %bb267, label %bb268

bb270:                                            ; preds = %bb266
  call void @panic(ptr @global_str.6)
  unreachable

bb271:                                            ; preds = %bb267
  %auto_cast_ptr122 = inttoptr i64 %load111 to ptr
  %call123 = call ptr @Vec_str_get(ptr %auto_cast_ptr122, i64 %load110)
  %call124 = call i32 (ptr, ...) @printf(ptr @global_str.17, ptr %call123)
  %load125 = load i64, ptr %i, align 4
  %add126 = add i64 %load125, 1
  store i64 %add126, ptr %i, align 4
  br label %bb266

bb272:                                            ; preds = %bb267
  call void @panic(ptr @global_str.6)
  unreachable

bb273:                                            ; preds = %bb268
  %auto_cast_ptr127 = inttoptr i64 %load115 to ptr
  call void @Vec_str_drop(ptr %auto_cast_ptr127)
  %load128 = load ptr, ptr %stdin, align 8
  call void @free(ptr %load128)
  store ptr null, ptr %stdin, align 8
  %load129 = load ptr, ptr %point3, align 8
  %ptr2int130 = ptrtoint ptr %load129 to i64
  %cmpne131 = icmp ne i64 %ptr2int130, 0
  %zext132 = zext i1 %cmpne131 to i64
  %cond_true133 = icmp ne i64 %zext132, 0
  br i1 %cond_true133, label %bb275, label %bb276

bb274:                                            ; preds = %bb268
  call void @panic(ptr @global_str.6)
  unreachable

bb275:                                            ; preds = %bb273
  %is_not_null = icmp ne ptr %load129, null
  br i1 %is_not_null, label %arc.release.do, label %arc.release.cont

bb276:                                            ; preds = %arc.release.cont, %bb273
  ret i64 0

arc.release.do:                                   ; preds = %bb275
  %ref_ptr = getelementptr i64, ptr %load129, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = sub i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  %is_zero = icmp eq i64 %new_count, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb275
  br label %bb276

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define i64 @Result_T_E_Ok(i64 %0) {
bb277:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 0, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Result_T_E_Err(i64 %0) {
bb278:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 1, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Option_T_Some(i64 %0) {
bb279:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 0, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Option_T_None() {
bb280:
  %arr_alloc = call ptr @malloc(i64 24)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 1, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 1, ptr %gep, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Result_T_str_Ok(i64 %0) {
bb281:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 0, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Result_T_str_Err(i64 %0) {
bb282:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 1, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i1 @Result_T_str_is_ok(ptr %0) {
bb283:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i1, align 1
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb285, label %bb286

bb284:                                            ; preds = %bb288, %bb287, %bb285
  %load2 = load i1, ptr %match_res, align 1
  %load_zext = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %load_zext to i1
  ret i1 %ret_trunc

bb285:                                            ; preds = %bb283
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb284

bb286:                                            ; preds = %bb283
  %cmpeq5 = icmp eq i64 %load1, 1
  %zext6 = zext i1 %cmpeq5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb287, label %bb288

bb287:                                            ; preds = %bb286
  %gep8 = getelementptr i64, ptr %load, i64 1
  %load9 = load i64, ptr %gep8, align 4
  %e = alloca i64, align 8
  store i64 %load9, ptr %e, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb284

bb288:                                            ; preds = %bb286
  br label %bb284
}

define i1 @Result_T_str_is_err(ptr %0) {
bb289:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i1, align 1
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb291, label %bb292

bb290:                                            ; preds = %bb294, %bb293, %bb291
  %load2 = load i1, ptr %match_res, align 1
  %load_zext = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %load_zext to i1
  ret i1 %ret_trunc

bb291:                                            ; preds = %bb289
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb290

bb292:                                            ; preds = %bb289
  %cmpeq5 = icmp eq i64 %load1, 1
  %zext6 = zext i1 %cmpeq5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb293, label %bb294

bb293:                                            ; preds = %bb292
  %gep8 = getelementptr i64, ptr %load, i64 1
  %load9 = load i64, ptr %gep8, align 4
  %e = alloca i64, align 8
  store i64 %load9, ptr %e, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb290

bb294:                                            ; preds = %bb292
  br label %bb290
}

define ptr @Result_T_str_unwrap(ptr %0) {
bb295:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i64, align 8
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb297, label %bb298

bb296:                                            ; preds = %bb300, %bb299, %bb297
  %load2 = load i64, ptr %match_res, align 4
  %ret_cast_ptr = inttoptr i64 %load2 to ptr
  ret ptr %ret_cast_ptr

bb297:                                            ; preds = %bb295
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  %load5 = load i64, ptr %v, align 4
  store i64 %load5, ptr %match_res, align 4
  br label %bb296

bb298:                                            ; preds = %bb295
  %cmpeq6 = icmp eq i64 %load1, 1
  %zext7 = zext i1 %cmpeq6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb299, label %bb300

bb299:                                            ; preds = %bb298
  %gep9 = getelementptr i64, ptr %load, i64 1
  %load10 = load i64, ptr %gep9, align 4
  %e = alloca i64, align 8
  store i64 %load10, ptr %e, align 4
  call void @panic(ptr @global_str.18)
  %dummy = alloca i64, align 8
  store i64 0, ptr %dummy, align 4
  %load11 = load i64, ptr %dummy, align 4
  store i64 %load11, ptr %match_res, align 4
  br label %bb296

bb300:                                            ; preds = %bb298
  br label %bb296
}

define ptr @Result_T_str_unwrap_or(ptr %0, ptr %1) {
bb301:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %default_val = alloca i64, align 8
  %store_cast_int = ptrtoint ptr %1 to i64
  store i64 %store_cast_int, ptr %default_val, align 4
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i64, align 8
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb303, label %bb304

bb302:                                            ; preds = %bb306, %bb305, %bb303
  %load2 = load i64, ptr %match_res, align 4
  %ret_cast_ptr = inttoptr i64 %load2 to ptr
  ret ptr %ret_cast_ptr

bb303:                                            ; preds = %bb301
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  %load5 = load i64, ptr %v, align 4
  store i64 %load5, ptr %match_res, align 4
  br label %bb302

bb304:                                            ; preds = %bb301
  %cmpeq6 = icmp eq i64 %load1, 1
  %zext7 = zext i1 %cmpeq6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb305, label %bb306

bb305:                                            ; preds = %bb304
  %gep9 = getelementptr i64, ptr %load, i64 1
  %load10 = load i64, ptr %gep9, align 4
  %e = alloca i64, align 8
  store i64 %load10, ptr %e, align 4
  %load11 = load i64, ptr %default_val, align 4
  store i64 %load11, ptr %match_res, align 4
  br label %bb302

bb306:                                            ; preds = %bb304
  br label %bb302
}

define i64 @Vec_str_new() {
bb307:
  %struct_alloc = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 24, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %gep = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 0
  store ptr null, ptr %gep, align 8
  %gep1 = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 1
  store i64 0, ptr %gep1, align 4
  %gep2 = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 2
  store i64 0, ptr %gep2, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Vec_str_length(ptr %0) {
bb308:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb309, label %bb310

bb309:                                            ; preds = %bb308
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 4
  ret i64 %load1

bb310:                                            ; preds = %bb308
  call void @panic(ptr @global_str.19)
  unreachable
}

define i64 @Vec_str_get_capacity(ptr %0) {
bb311:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb312, label %bb313

bb312:                                            ; preds = %bb311
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 2
  %load1 = load i64, ptr %gep, align 4
  ret i64 %load1

bb313:                                            ; preds = %bb311
  call void @panic(ptr @global_str.19)
  unreachable
}

define void @Vec_str_reserve(ptr %0, i64 %1) {
bb314:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %new_cap = alloca i64, align 8
  store i64 %1, ptr %new_cap, align 4
  %load = load i64, ptr %new_cap, align 4
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb315, label %bb316

bb315:                                            ; preds = %bb314
  %gep = getelementptr %Vec_str, ptr %load1, i64 0, i32 2
  %load2 = load i64, ptr %gep, align 4
  %cmple = icmp ule i64 %load, %load2
  %zext3 = zext i1 %cmple to i64
  %cond_true4 = icmp ne i64 %zext3, 0
  br i1 %cond_true4, label %bb317, label %bb318

bb316:                                            ; preds = %bb314
  call void @panic(ptr @global_str.19)
  unreachable

bb317:                                            ; preds = %bb315
  ret void

bb318:                                            ; preds = %bb315
  %size_in_bytes = alloca i64, align 8
  store i64 0, ptr %size_in_bytes, align 4
  %load5 = load i64, ptr %new_cap, align 4
  %mul = mul i64 %load5, 8
  store i64 %mul, ptr %size_in_bytes, align 4
  %load6 = load ptr, ptr %self, align 8
  %ptr2int7 = ptrtoint ptr %load6 to i64
  %cmpne8 = icmp ne i64 %ptr2int7, 0
  %zext9 = zext i1 %cmpne8 to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb319, label %bb320

bb319:                                            ; preds = %bb318
  %gep11 = getelementptr %Vec_str, ptr %load6, i64 0, i32 2
  %load12 = load i64, ptr %gep11, align 4
  %cmpeq = icmp eq i64 %load12, 0
  %zext13 = zext i1 %cmpeq to i64
  %cond_true14 = icmp ne i64 %zext13, 0
  br i1 %cond_true14, label %bb321, label %bb323

bb320:                                            ; preds = %bb318
  call void @panic(ptr @global_str.19)
  unreachable

bb321:                                            ; preds = %bb319
  %load15 = load ptr, ptr %self, align 8
  %ptr2int16 = ptrtoint ptr %load15 to i64
  %cmpne17 = icmp ne i64 %ptr2int16, 0
  %zext18 = zext i1 %cmpne17 to i64
  %cond_true19 = icmp ne i64 %zext18, 0
  br i1 %cond_true19, label %bb324, label %bb325

bb322:                                            ; preds = %bb328, %bb324
  %load20 = load ptr, ptr %self, align 8
  %ptr2int21 = ptrtoint ptr %load20 to i64
  %cmpne22 = icmp ne i64 %ptr2int21, 0
  %zext23 = zext i1 %cmpne22 to i64
  %cond_true24 = icmp ne i64 %zext23, 0
  br i1 %cond_true24, label %bb330, label %bb331

bb323:                                            ; preds = %bb319
  %load25 = load ptr, ptr %self, align 8
  %ptr2int26 = ptrtoint ptr %load25 to i64
  %cmpne27 = icmp ne i64 %ptr2int26, 0
  %zext28 = zext i1 %cmpne27 to i64
  %cond_true29 = icmp ne i64 %zext28, 0
  br i1 %cond_true29, label %bb326, label %bb327

bb324:                                            ; preds = %bb321
  %load30 = load i64, ptr %size_in_bytes, align 4
  %call = call ptr @malloc(i64 %load30)
  %gep31 = getelementptr %Vec_str, ptr %load15, i64 0, i32 0
  store ptr %call, ptr %gep31, align 8
  br label %bb322

bb325:                                            ; preds = %bb321
  call void @panic(ptr @global_str.20)
  unreachable

bb326:                                            ; preds = %bb323
  %load32 = load ptr, ptr %self, align 8
  %ptr2int33 = ptrtoint ptr %load32 to i64
  %cmpne34 = icmp ne i64 %ptr2int33, 0
  %zext35 = zext i1 %cmpne34 to i64
  %cond_true36 = icmp ne i64 %zext35, 0
  br i1 %cond_true36, label %bb328, label %bb329

bb327:                                            ; preds = %bb323
  call void @panic(ptr @global_str.20)
  unreachable

bb328:                                            ; preds = %bb326
  %gep37 = getelementptr %Vec_str, ptr %load32, i64 0, i32 0
  %load38 = load ptr, ptr %gep37, align 8
  %load39 = load i64, ptr %size_in_bytes, align 4
  %call40 = call ptr @realloc(ptr %load38, i64 %load39)
  %gep41 = getelementptr %Vec_str, ptr %load25, i64 0, i32 0
  store ptr %call40, ptr %gep41, align 8
  br label %bb322

bb329:                                            ; preds = %bb326
  call void @panic(ptr @global_str.19)
  unreachable

bb330:                                            ; preds = %bb322
  %gep42 = getelementptr %Vec_str, ptr %load20, i64 0, i32 0
  %load43 = load ptr, ptr %gep42, align 8
  %ptr2int44 = ptrtoint ptr %load43 to i64
  %cmpeq45 = icmp eq i64 %ptr2int44, 0
  %zext46 = zext i1 %cmpeq45 to i64
  %cond_true47 = icmp ne i64 %zext46, 0
  br i1 %cond_true47, label %bb332, label %bb333

bb331:                                            ; preds = %bb322
  call void @panic(ptr @global_str.19)
  unreachable

bb332:                                            ; preds = %bb330
  call void @panic(ptr @global_str.21)
  br label %bb333

bb333:                                            ; preds = %bb332, %bb330
  %load48 = load ptr, ptr %self, align 8
  %ptr2int49 = ptrtoint ptr %load48 to i64
  %cmpne50 = icmp ne i64 %ptr2int49, 0
  %zext51 = zext i1 %cmpne50 to i64
  %cond_true52 = icmp ne i64 %zext51, 0
  br i1 %cond_true52, label %bb334, label %bb335

bb334:                                            ; preds = %bb333
  %load53 = load i64, ptr %new_cap, align 4
  %gep54 = getelementptr %Vec_str, ptr %load48, i64 0, i32 2
  store i64 %load53, ptr %gep54, align 4
  ret void

bb335:                                            ; preds = %bb333
  call void @panic(ptr @global_str.20)
  unreachable
}

define void @Vec_str_drop(ptr %0) {
bb336:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb337, label %bb338

bb337:                                            ; preds = %bb336
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 2
  %load1 = load i64, ptr %gep, align 4
  %cmpgt = icmp ugt i64 %load1, 0
  %zext2 = zext i1 %cmpgt to i64
  %cond_true3 = icmp ne i64 %zext2, 0
  br i1 %cond_true3, label %bb339, label %bb340

bb338:                                            ; preds = %bb336
  call void @panic(ptr @global_str.19)
  unreachable

bb339:                                            ; preds = %bb337
  %load4 = load ptr, ptr %self, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb341, label %bb342

bb340:                                            ; preds = %bb345, %bb337
  ret void

bb341:                                            ; preds = %bb339
  %gep9 = getelementptr %Vec_str, ptr %load4, i64 0, i32 0
  %load10 = load ptr, ptr %gep9, align 8
  call void @free(ptr %load10)
  %load11 = load ptr, ptr %self, align 8
  %gep12 = getelementptr i64, ptr %load11, i64 0
  store i64 0, ptr %gep12, align 4
  %load13 = load ptr, ptr %self, align 8
  %ptr2int14 = ptrtoint ptr %load13 to i64
  %cmpne15 = icmp ne i64 %ptr2int14, 0
  %zext16 = zext i1 %cmpne15 to i64
  %cond_true17 = icmp ne i64 %zext16, 0
  br i1 %cond_true17, label %bb343, label %bb344

bb342:                                            ; preds = %bb339
  call void @panic(ptr @global_str.19)
  unreachable

bb343:                                            ; preds = %bb341
  %gep18 = getelementptr %Vec_str, ptr %load13, i64 0, i32 2
  store i64 0, ptr %gep18, align 4
  %load19 = load ptr, ptr %self, align 8
  %ptr2int20 = ptrtoint ptr %load19 to i64
  %cmpne21 = icmp ne i64 %ptr2int20, 0
  %zext22 = zext i1 %cmpne21 to i64
  %cond_true23 = icmp ne i64 %zext22, 0
  br i1 %cond_true23, label %bb345, label %bb346

bb344:                                            ; preds = %bb341
  call void @panic(ptr @global_str.20)
  unreachable

bb345:                                            ; preds = %bb343
  %gep24 = getelementptr %Vec_str, ptr %load19, i64 0, i32 1
  store i64 0, ptr %gep24, align 4
  br label %bb340

bb346:                                            ; preds = %bb343
  call void @panic(ptr @global_str.20)
  unreachable
}

define void @Vec_str_push(ptr %0, ptr %1) {
bb347:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %item = alloca ptr, align 8
  store ptr %1, ptr %item, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb348, label %bb349

bb348:                                            ; preds = %bb347
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb350, label %bb351

bb349:                                            ; preds = %bb347
  call void @panic(ptr @global_str.19)
  unreachable

bb350:                                            ; preds = %bb348
  %gep7 = getelementptr %Vec_str, ptr %load2, i64 0, i32 2
  %load8 = load i64, ptr %gep7, align 4
  %cmpeq = icmp eq i64 %load1, %load8
  %zext9 = zext i1 %cmpeq to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb352, label %bb353

bb351:                                            ; preds = %bb348
  call void @panic(ptr @global_str.19)
  unreachable

bb352:                                            ; preds = %bb350
  %next_cap = alloca i64, align 8
  store i64 0, ptr %next_cap, align 4
  store i64 4, ptr %next_cap, align 4
  %load11 = load ptr, ptr %self, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb354, label %bb355

bb353:                                            ; preds = %bb360, %bb350
  %load16 = load ptr, ptr %self, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpne18 = icmp ne i64 %ptr2int17, 0
  %zext19 = zext i1 %cmpne18 to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb362, label %bb363

bb354:                                            ; preds = %bb352
  %gep21 = getelementptr %Vec_str, ptr %load11, i64 0, i32 2
  %load22 = load i64, ptr %gep21, align 4
  %cmpgt = icmp ugt i64 %load22, 0
  %zext23 = zext i1 %cmpgt to i64
  %cond_true24 = icmp ne i64 %zext23, 0
  br i1 %cond_true24, label %bb356, label %bb357

bb355:                                            ; preds = %bb352
  call void @panic(ptr @global_str.19)
  unreachable

bb356:                                            ; preds = %bb354
  %load25 = load ptr, ptr %self, align 8
  %ptr2int26 = ptrtoint ptr %load25 to i64
  %cmpne27 = icmp ne i64 %ptr2int26, 0
  %zext28 = zext i1 %cmpne27 to i64
  %cond_true29 = icmp ne i64 %zext28, 0
  br i1 %cond_true29, label %bb358, label %bb359

bb357:                                            ; preds = %bb358, %bb354
  %load30 = load i64, ptr %next_cap, align 4
  %load31 = load ptr, ptr %self, align 8
  %ptr2int32 = ptrtoint ptr %load31 to i64
  %cmpne33 = icmp ne i64 %ptr2int32, 0
  %zext34 = zext i1 %cmpne33 to i64
  %cond_true35 = icmp ne i64 %zext34, 0
  br i1 %cond_true35, label %bb360, label %bb361

bb358:                                            ; preds = %bb356
  %gep36 = getelementptr %Vec_str, ptr %load25, i64 0, i32 2
  %load37 = load i64, ptr %gep36, align 4
  %mul = mul i64 %load37, 2
  store i64 %mul, ptr %next_cap, align 4
  br label %bb357

bb359:                                            ; preds = %bb356
  call void @panic(ptr @global_str.19)
  unreachable

bb360:                                            ; preds = %bb357
  call void @Vec_str_reserve(ptr %load31, i64 %load30)
  br label %bb353

bb361:                                            ; preds = %bb357
  call void @panic(ptr @global_str.6)
  unreachable

bb362:                                            ; preds = %bb353
  %gep38 = getelementptr %Vec_str, ptr %load16, i64 0, i32 0
  %load39 = load ptr, ptr %gep38, align 8
  %ptr2int40 = ptrtoint ptr %load39 to i64
  %cmpne41 = icmp ne i64 %ptr2int40, 0
  %zext42 = zext i1 %cmpne41 to i64
  %cond_true43 = icmp ne i64 %zext42, 0
  br i1 %cond_true43, label %bb364, label %bb365

bb363:                                            ; preds = %bb353
  call void @panic(ptr @global_str.19)
  unreachable

bb364:                                            ; preds = %bb362
  %load44 = load ptr, ptr %self, align 8
  %ptr2int45 = ptrtoint ptr %load44 to i64
  %cmpne46 = icmp ne i64 %ptr2int45, 0
  %zext47 = zext i1 %cmpne46 to i64
  %cond_true48 = icmp ne i64 %zext47, 0
  br i1 %cond_true48, label %bb366, label %bb367

bb365:                                            ; preds = %bb362
  call void @panic(ptr @global_str.3)
  unreachable

bb366:                                            ; preds = %bb364
  %gep49 = getelementptr %Vec_str, ptr %load44, i64 0, i32 1
  %load50 = load i64, ptr %gep49, align 4
  %gep51 = getelementptr i64, ptr %load39, i64 -1
  %load52 = load i64, ptr %gep51, align 4
  %cmplt = icmp slt i64 %load50, %load52
  %zext53 = zext i1 %cmplt to i64
  %cond_true54 = icmp ne i64 %zext53, 0
  br i1 %cond_true54, label %bb368, label %bb369

bb367:                                            ; preds = %bb364
  call void @panic(ptr @global_str.19)
  unreachable

bb368:                                            ; preds = %bb366
  %cmpge = icmp sge i64 %load50, 0
  %zext55 = zext i1 %cmpge to i64
  %cond_true56 = icmp ne i64 %zext55, 0
  br i1 %cond_true56, label %bb370, label %bb369

bb369:                                            ; preds = %bb368, %bb366
  call void @panic(ptr @global_str.4)
  unreachable

bb370:                                            ; preds = %bb368
  %load57 = load ptr, ptr %item, align 8
  %gep58 = getelementptr ptr, ptr %load39, i64 %load50
  store ptr %load57, ptr %gep58, align 8
  %load59 = load ptr, ptr %self, align 8
  %ptr2int60 = ptrtoint ptr %load59 to i64
  %cmpne61 = icmp ne i64 %ptr2int60, 0
  %zext62 = zext i1 %cmpne61 to i64
  %cond_true63 = icmp ne i64 %zext62, 0
  br i1 %cond_true63, label %bb371, label %bb372

bb371:                                            ; preds = %bb370
  %load64 = load ptr, ptr %self, align 8
  %ptr2int65 = ptrtoint ptr %load64 to i64
  %cmpne66 = icmp ne i64 %ptr2int65, 0
  %zext67 = zext i1 %cmpne66 to i64
  %cond_true68 = icmp ne i64 %zext67, 0
  br i1 %cond_true68, label %bb373, label %bb374

bb372:                                            ; preds = %bb370
  call void @panic(ptr @global_str.20)
  unreachable

bb373:                                            ; preds = %bb371
  %gep69 = getelementptr %Vec_str, ptr %load64, i64 0, i32 1
  %load70 = load i64, ptr %gep69, align 4
  %add = add i64 %load70, 1
  %gep71 = getelementptr %Vec_str, ptr %load59, i64 0, i32 1
  store i64 %add, ptr %gep71, align 4
  ret void

bb374:                                            ; preds = %bb371
  call void @panic(ptr @global_str.19)
  unreachable
}

define void @Vec_str_insert(ptr %0, i64 %1, ptr %2) {
bb375:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %index = alloca i64, align 8
  store i64 %1, ptr %index, align 4
  %item = alloca ptr, align 8
  store ptr %2, ptr %item, align 8
  %load = load i64, ptr %index, align 4
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb376, label %bb377

bb376:                                            ; preds = %bb375
  %gep = getelementptr %Vec_str, ptr %load1, i64 0, i32 1
  %load2 = load i64, ptr %gep, align 4
  %cmpgt = icmp ugt i64 %load, %load2
  %zext3 = zext i1 %cmpgt to i64
  %cond_true4 = icmp ne i64 %zext3, 0
  br i1 %cond_true4, label %bb378, label %bb379

bb377:                                            ; preds = %bb375
  call void @panic(ptr @global_str.19)
  unreachable

bb378:                                            ; preds = %bb376
  call void @panic(ptr @global_str.22)
  br label %bb379

bb379:                                            ; preds = %bb378, %bb376
  %load5 = load ptr, ptr %self, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb380, label %bb381

bb380:                                            ; preds = %bb379
  %gep10 = getelementptr %Vec_str, ptr %load5, i64 0, i32 1
  %load11 = load i64, ptr %gep10, align 4
  %load12 = load ptr, ptr %self, align 8
  %ptr2int13 = ptrtoint ptr %load12 to i64
  %cmpne14 = icmp ne i64 %ptr2int13, 0
  %zext15 = zext i1 %cmpne14 to i64
  %cond_true16 = icmp ne i64 %zext15, 0
  br i1 %cond_true16, label %bb382, label %bb383

bb381:                                            ; preds = %bb379
  call void @panic(ptr @global_str.19)
  unreachable

bb382:                                            ; preds = %bb380
  %gep17 = getelementptr %Vec_str, ptr %load12, i64 0, i32 2
  %load18 = load i64, ptr %gep17, align 4
  %cmpeq = icmp eq i64 %load11, %load18
  %zext19 = zext i1 %cmpeq to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb384, label %bb385

bb383:                                            ; preds = %bb380
  call void @panic(ptr @global_str.19)
  unreachable

bb384:                                            ; preds = %bb382
  %next_cap = alloca i64, align 8
  store i64 0, ptr %next_cap, align 4
  store i64 4, ptr %next_cap, align 4
  %load21 = load ptr, ptr %self, align 8
  %ptr2int22 = ptrtoint ptr %load21 to i64
  %cmpne23 = icmp ne i64 %ptr2int22, 0
  %zext24 = zext i1 %cmpne23 to i64
  %cond_true25 = icmp ne i64 %zext24, 0
  br i1 %cond_true25, label %bb386, label %bb387

bb385:                                            ; preds = %bb392, %bb382
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 4
  %load26 = load ptr, ptr %self, align 8
  %ptr2int27 = ptrtoint ptr %load26 to i64
  %cmpne28 = icmp ne i64 %ptr2int27, 0
  %zext29 = zext i1 %cmpne28 to i64
  %cond_true30 = icmp ne i64 %zext29, 0
  br i1 %cond_true30, label %bb394, label %bb395

bb386:                                            ; preds = %bb384
  %gep31 = getelementptr %Vec_str, ptr %load21, i64 0, i32 2
  %load32 = load i64, ptr %gep31, align 4
  %cmpgt33 = icmp ugt i64 %load32, 0
  %zext34 = zext i1 %cmpgt33 to i64
  %cond_true35 = icmp ne i64 %zext34, 0
  br i1 %cond_true35, label %bb388, label %bb389

bb387:                                            ; preds = %bb384
  call void @panic(ptr @global_str.19)
  unreachable

bb388:                                            ; preds = %bb386
  %load36 = load ptr, ptr %self, align 8
  %ptr2int37 = ptrtoint ptr %load36 to i64
  %cmpne38 = icmp ne i64 %ptr2int37, 0
  %zext39 = zext i1 %cmpne38 to i64
  %cond_true40 = icmp ne i64 %zext39, 0
  br i1 %cond_true40, label %bb390, label %bb391

bb389:                                            ; preds = %bb390, %bb386
  %load41 = load i64, ptr %next_cap, align 4
  %load42 = load ptr, ptr %self, align 8
  %ptr2int43 = ptrtoint ptr %load42 to i64
  %cmpne44 = icmp ne i64 %ptr2int43, 0
  %zext45 = zext i1 %cmpne44 to i64
  %cond_true46 = icmp ne i64 %zext45, 0
  br i1 %cond_true46, label %bb392, label %bb393

bb390:                                            ; preds = %bb388
  %gep47 = getelementptr %Vec_str, ptr %load36, i64 0, i32 2
  %load48 = load i64, ptr %gep47, align 4
  %mul = mul i64 %load48, 2
  store i64 %mul, ptr %next_cap, align 4
  br label %bb389

bb391:                                            ; preds = %bb388
  call void @panic(ptr @global_str.19)
  unreachable

bb392:                                            ; preds = %bb389
  call void @Vec_str_reserve(ptr %load42, i64 %load41)
  br label %bb385

bb393:                                            ; preds = %bb389
  call void @panic(ptr @global_str.6)
  unreachable

bb394:                                            ; preds = %bb385
  %gep49 = getelementptr %Vec_str, ptr %load26, i64 0, i32 1
  %load50 = load i64, ptr %gep49, align 4
  store i64 %load50, ptr %i, align 4
  br label %bb396

bb395:                                            ; preds = %bb385
  call void @panic(ptr @global_str.19)
  unreachable

bb396:                                            ; preds = %bb412, %bb394
  %load51 = load i64, ptr %i, align 4
  %load52 = load i64, ptr %index, align 4
  %cmpgt53 = icmp ugt i64 %load51, %load52
  %zext54 = zext i1 %cmpgt53 to i64
  %cond_true55 = icmp ne i64 %zext54, 0
  br i1 %cond_true55, label %bb397, label %bb398

bb397:                                            ; preds = %bb396
  %load56 = load ptr, ptr %self, align 8
  %ptr2int57 = ptrtoint ptr %load56 to i64
  %cmpne58 = icmp ne i64 %ptr2int57, 0
  %zext59 = zext i1 %cmpne58 to i64
  %cond_true60 = icmp ne i64 %zext59, 0
  br i1 %cond_true60, label %bb399, label %bb400

bb398:                                            ; preds = %bb396
  %load61 = load ptr, ptr %self, align 8
  %ptr2int62 = ptrtoint ptr %load61 to i64
  %cmpne63 = icmp ne i64 %ptr2int62, 0
  %zext64 = zext i1 %cmpne63 to i64
  %cond_true65 = icmp ne i64 %zext64, 0
  br i1 %cond_true65, label %bb413, label %bb414

bb399:                                            ; preds = %bb397
  %gep66 = getelementptr %Vec_str, ptr %load56, i64 0, i32 0
  %load67 = load ptr, ptr %gep66, align 8
  %ptr2int68 = ptrtoint ptr %load67 to i64
  %cmpne69 = icmp ne i64 %ptr2int68, 0
  %zext70 = zext i1 %cmpne69 to i64
  %cond_true71 = icmp ne i64 %zext70, 0
  br i1 %cond_true71, label %bb401, label %bb402

bb400:                                            ; preds = %bb397
  call void @panic(ptr @global_str.19)
  unreachable

bb401:                                            ; preds = %bb399
  %load72 = load i64, ptr %i, align 4
  %gep73 = getelementptr i64, ptr %load67, i64 -1
  %load74 = load i64, ptr %gep73, align 4
  %cmplt = icmp slt i64 %load72, %load74
  %zext75 = zext i1 %cmplt to i64
  %cond_true76 = icmp ne i64 %zext75, 0
  br i1 %cond_true76, label %bb403, label %bb404

bb402:                                            ; preds = %bb399
  call void @panic(ptr @global_str.3)
  unreachable

bb403:                                            ; preds = %bb401
  %cmpge = icmp sge i64 %load72, 0
  %zext77 = zext i1 %cmpge to i64
  %cond_true78 = icmp ne i64 %zext77, 0
  br i1 %cond_true78, label %bb405, label %bb404

bb404:                                            ; preds = %bb403, %bb401
  call void @panic(ptr @global_str.4)
  unreachable

bb405:                                            ; preds = %bb403
  %load79 = load ptr, ptr %self, align 8
  %ptr2int80 = ptrtoint ptr %load79 to i64
  %cmpne81 = icmp ne i64 %ptr2int80, 0
  %zext82 = zext i1 %cmpne81 to i64
  %cond_true83 = icmp ne i64 %zext82, 0
  br i1 %cond_true83, label %bb406, label %bb407

bb406:                                            ; preds = %bb405
  %gep84 = getelementptr %Vec_str, ptr %load79, i64 0, i32 0
  %load85 = load ptr, ptr %gep84, align 8
  %ptr2int86 = ptrtoint ptr %load85 to i64
  %cmpne87 = icmp ne i64 %ptr2int86, 0
  %zext88 = zext i1 %cmpne87 to i64
  %cond_true89 = icmp ne i64 %zext88, 0
  br i1 %cond_true89, label %bb408, label %bb409

bb407:                                            ; preds = %bb405
  call void @panic(ptr @global_str.19)
  unreachable

bb408:                                            ; preds = %bb406
  %load90 = load i64, ptr %i, align 4
  %sub = sub i64 %load90, 1
  %gep91 = getelementptr i64, ptr %load85, i64 -1
  %load92 = load i64, ptr %gep91, align 4
  %cmplt93 = icmp slt i64 %sub, %load92
  %zext94 = zext i1 %cmplt93 to i64
  %cond_true95 = icmp ne i64 %zext94, 0
  br i1 %cond_true95, label %bb410, label %bb411

bb409:                                            ; preds = %bb406
  call void @panic(ptr @global_str.5)
  unreachable

bb410:                                            ; preds = %bb408
  %cmpge96 = icmp sge i64 %sub, 0
  %zext97 = zext i1 %cmpge96 to i64
  %cond_true98 = icmp ne i64 %zext97, 0
  br i1 %cond_true98, label %bb412, label %bb411

bb411:                                            ; preds = %bb410, %bb408
  call void @panic(ptr @global_str.4)
  unreachable

bb412:                                            ; preds = %bb410
  %gep99 = getelementptr ptr, ptr %load85, i64 %sub
  %load100 = load ptr, ptr %gep99, align 8
  %gep101 = getelementptr ptr, ptr %load67, i64 %load72
  store ptr %load100, ptr %gep101, align 8
  %load102 = load i64, ptr %i, align 4
  %sub103 = sub i64 %load102, 1
  store i64 %sub103, ptr %i, align 4
  br label %bb396

bb413:                                            ; preds = %bb398
  %gep104 = getelementptr %Vec_str, ptr %load61, i64 0, i32 0
  %load105 = load ptr, ptr %gep104, align 8
  %ptr2int106 = ptrtoint ptr %load105 to i64
  %cmpne107 = icmp ne i64 %ptr2int106, 0
  %zext108 = zext i1 %cmpne107 to i64
  %cond_true109 = icmp ne i64 %zext108, 0
  br i1 %cond_true109, label %bb415, label %bb416

bb414:                                            ; preds = %bb398
  call void @panic(ptr @global_str.19)
  unreachable

bb415:                                            ; preds = %bb413
  %load110 = load i64, ptr %index, align 4
  %gep111 = getelementptr i64, ptr %load105, i64 -1
  %load112 = load i64, ptr %gep111, align 4
  %cmplt113 = icmp slt i64 %load110, %load112
  %zext114 = zext i1 %cmplt113 to i64
  %cond_true115 = icmp ne i64 %zext114, 0
  br i1 %cond_true115, label %bb417, label %bb418

bb416:                                            ; preds = %bb413
  call void @panic(ptr @global_str.3)
  unreachable

bb417:                                            ; preds = %bb415
  %cmpge116 = icmp sge i64 %load110, 0
  %zext117 = zext i1 %cmpge116 to i64
  %cond_true118 = icmp ne i64 %zext117, 0
  br i1 %cond_true118, label %bb419, label %bb418

bb418:                                            ; preds = %bb417, %bb415
  call void @panic(ptr @global_str.4)
  unreachable

bb419:                                            ; preds = %bb417
  %load119 = load ptr, ptr %item, align 8
  %gep120 = getelementptr ptr, ptr %load105, i64 %load110
  store ptr %load119, ptr %gep120, align 8
  %load121 = load ptr, ptr %self, align 8
  %ptr2int122 = ptrtoint ptr %load121 to i64
  %cmpne123 = icmp ne i64 %ptr2int122, 0
  %zext124 = zext i1 %cmpne123 to i64
  %cond_true125 = icmp ne i64 %zext124, 0
  br i1 %cond_true125, label %bb420, label %bb421

bb420:                                            ; preds = %bb419
  %load126 = load ptr, ptr %self, align 8
  %ptr2int127 = ptrtoint ptr %load126 to i64
  %cmpne128 = icmp ne i64 %ptr2int127, 0
  %zext129 = zext i1 %cmpne128 to i64
  %cond_true130 = icmp ne i64 %zext129, 0
  br i1 %cond_true130, label %bb422, label %bb423

bb421:                                            ; preds = %bb419
  call void @panic(ptr @global_str.20)
  unreachable

bb422:                                            ; preds = %bb420
  %gep131 = getelementptr %Vec_str, ptr %load126, i64 0, i32 1
  %load132 = load i64, ptr %gep131, align 4
  %add = add i64 %load132, 1
  %gep133 = getelementptr %Vec_str, ptr %load121, i64 0, i32 1
  store i64 %add, ptr %gep133, align 4
  ret void

bb423:                                            ; preds = %bb420
  call void @panic(ptr @global_str.19)
  unreachable
}

define i64 @Vec_str_pop(ptr %0) {
bb424:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb425, label %bb426

bb425:                                            ; preds = %bb424
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext2 = zext i1 %cmpeq to i64
  %cond_true3 = icmp ne i64 %zext2, 0
  br i1 %cond_true3, label %bb427, label %bb428

bb426:                                            ; preds = %bb424
  call void @panic(ptr @global_str.19)
  unreachable

bb427:                                            ; preds = %bb425
  %call = call i64 @Result_str_str_Err(i64 ptrtoint (ptr @global_str.23 to i64))
  ret i64 %call

bb428:                                            ; preds = %bb425
  %load4 = load ptr, ptr %self, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb429, label %bb430

bb429:                                            ; preds = %bb428
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb431, label %bb432

bb430:                                            ; preds = %bb428
  call void @panic(ptr @global_str.20)
  unreachable

bb431:                                            ; preds = %bb429
  %gep14 = getelementptr %Vec_str, ptr %load9, i64 0, i32 1
  %load15 = load i64, ptr %gep14, align 4
  %sub = sub i64 %load15, 1
  %gep16 = getelementptr %Vec_str, ptr %load4, i64 0, i32 1
  store i64 %sub, ptr %gep16, align 4
  %load17 = load ptr, ptr %self, align 8
  %ptr2int18 = ptrtoint ptr %load17 to i64
  %cmpne19 = icmp ne i64 %ptr2int18, 0
  %zext20 = zext i1 %cmpne19 to i64
  %cond_true21 = icmp ne i64 %zext20, 0
  br i1 %cond_true21, label %bb433, label %bb434

bb432:                                            ; preds = %bb429
  call void @panic(ptr @global_str.19)
  unreachable

bb433:                                            ; preds = %bb431
  %gep22 = getelementptr %Vec_str, ptr %load17, i64 0, i32 0
  %load23 = load ptr, ptr %gep22, align 8
  %ptr2int24 = ptrtoint ptr %load23 to i64
  %cmpne25 = icmp ne i64 %ptr2int24, 0
  %zext26 = zext i1 %cmpne25 to i64
  %cond_true27 = icmp ne i64 %zext26, 0
  br i1 %cond_true27, label %bb435, label %bb436

bb434:                                            ; preds = %bb431
  call void @panic(ptr @global_str.19)
  unreachable

bb435:                                            ; preds = %bb433
  %load28 = load ptr, ptr %self, align 8
  %ptr2int29 = ptrtoint ptr %load28 to i64
  %cmpne30 = icmp ne i64 %ptr2int29, 0
  %zext31 = zext i1 %cmpne30 to i64
  %cond_true32 = icmp ne i64 %zext31, 0
  br i1 %cond_true32, label %bb437, label %bb438

bb436:                                            ; preds = %bb433
  call void @panic(ptr @global_str.5)
  unreachable

bb437:                                            ; preds = %bb435
  %gep33 = getelementptr %Vec_str, ptr %load28, i64 0, i32 1
  %load34 = load i64, ptr %gep33, align 4
  %gep35 = getelementptr i64, ptr %load23, i64 -1
  %load36 = load i64, ptr %gep35, align 4
  %cmplt = icmp slt i64 %load34, %load36
  %zext37 = zext i1 %cmplt to i64
  %cond_true38 = icmp ne i64 %zext37, 0
  br i1 %cond_true38, label %bb439, label %bb440

bb438:                                            ; preds = %bb435
  call void @panic(ptr @global_str.19)
  unreachable

bb439:                                            ; preds = %bb437
  %cmpge = icmp sge i64 %load34, 0
  %zext39 = zext i1 %cmpge to i64
  %cond_true40 = icmp ne i64 %zext39, 0
  br i1 %cond_true40, label %bb441, label %bb440

bb440:                                            ; preds = %bb439, %bb437
  call void @panic(ptr @global_str.4)
  unreachable

bb441:                                            ; preds = %bb439
  %gep41 = getelementptr ptr, ptr %load23, i64 %load34
  %load42 = load ptr, ptr %gep41, align 8
  %auto_cast_int = ptrtoint ptr %load42 to i64
  %call43 = call i64 @Result_str_str_Ok(i64 %auto_cast_int)
  ret i64 %call43
}

define ptr @Vec_str_remove(ptr %0, i64 %1) {
bb442:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %index = alloca i64, align 8
  store i64 %1, ptr %index, align 4
  %load = load i64, ptr %index, align 4
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb443, label %bb444

bb443:                                            ; preds = %bb442
  %gep = getelementptr %Vec_str, ptr %load1, i64 0, i32 1
  %load2 = load i64, ptr %gep, align 4
  %cmpge = icmp uge i64 %load, %load2
  %zext3 = zext i1 %cmpge to i64
  %cond_true4 = icmp ne i64 %zext3, 0
  br i1 %cond_true4, label %bb445, label %bb446

bb444:                                            ; preds = %bb442
  call void @panic(ptr @global_str.19)
  unreachable

bb445:                                            ; preds = %bb443
  call void @panic(ptr @global_str.24)
  br label %bb446

bb446:                                            ; preds = %bb445, %bb443
  %removed_item = alloca ptr, align 8
  store ptr null, ptr %removed_item, align 8
  %load5 = load ptr, ptr %self, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb447, label %bb448

bb447:                                            ; preds = %bb446
  %gep10 = getelementptr %Vec_str, ptr %load5, i64 0, i32 0
  %load11 = load ptr, ptr %gep10, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb449, label %bb450

bb448:                                            ; preds = %bb446
  call void @panic(ptr @global_str.19)
  unreachable

bb449:                                            ; preds = %bb447
  %load16 = load i64, ptr %index, align 4
  %gep17 = getelementptr i64, ptr %load11, i64 -1
  %load18 = load i64, ptr %gep17, align 4
  %cmplt = icmp slt i64 %load16, %load18
  %zext19 = zext i1 %cmplt to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb451, label %bb452

bb450:                                            ; preds = %bb447
  call void @panic(ptr @global_str.5)
  unreachable

bb451:                                            ; preds = %bb449
  %cmpge21 = icmp sge i64 %load16, 0
  %zext22 = zext i1 %cmpge21 to i64
  %cond_true23 = icmp ne i64 %zext22, 0
  br i1 %cond_true23, label %bb453, label %bb452

bb452:                                            ; preds = %bb451, %bb449
  call void @panic(ptr @global_str.4)
  unreachable

bb453:                                            ; preds = %bb451
  %gep24 = getelementptr ptr, ptr %load11, i64 %load16
  %load25 = load ptr, ptr %gep24, align 8
  store ptr %load25, ptr %removed_item, align 8
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 4
  %load26 = load i64, ptr %index, align 4
  store i64 %load26, ptr %i, align 4
  br label %bb454

bb454:                                            ; preds = %bb472, %bb453
  %load27 = load i64, ptr %i, align 4
  %load28 = load ptr, ptr %self, align 8
  %ptr2int29 = ptrtoint ptr %load28 to i64
  %cmpne30 = icmp ne i64 %ptr2int29, 0
  %zext31 = zext i1 %cmpne30 to i64
  %cond_true32 = icmp ne i64 %zext31, 0
  br i1 %cond_true32, label %bb457, label %bb458

bb455:                                            ; preds = %bb457
  %load33 = load ptr, ptr %self, align 8
  %ptr2int34 = ptrtoint ptr %load33 to i64
  %cmpne35 = icmp ne i64 %ptr2int34, 0
  %zext36 = zext i1 %cmpne35 to i64
  %cond_true37 = icmp ne i64 %zext36, 0
  br i1 %cond_true37, label %bb459, label %bb460

bb456:                                            ; preds = %bb457
  %load38 = load ptr, ptr %self, align 8
  %ptr2int39 = ptrtoint ptr %load38 to i64
  %cmpne40 = icmp ne i64 %ptr2int39, 0
  %zext41 = zext i1 %cmpne40 to i64
  %cond_true42 = icmp ne i64 %zext41, 0
  br i1 %cond_true42, label %bb473, label %bb474

bb457:                                            ; preds = %bb454
  %gep43 = getelementptr %Vec_str, ptr %load28, i64 0, i32 1
  %load44 = load i64, ptr %gep43, align 4
  %sub = sub i64 %load44, 1
  %cmplt45 = icmp ult i64 %load27, %sub
  %zext46 = zext i1 %cmplt45 to i64
  %cond_true47 = icmp ne i64 %zext46, 0
  br i1 %cond_true47, label %bb455, label %bb456

bb458:                                            ; preds = %bb454
  call void @panic(ptr @global_str.19)
  unreachable

bb459:                                            ; preds = %bb455
  %gep48 = getelementptr %Vec_str, ptr %load33, i64 0, i32 0
  %load49 = load ptr, ptr %gep48, align 8
  %ptr2int50 = ptrtoint ptr %load49 to i64
  %cmpne51 = icmp ne i64 %ptr2int50, 0
  %zext52 = zext i1 %cmpne51 to i64
  %cond_true53 = icmp ne i64 %zext52, 0
  br i1 %cond_true53, label %bb461, label %bb462

bb460:                                            ; preds = %bb455
  call void @panic(ptr @global_str.19)
  unreachable

bb461:                                            ; preds = %bb459
  %load54 = load i64, ptr %i, align 4
  %gep55 = getelementptr i64, ptr %load49, i64 -1
  %load56 = load i64, ptr %gep55, align 4
  %cmplt57 = icmp slt i64 %load54, %load56
  %zext58 = zext i1 %cmplt57 to i64
  %cond_true59 = icmp ne i64 %zext58, 0
  br i1 %cond_true59, label %bb463, label %bb464

bb462:                                            ; preds = %bb459
  call void @panic(ptr @global_str.3)
  unreachable

bb463:                                            ; preds = %bb461
  %cmpge60 = icmp sge i64 %load54, 0
  %zext61 = zext i1 %cmpge60 to i64
  %cond_true62 = icmp ne i64 %zext61, 0
  br i1 %cond_true62, label %bb465, label %bb464

bb464:                                            ; preds = %bb463, %bb461
  call void @panic(ptr @global_str.4)
  unreachable

bb465:                                            ; preds = %bb463
  %load63 = load ptr, ptr %self, align 8
  %ptr2int64 = ptrtoint ptr %load63 to i64
  %cmpne65 = icmp ne i64 %ptr2int64, 0
  %zext66 = zext i1 %cmpne65 to i64
  %cond_true67 = icmp ne i64 %zext66, 0
  br i1 %cond_true67, label %bb466, label %bb467

bb466:                                            ; preds = %bb465
  %gep68 = getelementptr %Vec_str, ptr %load63, i64 0, i32 0
  %load69 = load ptr, ptr %gep68, align 8
  %ptr2int70 = ptrtoint ptr %load69 to i64
  %cmpne71 = icmp ne i64 %ptr2int70, 0
  %zext72 = zext i1 %cmpne71 to i64
  %cond_true73 = icmp ne i64 %zext72, 0
  br i1 %cond_true73, label %bb468, label %bb469

bb467:                                            ; preds = %bb465
  call void @panic(ptr @global_str.19)
  unreachable

bb468:                                            ; preds = %bb466
  %load74 = load i64, ptr %i, align 4
  %add = add i64 %load74, 1
  %gep75 = getelementptr i64, ptr %load69, i64 -1
  %load76 = load i64, ptr %gep75, align 4
  %cmplt77 = icmp slt i64 %add, %load76
  %zext78 = zext i1 %cmplt77 to i64
  %cond_true79 = icmp ne i64 %zext78, 0
  br i1 %cond_true79, label %bb470, label %bb471

bb469:                                            ; preds = %bb466
  call void @panic(ptr @global_str.5)
  unreachable

bb470:                                            ; preds = %bb468
  %cmpge80 = icmp sge i64 %add, 0
  %zext81 = zext i1 %cmpge80 to i64
  %cond_true82 = icmp ne i64 %zext81, 0
  br i1 %cond_true82, label %bb472, label %bb471

bb471:                                            ; preds = %bb470, %bb468
  call void @panic(ptr @global_str.4)
  unreachable

bb472:                                            ; preds = %bb470
  %gep83 = getelementptr ptr, ptr %load69, i64 %add
  %load84 = load ptr, ptr %gep83, align 8
  %gep85 = getelementptr ptr, ptr %load49, i64 %load54
  store ptr %load84, ptr %gep85, align 8
  %load86 = load i64, ptr %i, align 4
  %add87 = add i64 %load86, 1
  store i64 %add87, ptr %i, align 4
  br label %bb454

bb473:                                            ; preds = %bb456
  %load88 = load ptr, ptr %self, align 8
  %ptr2int89 = ptrtoint ptr %load88 to i64
  %cmpne90 = icmp ne i64 %ptr2int89, 0
  %zext91 = zext i1 %cmpne90 to i64
  %cond_true92 = icmp ne i64 %zext91, 0
  br i1 %cond_true92, label %bb475, label %bb476

bb474:                                            ; preds = %bb456
  call void @panic(ptr @global_str.20)
  unreachable

bb475:                                            ; preds = %bb473
  %gep93 = getelementptr %Vec_str, ptr %load88, i64 0, i32 1
  %load94 = load i64, ptr %gep93, align 4
  %sub95 = sub i64 %load94, 1
  %gep96 = getelementptr %Vec_str, ptr %load38, i64 0, i32 1
  store i64 %sub95, ptr %gep96, align 4
  %load97 = load ptr, ptr %removed_item, align 8
  ret ptr %load97

bb476:                                            ; preds = %bb473
  call void @panic(ptr @global_str.19)
  unreachable
}

define ptr @Vec_str_swap_remove(ptr %0, i64 %1) {
bb477:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %index = alloca i64, align 8
  store i64 %1, ptr %index, align 4
  %load = load i64, ptr %index, align 4
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb478, label %bb479

bb478:                                            ; preds = %bb477
  %gep = getelementptr %Vec_str, ptr %load1, i64 0, i32 1
  %load2 = load i64, ptr %gep, align 4
  %cmpge = icmp uge i64 %load, %load2
  %zext3 = zext i1 %cmpge to i64
  %cond_true4 = icmp ne i64 %zext3, 0
  br i1 %cond_true4, label %bb480, label %bb481

bb479:                                            ; preds = %bb477
  call void @panic(ptr @global_str.19)
  unreachable

bb480:                                            ; preds = %bb478
  call void @panic(ptr @global_str.25)
  br label %bb481

bb481:                                            ; preds = %bb480, %bb478
  %removed_item = alloca ptr, align 8
  store ptr null, ptr %removed_item, align 8
  %load5 = load ptr, ptr %self, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb482, label %bb483

bb482:                                            ; preds = %bb481
  %gep10 = getelementptr %Vec_str, ptr %load5, i64 0, i32 0
  %load11 = load ptr, ptr %gep10, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb484, label %bb485

bb483:                                            ; preds = %bb481
  call void @panic(ptr @global_str.19)
  unreachable

bb484:                                            ; preds = %bb482
  %load16 = load i64, ptr %index, align 4
  %gep17 = getelementptr i64, ptr %load11, i64 -1
  %load18 = load i64, ptr %gep17, align 4
  %cmplt = icmp slt i64 %load16, %load18
  %zext19 = zext i1 %cmplt to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb486, label %bb487

bb485:                                            ; preds = %bb482
  call void @panic(ptr @global_str.5)
  unreachable

bb486:                                            ; preds = %bb484
  %cmpge21 = icmp sge i64 %load16, 0
  %zext22 = zext i1 %cmpge21 to i64
  %cond_true23 = icmp ne i64 %zext22, 0
  br i1 %cond_true23, label %bb488, label %bb487

bb487:                                            ; preds = %bb486, %bb484
  call void @panic(ptr @global_str.4)
  unreachable

bb488:                                            ; preds = %bb486
  %gep24 = getelementptr ptr, ptr %load11, i64 %load16
  %load25 = load ptr, ptr %gep24, align 8
  store ptr %load25, ptr %removed_item, align 8
  %load26 = load ptr, ptr %self, align 8
  %ptr2int27 = ptrtoint ptr %load26 to i64
  %cmpne28 = icmp ne i64 %ptr2int27, 0
  %zext29 = zext i1 %cmpne28 to i64
  %cond_true30 = icmp ne i64 %zext29, 0
  br i1 %cond_true30, label %bb489, label %bb490

bb489:                                            ; preds = %bb488
  %load31 = load ptr, ptr %self, align 8
  %ptr2int32 = ptrtoint ptr %load31 to i64
  %cmpne33 = icmp ne i64 %ptr2int32, 0
  %zext34 = zext i1 %cmpne33 to i64
  %cond_true35 = icmp ne i64 %zext34, 0
  br i1 %cond_true35, label %bb491, label %bb492

bb490:                                            ; preds = %bb488
  call void @panic(ptr @global_str.20)
  unreachable

bb491:                                            ; preds = %bb489
  %gep36 = getelementptr %Vec_str, ptr %load31, i64 0, i32 1
  %load37 = load i64, ptr %gep36, align 4
  %sub = sub i64 %load37, 1
  %gep38 = getelementptr %Vec_str, ptr %load26, i64 0, i32 1
  store i64 %sub, ptr %gep38, align 4
  %load39 = load i64, ptr %index, align 4
  %load40 = load ptr, ptr %self, align 8
  %ptr2int41 = ptrtoint ptr %load40 to i64
  %cmpne42 = icmp ne i64 %ptr2int41, 0
  %zext43 = zext i1 %cmpne42 to i64
  %cond_true44 = icmp ne i64 %zext43, 0
  br i1 %cond_true44, label %bb493, label %bb494

bb492:                                            ; preds = %bb489
  call void @panic(ptr @global_str.19)
  unreachable

bb493:                                            ; preds = %bb491
  %gep45 = getelementptr %Vec_str, ptr %load40, i64 0, i32 1
  %load46 = load i64, ptr %gep45, align 4
  %cmplt47 = icmp ult i64 %load39, %load46
  %zext48 = zext i1 %cmplt47 to i64
  %cond_true49 = icmp ne i64 %zext48, 0
  br i1 %cond_true49, label %bb495, label %bb496

bb494:                                            ; preds = %bb491
  call void @panic(ptr @global_str.19)
  unreachable

bb495:                                            ; preds = %bb493
  %load50 = load ptr, ptr %self, align 8
  %ptr2int51 = ptrtoint ptr %load50 to i64
  %cmpne52 = icmp ne i64 %ptr2int51, 0
  %zext53 = zext i1 %cmpne52 to i64
  %cond_true54 = icmp ne i64 %zext53, 0
  br i1 %cond_true54, label %bb497, label %bb498

bb496:                                            ; preds = %bb512, %bb493
  %load55 = load ptr, ptr %removed_item, align 8
  ret ptr %load55

bb497:                                            ; preds = %bb495
  %gep56 = getelementptr %Vec_str, ptr %load50, i64 0, i32 0
  %load57 = load ptr, ptr %gep56, align 8
  %ptr2int58 = ptrtoint ptr %load57 to i64
  %cmpne59 = icmp ne i64 %ptr2int58, 0
  %zext60 = zext i1 %cmpne59 to i64
  %cond_true61 = icmp ne i64 %zext60, 0
  br i1 %cond_true61, label %bb499, label %bb500

bb498:                                            ; preds = %bb495
  call void @panic(ptr @global_str.19)
  unreachable

bb499:                                            ; preds = %bb497
  %load62 = load i64, ptr %index, align 4
  %gep63 = getelementptr i64, ptr %load57, i64 -1
  %load64 = load i64, ptr %gep63, align 4
  %cmplt65 = icmp slt i64 %load62, %load64
  %zext66 = zext i1 %cmplt65 to i64
  %cond_true67 = icmp ne i64 %zext66, 0
  br i1 %cond_true67, label %bb501, label %bb502

bb500:                                            ; preds = %bb497
  call void @panic(ptr @global_str.3)
  unreachable

bb501:                                            ; preds = %bb499
  %cmpge68 = icmp sge i64 %load62, 0
  %zext69 = zext i1 %cmpge68 to i64
  %cond_true70 = icmp ne i64 %zext69, 0
  br i1 %cond_true70, label %bb503, label %bb502

bb502:                                            ; preds = %bb501, %bb499
  call void @panic(ptr @global_str.4)
  unreachable

bb503:                                            ; preds = %bb501
  %load71 = load ptr, ptr %self, align 8
  %ptr2int72 = ptrtoint ptr %load71 to i64
  %cmpne73 = icmp ne i64 %ptr2int72, 0
  %zext74 = zext i1 %cmpne73 to i64
  %cond_true75 = icmp ne i64 %zext74, 0
  br i1 %cond_true75, label %bb504, label %bb505

bb504:                                            ; preds = %bb503
  %gep76 = getelementptr %Vec_str, ptr %load71, i64 0, i32 0
  %load77 = load ptr, ptr %gep76, align 8
  %ptr2int78 = ptrtoint ptr %load77 to i64
  %cmpne79 = icmp ne i64 %ptr2int78, 0
  %zext80 = zext i1 %cmpne79 to i64
  %cond_true81 = icmp ne i64 %zext80, 0
  br i1 %cond_true81, label %bb506, label %bb507

bb505:                                            ; preds = %bb503
  call void @panic(ptr @global_str.19)
  unreachable

bb506:                                            ; preds = %bb504
  %load82 = load ptr, ptr %self, align 8
  %ptr2int83 = ptrtoint ptr %load82 to i64
  %cmpne84 = icmp ne i64 %ptr2int83, 0
  %zext85 = zext i1 %cmpne84 to i64
  %cond_true86 = icmp ne i64 %zext85, 0
  br i1 %cond_true86, label %bb508, label %bb509

bb507:                                            ; preds = %bb504
  call void @panic(ptr @global_str.5)
  unreachable

bb508:                                            ; preds = %bb506
  %gep87 = getelementptr %Vec_str, ptr %load82, i64 0, i32 1
  %load88 = load i64, ptr %gep87, align 4
  %gep89 = getelementptr i64, ptr %load77, i64 -1
  %load90 = load i64, ptr %gep89, align 4
  %cmplt91 = icmp slt i64 %load88, %load90
  %zext92 = zext i1 %cmplt91 to i64
  %cond_true93 = icmp ne i64 %zext92, 0
  br i1 %cond_true93, label %bb510, label %bb511

bb509:                                            ; preds = %bb506
  call void @panic(ptr @global_str.19)
  unreachable

bb510:                                            ; preds = %bb508
  %cmpge94 = icmp sge i64 %load88, 0
  %zext95 = zext i1 %cmpge94 to i64
  %cond_true96 = icmp ne i64 %zext95, 0
  br i1 %cond_true96, label %bb512, label %bb511

bb511:                                            ; preds = %bb510, %bb508
  call void @panic(ptr @global_str.4)
  unreachable

bb512:                                            ; preds = %bb510
  %gep97 = getelementptr ptr, ptr %load77, i64 %load88
  %load98 = load ptr, ptr %gep97, align 8
  %gep99 = getelementptr ptr, ptr %load57, i64 %load62
  store ptr %load98, ptr %gep99, align 8
  br label %bb496
}

define void @Vec_str_clear(ptr %0) {
bb513:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb514, label %bb515

bb514:                                            ; preds = %bb513
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 1
  store i64 0, ptr %gep, align 4
  ret void

bb515:                                            ; preds = %bb513
  call void @panic(ptr @global_str.20)
  unreachable
}

define ptr @Vec_str_get(ptr %0, i64 %1) {
bb516:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %index = alloca i64, align 8
  store i64 %1, ptr %index, align 4
  %load = load i64, ptr %index, align 4
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb517, label %bb518

bb517:                                            ; preds = %bb516
  %gep = getelementptr %Vec_str, ptr %load1, i64 0, i32 1
  %load2 = load i64, ptr %gep, align 4
  %cmpge = icmp uge i64 %load, %load2
  %zext3 = zext i1 %cmpge to i64
  %cond_true4 = icmp ne i64 %zext3, 0
  br i1 %cond_true4, label %bb519, label %bb520

bb518:                                            ; preds = %bb516
  call void @panic(ptr @global_str.19)
  unreachable

bb519:                                            ; preds = %bb517
  call void @panic(ptr @global_str.26)
  br label %bb520

bb520:                                            ; preds = %bb519, %bb517
  %load5 = load ptr, ptr %self, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb521, label %bb522

bb521:                                            ; preds = %bb520
  %gep10 = getelementptr %Vec_str, ptr %load5, i64 0, i32 0
  %load11 = load ptr, ptr %gep10, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb523, label %bb524

bb522:                                            ; preds = %bb520
  call void @panic(ptr @global_str.19)
  unreachable

bb523:                                            ; preds = %bb521
  %load16 = load i64, ptr %index, align 4
  %gep17 = getelementptr i64, ptr %load11, i64 -1
  %load18 = load i64, ptr %gep17, align 4
  %cmplt = icmp slt i64 %load16, %load18
  %zext19 = zext i1 %cmplt to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb525, label %bb526

bb524:                                            ; preds = %bb521
  call void @panic(ptr @global_str.5)
  unreachable

bb525:                                            ; preds = %bb523
  %cmpge21 = icmp sge i64 %load16, 0
  %zext22 = zext i1 %cmpge21 to i64
  %cond_true23 = icmp ne i64 %zext22, 0
  br i1 %cond_true23, label %bb527, label %bb526

bb526:                                            ; preds = %bb525, %bb523
  call void @panic(ptr @global_str.4)
  unreachable

bb527:                                            ; preds = %bb525
  %gep24 = getelementptr ptr, ptr %load11, i64 %load16
  %load25 = load ptr, ptr %gep24, align 8
  ret ptr %load25
}

define void @Vec_str_set(ptr %0, i64 %1, ptr %2) {
bb528:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %index = alloca i64, align 8
  store i64 %1, ptr %index, align 4
  %value = alloca ptr, align 8
  store ptr %2, ptr %value, align 8
  %load = load i64, ptr %index, align 4
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb529, label %bb530

bb529:                                            ; preds = %bb528
  %gep = getelementptr %Vec_str, ptr %load1, i64 0, i32 1
  %load2 = load i64, ptr %gep, align 4
  %cmpge = icmp uge i64 %load, %load2
  %zext3 = zext i1 %cmpge to i64
  %cond_true4 = icmp ne i64 %zext3, 0
  br i1 %cond_true4, label %bb531, label %bb532

bb530:                                            ; preds = %bb528
  call void @panic(ptr @global_str.19)
  unreachable

bb531:                                            ; preds = %bb529
  call void @panic(ptr @global_str.27)
  br label %bb532

bb532:                                            ; preds = %bb531, %bb529
  %load5 = load ptr, ptr %self, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb533, label %bb534

bb533:                                            ; preds = %bb532
  %gep10 = getelementptr %Vec_str, ptr %load5, i64 0, i32 0
  %load11 = load ptr, ptr %gep10, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb535, label %bb536

bb534:                                            ; preds = %bb532
  call void @panic(ptr @global_str.19)
  unreachable

bb535:                                            ; preds = %bb533
  %load16 = load i64, ptr %index, align 4
  %gep17 = getelementptr i64, ptr %load11, i64 -1
  %load18 = load i64, ptr %gep17, align 4
  %cmplt = icmp slt i64 %load16, %load18
  %zext19 = zext i1 %cmplt to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb537, label %bb538

bb536:                                            ; preds = %bb533
  call void @panic(ptr @global_str.3)
  unreachable

bb537:                                            ; preds = %bb535
  %cmpge21 = icmp sge i64 %load16, 0
  %zext22 = zext i1 %cmpge21 to i64
  %cond_true23 = icmp ne i64 %zext22, 0
  br i1 %cond_true23, label %bb539, label %bb538

bb538:                                            ; preds = %bb537, %bb535
  call void @panic(ptr @global_str.4)
  unreachable

bb539:                                            ; preds = %bb537
  %load24 = load ptr, ptr %value, align 8
  %gep25 = getelementptr ptr, ptr %load11, i64 %load16
  store ptr %load24, ptr %gep25, align 8
  ret void
}

define i1 @Vec_str_is_empty(ptr %0) {
bb540:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb541, label %bb542

bb541:                                            ; preds = %bb540
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext2 = zext i1 %cmpeq to i64
  %ret_trunc = trunc i64 %zext2 to i1
  ret i1 %ret_trunc

bb542:                                            ; preds = %bb540
  call void @panic(ptr @global_str.19)
  unreachable
}

define ptr @Vec_str_into_array(ptr %0) {
bb543:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %array_ptr = alloca ptr, align 8
  store ptr null, ptr %array_ptr, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb544, label %bb545

bb544:                                            ; preds = %bb543
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  store ptr %load1, ptr %array_ptr, align 8
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb546, label %bb547

bb545:                                            ; preds = %bb543
  call void @panic(ptr @global_str.19)
  unreachable

bb546:                                            ; preds = %bb544
  %gep7 = getelementptr %Vec_str, ptr %load2, i64 0, i32 2
  store i64 0, ptr %gep7, align 4
  %load8 = load ptr, ptr %self, align 8
  %ptr2int9 = ptrtoint ptr %load8 to i64
  %cmpne10 = icmp ne i64 %ptr2int9, 0
  %zext11 = zext i1 %cmpne10 to i64
  %cond_true12 = icmp ne i64 %zext11, 0
  br i1 %cond_true12, label %bb548, label %bb549

bb547:                                            ; preds = %bb544
  call void @panic(ptr @global_str.20)
  unreachable

bb548:                                            ; preds = %bb546
  %gep13 = getelementptr %Vec_str, ptr %load8, i64 0, i32 1
  store i64 0, ptr %gep13, align 4
  %load14 = load ptr, ptr %self, align 8
  %ptr2int15 = ptrtoint ptr %load14 to i64
  %cmpne16 = icmp ne i64 %ptr2int15, 0
  %zext17 = zext i1 %cmpne16 to i64
  %cond_true18 = icmp ne i64 %zext17, 0
  br i1 %cond_true18, label %bb550, label %bb551

bb549:                                            ; preds = %bb546
  call void @panic(ptr @global_str.20)
  unreachable

bb550:                                            ; preds = %bb548
  %gep19 = getelementptr %Vec_str, ptr %load14, i64 0, i32 0
  store ptr null, ptr %gep19, align 8
  %load20 = load ptr, ptr %array_ptr, align 8
  ret ptr %load20

bb551:                                            ; preds = %bb548
  call void @panic(ptr @global_str.20)
  unreachable
}

define i64 @Vec_str_clone(ptr %0) {
bb552:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %new_vec = alloca ptr, align 8
  store ptr null, ptr %new_vec, align 8
  %struct_alloc = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 24, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %gep = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 0
  store ptr null, ptr %gep, align 8
  %gep1 = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 1
  store i64 0, ptr %gep1, align 4
  %gep2 = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 2
  store i64 0, ptr %gep2, align 4
  store ptr %data_ptr, ptr %new_vec, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb553, label %bb554

bb553:                                            ; preds = %bb552
  %gep3 = getelementptr %Vec_str, ptr %load, i64 0, i32 2
  %load4 = load i64, ptr %gep3, align 4
  %load5 = load ptr, ptr %new_vec, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb555, label %bb556

bb554:                                            ; preds = %bb552
  call void @panic(ptr @global_str.19)
  unreachable

bb555:                                            ; preds = %bb553
  call void @Vec_str_reserve(ptr %load5, i64 %load4)
  %load10 = load ptr, ptr %new_vec, align 8
  %ptr2int11 = ptrtoint ptr %load10 to i64
  %cmpne12 = icmp ne i64 %ptr2int11, 0
  %zext13 = zext i1 %cmpne12 to i64
  %cond_true14 = icmp ne i64 %zext13, 0
  br i1 %cond_true14, label %bb557, label %bb558

bb556:                                            ; preds = %bb553
  call void @panic(ptr @global_str.6)
  unreachable

bb557:                                            ; preds = %bb555
  %load15 = load ptr, ptr %self, align 8
  %ptr2int16 = ptrtoint ptr %load15 to i64
  %cmpne17 = icmp ne i64 %ptr2int16, 0
  %zext18 = zext i1 %cmpne17 to i64
  %cond_true19 = icmp ne i64 %zext18, 0
  br i1 %cond_true19, label %bb559, label %bb560

bb558:                                            ; preds = %bb555
  call void @panic(ptr @global_str.20)
  unreachable

bb559:                                            ; preds = %bb557
  %gep20 = getelementptr %Vec_str, ptr %load15, i64 0, i32 1
  %load21 = load i64, ptr %gep20, align 4
  %gep22 = getelementptr %Vec_str, ptr %load10, i64 0, i32 1
  store i64 %load21, ptr %gep22, align 4
  %load23 = load ptr, ptr %self, align 8
  %ptr2int24 = ptrtoint ptr %load23 to i64
  %cmpne25 = icmp ne i64 %ptr2int24, 0
  %zext26 = zext i1 %cmpne25 to i64
  %cond_true27 = icmp ne i64 %zext26, 0
  br i1 %cond_true27, label %bb561, label %bb562

bb560:                                            ; preds = %bb557
  call void @panic(ptr @global_str.19)
  unreachable

bb561:                                            ; preds = %bb559
  %gep28 = getelementptr %Vec_str, ptr %load23, i64 0, i32 1
  %load29 = load i64, ptr %gep28, align 4
  %cmpgt = icmp ugt i64 %load29, 0
  %zext30 = zext i1 %cmpgt to i64
  %cond_true31 = icmp ne i64 %zext30, 0
  br i1 %cond_true31, label %bb563, label %bb564

bb562:                                            ; preds = %bb559
  call void @panic(ptr @global_str.19)
  unreachable

bb563:                                            ; preds = %bb561
  %bytes_to_copy = alloca i64, align 8
  store i64 0, ptr %bytes_to_copy, align 4
  %load32 = load ptr, ptr %self, align 8
  %ptr2int33 = ptrtoint ptr %load32 to i64
  %cmpne34 = icmp ne i64 %ptr2int33, 0
  %zext35 = zext i1 %cmpne34 to i64
  %cond_true36 = icmp ne i64 %zext35, 0
  br i1 %cond_true36, label %bb565, label %bb566

bb564:                                            ; preds = %bb569, %bb561
  %load37 = load ptr, ptr %new_vec, align 8
  %load38 = load ptr, ptr %new_vec, align 8
  %ptr2int39 = ptrtoint ptr %load38 to i64
  %cmpne40 = icmp ne i64 %ptr2int39, 0
  %zext41 = zext i1 %cmpne40 to i64
  %cond_true42 = icmp ne i64 %zext41, 0
  br i1 %cond_true42, label %bb571, label %bb572

bb565:                                            ; preds = %bb563
  %gep43 = getelementptr %Vec_str, ptr %load32, i64 0, i32 1
  %load44 = load i64, ptr %gep43, align 4
  %mul = mul i64 %load44, 8
  store i64 %mul, ptr %bytes_to_copy, align 4
  %load45 = load ptr, ptr %new_vec, align 8
  %ptr2int46 = ptrtoint ptr %load45 to i64
  %cmpne47 = icmp ne i64 %ptr2int46, 0
  %zext48 = zext i1 %cmpne47 to i64
  %cond_true49 = icmp ne i64 %zext48, 0
  br i1 %cond_true49, label %bb567, label %bb568

bb566:                                            ; preds = %bb563
  call void @panic(ptr @global_str.19)
  unreachable

bb567:                                            ; preds = %bb565
  %gep50 = getelementptr %Vec_str, ptr %load45, i64 0, i32 0
  %load51 = load ptr, ptr %gep50, align 8
  %load52 = load ptr, ptr %self, align 8
  %ptr2int53 = ptrtoint ptr %load52 to i64
  %cmpne54 = icmp ne i64 %ptr2int53, 0
  %zext55 = zext i1 %cmpne54 to i64
  %cond_true56 = icmp ne i64 %zext55, 0
  br i1 %cond_true56, label %bb569, label %bb570

bb568:                                            ; preds = %bb565
  call void @panic(ptr @global_str.19)
  unreachable

bb569:                                            ; preds = %bb567
  %gep57 = getelementptr %Vec_str, ptr %load52, i64 0, i32 0
  %load58 = load ptr, ptr %gep57, align 8
  %load59 = load i64, ptr %bytes_to_copy, align 4
  %call = call ptr @memcpy(ptr %load51, ptr %load58, i64 %load59)
  br label %bb564

bb570:                                            ; preds = %bb567
  call void @panic(ptr @global_str.19)
  unreachable

bb571:                                            ; preds = %bb564
  %is_not_null = icmp ne ptr %load38, null
  br i1 %is_not_null, label %arc.release.do, label %arc.release.cont

bb572:                                            ; preds = %arc.release.cont, %bb564
  %ret_cast_int = ptrtoint ptr %load37 to i64
  ret i64 %ret_cast_int

arc.release.do:                                   ; preds = %bb571
  %ref_ptr = getelementptr i64, ptr %load38, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = sub i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  %is_zero = icmp eq i64 %new_count, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb571
  br label %bb572

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define float @Point3_f32_add3(ptr %0) {
bb573:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb574, label %bb575

bb574:                                            ; preds = %bb573
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb576, label %bb577

bb575:                                            ; preds = %bb573
  call void @panic(ptr @global_str.19)
  unreachable

bb576:                                            ; preds = %bb574
  %gep7 = getelementptr %Point3_f32, ptr %load2, i64 0, i32 1
  %load8 = load float, ptr %gep7, align 4
  %fadd = fadd float %load1, %load8
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb578, label %bb579

bb577:                                            ; preds = %bb574
  call void @panic(ptr @global_str.19)
  unreachable

bb578:                                            ; preds = %bb576
  %gep14 = getelementptr %Point3_f32, ptr %load9, i64 0, i32 2
  %load15 = load float, ptr %gep14, align 4
  %fadd16 = fadd float %fadd, %load15
  ret float %fadd16

bb579:                                            ; preds = %bb576
  call void @panic(ptr @global_str.19)
  unreachable
}

define float @Point3_f32_getX(ptr %0) {
bb580:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb581, label %bb582

bb581:                                            ; preds = %bb580
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb582:                                            ; preds = %bb580
  call void @panic(ptr @global_str.19)
  unreachable
}

define float @Point3_f32_getY(ptr %0) {
bb583:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb584, label %bb585

bb584:                                            ; preds = %bb583
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 1
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb585:                                            ; preds = %bb583
  call void @panic(ptr @global_str.19)
  unreachable
}

define float @Point3_f32_getZ(ptr %0) {
bb586:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb587, label %bb588

bb587:                                            ; preds = %bb586
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 2
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb588:                                            ; preds = %bb586
  call void @panic(ptr @global_str.19)
  unreachable
}

define i64 @Result_str_str_Ok(i64 %0) {
bb589:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 0, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i64 @Result_str_str_Err(i64 %0) {
bb590:
  %arr_alloc = call ptr @malloc(i64 32)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 2, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 1, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %0, ptr %gep1, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define i1 @Result_str_str_is_ok(ptr %0) {
bb591:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i1, align 1
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb593, label %bb594

bb592:                                            ; preds = %bb596, %bb595, %bb593
  %load2 = load i1, ptr %match_res, align 1
  %load_zext = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %load_zext to i1
  ret i1 %ret_trunc

bb593:                                            ; preds = %bb591
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb592

bb594:                                            ; preds = %bb591
  %cmpeq5 = icmp eq i64 %load1, 1
  %zext6 = zext i1 %cmpeq5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb595, label %bb596

bb595:                                            ; preds = %bb594
  %gep8 = getelementptr i64, ptr %load, i64 1
  %load9 = load i64, ptr %gep8, align 4
  %e = alloca i64, align 8
  store i64 %load9, ptr %e, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb592

bb596:                                            ; preds = %bb594
  br label %bb592
}

define i1 @Result_str_str_is_err(ptr %0) {
bb597:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i1, align 1
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb599, label %bb600

bb598:                                            ; preds = %bb602, %bb601, %bb599
  %load2 = load i1, ptr %match_res, align 1
  %load_zext = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %load_zext to i1
  ret i1 %ret_trunc

bb599:                                            ; preds = %bb597
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb598

bb600:                                            ; preds = %bb597
  %cmpeq5 = icmp eq i64 %load1, 1
  %zext6 = zext i1 %cmpeq5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb601, label %bb602

bb601:                                            ; preds = %bb600
  %gep8 = getelementptr i64, ptr %load, i64 1
  %load9 = load i64, ptr %gep8, align 4
  %e = alloca i64, align 8
  store i64 %load9, ptr %e, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb598

bb602:                                            ; preds = %bb600
  br label %bb598
}

define ptr @Result_str_str_unwrap(ptr %0) {
bb603:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i64, align 8
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb605, label %bb606

bb604:                                            ; preds = %bb608, %bb607, %bb605
  %load2 = load i64, ptr %match_res, align 4
  %ret_cast_ptr = inttoptr i64 %load2 to ptr
  ret ptr %ret_cast_ptr

bb605:                                            ; preds = %bb603
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  %load5 = load i64, ptr %v, align 4
  store i64 %load5, ptr %match_res, align 4
  br label %bb604

bb606:                                            ; preds = %bb603
  %cmpeq6 = icmp eq i64 %load1, 1
  %zext7 = zext i1 %cmpeq6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb607, label %bb608

bb607:                                            ; preds = %bb606
  %gep9 = getelementptr i64, ptr %load, i64 1
  %load10 = load i64, ptr %gep9, align 4
  %e = alloca i64, align 8
  store i64 %load10, ptr %e, align 4
  call void @panic(ptr @global_str.18)
  %dummy = alloca ptr, align 8
  store ptr null, ptr %dummy, align 8
  %load11 = load ptr, ptr %dummy, align 8
  %store_cast_int = ptrtoint ptr %load11 to i64
  store i64 %store_cast_int, ptr %match_res, align 4
  br label %bb604

bb608:                                            ; preds = %bb606
  br label %bb604
}

define ptr @Result_str_str_unwrap_or(ptr %0, ptr %1) {
bb609:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %default_val = alloca ptr, align 8
  store ptr %1, ptr %default_val, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i64, align 8
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb611, label %bb612

bb610:                                            ; preds = %bb614, %bb613, %bb611
  %load2 = load i64, ptr %match_res, align 4
  %ret_cast_ptr = inttoptr i64 %load2 to ptr
  ret ptr %ret_cast_ptr

bb611:                                            ; preds = %bb609
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  %load5 = load i64, ptr %v, align 4
  store i64 %load5, ptr %match_res, align 4
  br label %bb610

bb612:                                            ; preds = %bb609
  %cmpeq6 = icmp eq i64 %load1, 1
  %zext7 = zext i1 %cmpeq6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb613, label %bb614

bb613:                                            ; preds = %bb612
  %gep9 = getelementptr i64, ptr %load, i64 1
  %load10 = load i64, ptr %gep9, align 4
  %e = alloca i64, align 8
  store i64 %load10, ptr %e, align 4
  %load11 = load ptr, ptr %default_val, align 8
  %store_cast_int = ptrtoint ptr %load11 to i64
  store i64 %store_cast_int, ptr %match_res, align 4
  br label %bb610

bb614:                                            ; preds = %bb612
  br label %bb610
}

define void @__global_init() {
bb0:
  ret void
}
