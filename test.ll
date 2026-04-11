; ModuleID = 'fishy_module'
source_filename = "fishy_module"

%String = type { ptr, i64, i64 }
%File = type { ptr }
%Point3_f16 = type { half, half, half }
%Vec_T = type { ptr, i64, i64 }
%Point3_T = type { i64, i64, i64 }

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.1 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.2 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.3 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.4 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.5 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.6 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.7 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.8 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.9 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.10 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.11 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@global_str.12 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.13 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.14 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.15 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.16 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.17 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.18 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.19 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.20 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.21 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.22 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.23 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.24 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.25 = private unnamed_addr constant [14 x i8] c"Hello, World!\00", align 1
@global_str.26 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@global_str.27 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.28 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.29 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.30 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.31 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.32 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.33 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.34 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.35 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.36 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.37 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.38 = private unnamed_addr constant [40 x i8] c"Null pointer dereference (Array Write)!\00", align 1
@global_str.39 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.40 = private unnamed_addr constant [60 x i8] c"Array index out of bounds! Invalid memory access prevented.\00", align 1
@global_str.41 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.42 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.43 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.44 = private unnamed_addr constant [39 x i8] c"Null pointer dereference (Array Read)!\00", align 1
@global_str.45 = private unnamed_addr constant [60 x i8] c"Array index out of bounds! Invalid memory access prevented.\00", align 1
@global_str.46 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.47 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.48 = private unnamed_addr constant [38 x i8] c"Called Result#unwrap on an Err value\0A\00", align 1
@global_str.49 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.50 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.51 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.52 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.53 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.54 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.55 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.56 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.57 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.58 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.59 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.60 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1

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

define ptr @String_from_cstr(ptr %0) {
bb2:
  %cstr = alloca ptr, align 8
  store ptr %0, ptr %cstr, align 8
  %length = alloca i64, align 8
  store i64 0, ptr %length, align 4
  %load = load ptr, ptr %cstr, align 8
  %call = call i64 @strlen(ptr %load)
  store i64 %call, ptr %length, align 4
  %cap = alloca i64, align 8
  store i64 0, ptr %cap, align 4
  %load1 = load i64, ptr %length, align 4
  %add = add i64 %load1, 1
  store i64 %add, ptr %cap, align 4
  %ptr = alloca ptr, align 8
  store ptr null, ptr %ptr, align 8
  %load2 = load i64, ptr %cap, align 4
  %call3 = call ptr @malloc(i64 %load2)
  store ptr %call3, ptr %ptr, align 8
  %load4 = load ptr, ptr %ptr, align 8
  %load5 = load ptr, ptr %cstr, align 8
  %load6 = load i64, ptr %cap, align 4
  %call7 = call ptr @memcpy(ptr %load4, ptr %load5, i64 %load6)
  %struct_alloc = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 24, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %load8 = load ptr, ptr %ptr, align 8
  %gep = getelementptr %String, ptr %data_ptr, i64 0, i32 0
  store ptr %load8, ptr %gep, align 8
  %load9 = load i64, ptr %length, align 4
  %gep10 = getelementptr %String, ptr %data_ptr, i64 0, i32 1
  store i64 %load9, ptr %gep10, align 4
  %load11 = load i64, ptr %cap, align 4
  %gep12 = getelementptr %String, ptr %data_ptr, i64 0, i32 2
  store i64 %load11, ptr %gep12, align 4
  ret ptr %data_ptr
}

define void @String_append_cstr(ptr %0, ptr %1) {
bb3:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %cstr = alloca ptr, align 8
  store ptr %1, ptr %cstr, align 8
  %add_len = alloca i64, align 8
  store i64 0, ptr %add_len, align 4
  %load = load ptr, ptr %cstr, align 8
  %call = call i64 @strlen(ptr %load)
  store i64 %call, ptr %add_len, align 4
  %new_len = alloca i64, align 8
  store i64 0, ptr %new_len, align 4
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb4, label %bb5

bb4:                                              ; preds = %bb3
  %gep = getelementptr %String, ptr %load1, i64 0, i32 1
  %load2 = load i64, ptr %gep, align 4
  %load3 = load i64, ptr %add_len, align 4
  %add = add i64 %load2, %load3
  store i64 %add, ptr %new_len, align 4
  %load4 = load i64, ptr %new_len, align 4
  %add5 = add i64 %load4, 1
  %load6 = load ptr, ptr %self, align 8
  %ptr2int7 = ptrtoint ptr %load6 to i64
  %cmpne8 = icmp ne i64 %ptr2int7, 0
  %zext9 = zext i1 %cmpne8 to i64
  %trunc10 = trunc i64 %zext9 to i1
  br i1 %trunc10, label %bb6, label %bb7

bb5:                                              ; preds = %bb3
  call void @panic(ptr @global_str.3)
  unreachable

bb6:                                              ; preds = %bb4
  %gep11 = getelementptr %String, ptr %load6, i64 0, i32 2
  %load12 = load i64, ptr %gep11, align 4
  %cmpgt = icmp sgt i64 %add5, %load12
  %zext13 = zext i1 %cmpgt to i64
  %trunc14 = trunc i64 %zext13 to i1
  br i1 %trunc14, label %bb8, label %bb9

bb7:                                              ; preds = %bb4
  call void @panic(ptr @global_str.4)
  unreachable

bb8:                                              ; preds = %bb6
  %new_cap = alloca i64, align 8
  store i64 0, ptr %new_cap, align 4
  %load15 = load ptr, ptr %self, align 8
  %ptr2int16 = ptrtoint ptr %load15 to i64
  %cmpne17 = icmp ne i64 %ptr2int16, 0
  %zext18 = zext i1 %cmpne17 to i64
  %trunc19 = trunc i64 %zext18 to i1
  br i1 %trunc19, label %bb10, label %bb11

bb9:                                              ; preds = %bb18, %bb6
  %load20 = load ptr, ptr %self, align 8
  %ptr2int21 = ptrtoint ptr %load20 to i64
  %cmpne22 = icmp ne i64 %ptr2int21, 0
  %zext23 = zext i1 %cmpne22 to i64
  %trunc24 = trunc i64 %zext23 to i1
  br i1 %trunc24, label %bb20, label %bb21

bb10:                                             ; preds = %bb8
  %gep25 = getelementptr %String, ptr %load15, i64 0, i32 2
  %load26 = load i64, ptr %gep25, align 4
  %mul = mul i64 %load26, 2
  store i64 %mul, ptr %new_cap, align 4
  %load27 = load i64, ptr %new_cap, align 4
  %load28 = load i64, ptr %new_len, align 4
  %add29 = add i64 %load28, 1
  %cmplt = icmp slt i64 %load27, %add29
  %zext30 = zext i1 %cmplt to i64
  %trunc31 = trunc i64 %zext30 to i1
  br i1 %trunc31, label %bb12, label %bb13

bb11:                                             ; preds = %bb8
  call void @panic(ptr @global_str.5)
  unreachable

bb12:                                             ; preds = %bb10
  %load32 = load i64, ptr %new_len, align 4
  %add33 = add i64 %load32, 1
  store i64 %add33, ptr %new_cap, align 4
  br label %bb13

bb13:                                             ; preds = %bb12, %bb10
  %load34 = load ptr, ptr %self, align 8
  %ptr2int35 = ptrtoint ptr %load34 to i64
  %cmpne36 = icmp ne i64 %ptr2int35, 0
  %zext37 = zext i1 %cmpne36 to i64
  %trunc38 = trunc i64 %zext37 to i1
  br i1 %trunc38, label %bb14, label %bb15

bb14:                                             ; preds = %bb13
  %load39 = load ptr, ptr %self, align 8
  %ptr2int40 = ptrtoint ptr %load39 to i64
  %cmpne41 = icmp ne i64 %ptr2int40, 0
  %zext42 = zext i1 %cmpne41 to i64
  %trunc43 = trunc i64 %zext42 to i1
  br i1 %trunc43, label %bb16, label %bb17

bb15:                                             ; preds = %bb13
  call void @panic(ptr @global_str.6)
  unreachable

bb16:                                             ; preds = %bb14
  %gep44 = getelementptr %String, ptr %load39, i64 0, i32 0
  %load45 = load ptr, ptr %gep44, align 8
  %load46 = load i64, ptr %new_cap, align 4
  %call47 = call ptr @realloc(ptr %load45, i64 %load46)
  %gep48 = getelementptr %String, ptr %load34, i64 0, i32 0
  store ptr %call47, ptr %gep48, align 8
  %load49 = load ptr, ptr %self, align 8
  %ptr2int50 = ptrtoint ptr %load49 to i64
  %cmpne51 = icmp ne i64 %ptr2int50, 0
  %zext52 = zext i1 %cmpne51 to i64
  %trunc53 = trunc i64 %zext52 to i1
  br i1 %trunc53, label %bb18, label %bb19

bb17:                                             ; preds = %bb14
  call void @panic(ptr @global_str.7)
  unreachable

bb18:                                             ; preds = %bb16
  %load54 = load i64, ptr %new_cap, align 4
  %gep55 = getelementptr %String, ptr %load49, i64 0, i32 2
  store i64 %load54, ptr %gep55, align 4
  br label %bb9

bb19:                                             ; preds = %bb16
  call void @panic(ptr @global_str.8)
  unreachable

bb20:                                             ; preds = %bb9
  %gep56 = getelementptr %String, ptr %load20, i64 0, i32 0
  %load57 = load ptr, ptr %gep56, align 8
  %load58 = load ptr, ptr %cstr, align 8
  %call59 = call ptr @strcat(ptr %load57, ptr %load58)
  %load60 = load ptr, ptr %self, align 8
  %ptr2int61 = ptrtoint ptr %load60 to i64
  %cmpne62 = icmp ne i64 %ptr2int61, 0
  %zext63 = zext i1 %cmpne62 to i64
  %trunc64 = trunc i64 %zext63 to i1
  br i1 %trunc64, label %bb22, label %bb23

bb21:                                             ; preds = %bb9
  call void @panic(ptr @global_str.9)
  unreachable

bb22:                                             ; preds = %bb20
  %load65 = load i64, ptr %new_len, align 4
  %gep66 = getelementptr %String, ptr %load60, i64 0, i32 1
  store i64 %load65, ptr %gep66, align 4
  ret void

bb23:                                             ; preds = %bb20
  call void @panic(ptr @global_str.10)
  unreachable
}

define void @String_print(ptr %0) {
bb24:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb25, label %bb26

bb25:                                             ; preds = %bb24
  %gep = getelementptr %String, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str.11, ptr %load1)
  ret void

bb26:                                             ; preds = %bb24
  call void @panic(ptr @global_str.12)
  unreachable
}

define void @String_drop(ptr %0) {
bb27:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb28, label %bb29

bb28:                                             ; preds = %bb27
  %gep = getelementptr %String, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  call void @free(ptr %load1)
  %load2 = load ptr, ptr %self, align 8
  %gep3 = getelementptr i64, ptr %load2, i64 0
  store i64 0, ptr %gep3, align 4
  ret void

bb29:                                             ; preds = %bb27
  call void @panic(ptr @global_str.13)
  unreachable
}

define ptr @File_open(ptr %0, ptr %1) {
bb30:
  %filename = alloca ptr, align 8
  store ptr %0, ptr %filename, align 8
  %mode = alloca ptr, align 8
  store ptr %1, ptr %mode, align 8
  %h = alloca ptr, align 8
  store ptr null, ptr %h, align 8
  %load = load ptr, ptr %filename, align 8
  %load1 = load ptr, ptr %mode, align 8
  %call = call ptr @fopen(ptr %load, ptr %load1)
  store ptr %call, ptr %h, align 8
  %struct_alloc = call ptr @malloc(i64 24)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 8, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %load2 = load ptr, ptr %h, align 8
  %gep = getelementptr %File, ptr %data_ptr, i64 0, i32 0
  store ptr %load2, ptr %gep, align 8
  ret ptr %data_ptr
}

define void @File_write_string(ptr %0, ptr %1) {
bb31:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %text = alloca %String, align 8
  store ptr %1, ptr %text, align 8
  %is_not_null = icmp ne ptr %1, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb32:                                             ; preds = %arc.retain.cont
  %struct_first_field1 = extractvalue %String %load, 0
  %is_not_null2 = icmp ne ptr %struct_first_field1, null
  br i1 %is_not_null2, label %arc.retain.do3, label %arc.retain.cont4

bb33:                                             ; preds = %arc.retain.cont4, %arc.retain.cont
  %struct_first_field8 = extractvalue %String %load, 0
  %struct_ptr2int9 = ptrtoint ptr %struct_first_field8 to i64
  %cmpne10 = icmp ne i64 %struct_ptr2int9, 0
  %zext11 = zext i1 %cmpne10 to i64
  %trunc12 = trunc i64 %zext11 to i1
  br i1 %trunc12, label %bb34, label %bb35

bb34:                                             ; preds = %bb33
  %struct_first_field13 = extractvalue %String %load, 0
  %struct_ptr2int14 = ptrtoint ptr %struct_first_field13 to i64
  %inttoptr = inttoptr i64 %struct_ptr2int14 to ptr
  %gep = getelementptr %String, ptr %inttoptr, i64 0, i32 0
  %load15 = load ptr, ptr %gep, align 8
  %load16 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load16 to i64
  %cmpne17 = icmp ne i64 %ptr2int, 0
  %zext18 = zext i1 %cmpne17 to i64
  %trunc19 = trunc i64 %zext18 to i1
  br i1 %trunc19, label %bb36, label %bb37

bb35:                                             ; preds = %bb33
  call void @panic(ptr @global_str.14)
  unreachable

bb36:                                             ; preds = %bb34
  %gep20 = getelementptr %File, ptr %load16, i64 0, i32 0
  %load21 = load ptr, ptr %gep20, align 8
  %call = call i32 @fputs(ptr %load15, ptr %load21)
  %load22 = load %String, ptr %text, align 8
  %struct_first_field23 = extractvalue %String %load22, 0
  %struct_ptr2int24 = ptrtoint ptr %struct_first_field23 to i64
  %cmpne25 = icmp ne i64 %struct_ptr2int24, 0
  %zext26 = zext i1 %cmpne25 to i64
  %trunc27 = trunc i64 %zext26 to i1
  br i1 %trunc27, label %bb38, label %bb39

bb37:                                             ; preds = %bb34
  call void @panic(ptr @global_str.15)
  unreachable

bb38:                                             ; preds = %bb36
  %struct_first_field28 = extractvalue %String %load22, 0
  %is_not_null29 = icmp ne ptr %struct_first_field28, null
  br i1 %is_not_null29, label %arc.release.do, label %arc.release.cont

bb39:                                             ; preds = %arc.release.cont, %bb36
  ret void

arc.retain.do:                                    ; preds = %bb31
  %ref_ptr = getelementptr i64, ptr %1, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb31
  %load = load %String, ptr %text, align 8
  %struct_first_field = extractvalue %String %load, 0
  %struct_ptr2int = ptrtoint ptr %struct_first_field to i64
  %cmpne = icmp ne i64 %struct_ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb32, label %bb33

arc.retain.do3:                                   ; preds = %bb32
  %ref_ptr5 = getelementptr i64, ptr %struct_first_field1, i64 -2
  %current_count6 = load i64, ptr %ref_ptr5, align 4
  %new_count7 = add i64 %current_count6, 1
  store i64 %new_count7, ptr %ref_ptr5, align 4
  br label %arc.retain.cont4

arc.retain.cont4:                                 ; preds = %arc.retain.do3, %bb32
  br label %bb33

arc.release.do:                                   ; preds = %bb38
  %ref_ptr30 = getelementptr i64, ptr %struct_first_field28, i64 -2
  %current_count31 = load i64, ptr %ref_ptr30, align 4
  %new_count32 = sub i64 %current_count31, 1
  store i64 %new_count32, ptr %ref_ptr30, align 4
  %is_zero = icmp eq i64 %new_count32, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb38
  br label %bb39

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr30)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define void @File_write_cstr(ptr %0, ptr %1) {
bb40:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %text = alloca ptr, align 8
  store ptr %1, ptr %text, align 8
  %load = load ptr, ptr %text, align 8
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb41, label %bb42

bb41:                                             ; preds = %bb40
  %gep = getelementptr %File, ptr %load1, i64 0, i32 0
  %load2 = load ptr, ptr %gep, align 8
  %call = call i32 @fputs(ptr %load, ptr %load2)
  ret void

bb42:                                             ; preds = %bb40
  call void @panic(ptr @global_str.16)
  unreachable
}

define void @File_close(ptr %0) {
bb43:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb44, label %bb45

bb44:                                             ; preds = %bb43
  %gep = getelementptr %File, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  %call = call i32 @fclose(ptr %load1)
  ret void

bb45:                                             ; preds = %bb43
  call void @panic(ptr @global_str.17)
  unreachable
}

define void @File_drop(ptr %0) {
bb46:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  ret void
}

define i64 @Result_Ok(i64 %0) {
bb47:
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
bb48:
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

define i64 @main() {
bb49:
  %point3 = alloca i64, align 8
  store i64 0, ptr %point3, align 4
  %struct_alloc = call ptr @malloc(i64 22)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 6, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %gep = getelementptr %Point3_f16, ptr %data_ptr, i64 0, i32 0
  store half 0xH4066, ptr %gep, align 2
  %gep1 = getelementptr %Point3_f16, ptr %data_ptr, i64 0, i32 1
  store half 0xH4466, ptr %gep1, align 2
  %gep2 = getelementptr %Point3_f16, ptr %data_ptr, i64 0, i32 2
  store half 0xH469A, ptr %gep2, align 2
  %store_cast_int = ptrtoint ptr %data_ptr to i64
  store i64 %store_cast_int, ptr %point3, align 4
  %load = load i64, ptr %point3, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb50, label %bb51

bb50:                                             ; preds = %bb49
  %inttoptr = inttoptr i64 %load to ptr
  %is_not_null = icmp ne ptr %inttoptr, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb51:                                             ; preds = %arc.retain.cont, %bb49
  %auto_cast_ptr = inttoptr i64 %load to ptr
  %call = call half @Point3_f16_add3(ptr %auto_cast_ptr)
  %vararg_fpext = fpext half %call to double
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.18, double %vararg_fpext)
  %load4 = load i64, ptr %point3, align 4
  %cmpne5 = icmp ne i64 %load4, 0
  %zext6 = zext i1 %cmpne5 to i64
  %trunc7 = trunc i64 %zext6 to i1
  br i1 %trunc7, label %bb52, label %bb53

bb52:                                             ; preds = %bb51
  %inttoptr8 = inttoptr i64 %load4 to ptr
  %is_not_null9 = icmp ne ptr %inttoptr8, null
  br i1 %is_not_null9, label %arc.retain.do10, label %arc.retain.cont11

bb53:                                             ; preds = %arc.retain.cont11, %bb51
  %auto_cast_ptr15 = inttoptr i64 %load4 to ptr
  %call16 = call half @Point3_f16_getX(ptr %auto_cast_ptr15)
  %vararg_fpext17 = fpext half %call16 to double
  %call18 = call i32 (ptr, ...) @printf(ptr @global_str.19, double %vararg_fpext17)
  %load19 = load i64, ptr %point3, align 4
  %cmpne20 = icmp ne i64 %load19, 0
  %zext21 = zext i1 %cmpne20 to i64
  %trunc22 = trunc i64 %zext21 to i1
  br i1 %trunc22, label %bb54, label %bb55

bb54:                                             ; preds = %bb53
  %inttoptr23 = inttoptr i64 %load19 to ptr
  %is_not_null24 = icmp ne ptr %inttoptr23, null
  br i1 %is_not_null24, label %arc.retain.do25, label %arc.retain.cont26

bb55:                                             ; preds = %arc.retain.cont26, %bb53
  %auto_cast_ptr30 = inttoptr i64 %load19 to ptr
  %call31 = call half @Point3_f16_getY(ptr %auto_cast_ptr30)
  %vararg_fpext32 = fpext half %call31 to double
  %call33 = call i32 (ptr, ...) @printf(ptr @global_str.20, double %vararg_fpext32)
  %load34 = load i64, ptr %point3, align 4
  %cmpne35 = icmp ne i64 %load34, 0
  %zext36 = zext i1 %cmpne35 to i64
  %trunc37 = trunc i64 %zext36 to i1
  br i1 %trunc37, label %bb56, label %bb57

bb56:                                             ; preds = %bb55
  %inttoptr38 = inttoptr i64 %load34 to ptr
  %is_not_null39 = icmp ne ptr %inttoptr38, null
  br i1 %is_not_null39, label %arc.retain.do40, label %arc.retain.cont41

bb57:                                             ; preds = %arc.retain.cont41, %bb55
  %auto_cast_ptr45 = inttoptr i64 %load34 to ptr
  %call46 = call half @Point3_f16_getZ(ptr %auto_cast_ptr45)
  %vararg_fpext47 = fpext half %call46 to double
  %call48 = call i32 (ptr, ...) @printf(ptr @global_str.21, double %vararg_fpext47)
  %call49 = call i32 (ptr, ...) @printf(ptr @global_str.22, double 0x3FF1980000000000)
  %call50 = call i32 (ptr, ...) @printf(ptr @global_str.23, double 0x3FF19999A0000000)
  %call51 = call i32 (ptr, ...) @printf(ptr @global_str.24, double 1.100000e+00)
  %str = alloca i64, align 8
  store i64 0, ptr %str, align 4
  %call52 = call ptr @String_from_cstr(ptr @global_str.25)
  %store_cast_int53 = ptrtoint ptr %call52 to i64
  store i64 %store_cast_int53, ptr %str, align 4
  %load54 = load i64, ptr %str, align 4
  %cmpne55 = icmp ne i64 %load54, 0
  %zext56 = zext i1 %cmpne55 to i64
  %trunc57 = trunc i64 %zext56 to i1
  br i1 %trunc57, label %bb58, label %bb59

bb58:                                             ; preds = %bb57
  %inttoptr58 = inttoptr i64 %load54 to ptr
  %gep59 = getelementptr i64, ptr %inttoptr58, i64 0
  %load60 = load i64, ptr %gep59, align 4
  %call61 = call i32 (ptr, ...) @printf(ptr @global_str.26, i64 %load60)
  %load62 = load i64, ptr %point3, align 4
  %cmpne63 = icmp ne i64 %load62, 0
  %zext64 = zext i1 %cmpne63 to i64
  %trunc65 = trunc i64 %zext64 to i1
  br i1 %trunc65, label %bb60, label %bb61

bb59:                                             ; preds = %bb57
  call void @panic(ptr @global_str.27)
  unreachable

bb60:                                             ; preds = %bb58
  %inttoptr66 = inttoptr i64 %load62 to ptr
  %is_not_null67 = icmp ne ptr %inttoptr66, null
  br i1 %is_not_null67, label %arc.release.do, label %arc.release.cont

bb61:                                             ; preds = %arc.release.cont, %bb58
  ret i64 0

arc.retain.do:                                    ; preds = %bb50
  %ref_ptr = getelementptr i64, ptr %inttoptr, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb50
  br label %bb51

arc.retain.do10:                                  ; preds = %bb52
  %ref_ptr12 = getelementptr i64, ptr %inttoptr8, i64 -2
  %current_count13 = load i64, ptr %ref_ptr12, align 4
  %new_count14 = add i64 %current_count13, 1
  store i64 %new_count14, ptr %ref_ptr12, align 4
  br label %arc.retain.cont11

arc.retain.cont11:                                ; preds = %arc.retain.do10, %bb52
  br label %bb53

arc.retain.do25:                                  ; preds = %bb54
  %ref_ptr27 = getelementptr i64, ptr %inttoptr23, i64 -2
  %current_count28 = load i64, ptr %ref_ptr27, align 4
  %new_count29 = add i64 %current_count28, 1
  store i64 %new_count29, ptr %ref_ptr27, align 4
  br label %arc.retain.cont26

arc.retain.cont26:                                ; preds = %arc.retain.do25, %bb54
  br label %bb55

arc.retain.do40:                                  ; preds = %bb56
  %ref_ptr42 = getelementptr i64, ptr %inttoptr38, i64 -2
  %current_count43 = load i64, ptr %ref_ptr42, align 4
  %new_count44 = add i64 %current_count43, 1
  store i64 %new_count44, ptr %ref_ptr42, align 4
  br label %arc.retain.cont41

arc.retain.cont41:                                ; preds = %arc.retain.do40, %bb56
  br label %bb57

arc.release.do:                                   ; preds = %bb60
  %ref_ptr68 = getelementptr i64, ptr %inttoptr66, i64 -2
  %current_count69 = load i64, ptr %ref_ptr68, align 4
  %new_count70 = sub i64 %current_count69, 1
  store i64 %new_count70, ptr %ref_ptr68, align 4
  %is_zero = icmp eq i64 %new_count70, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb60
  br label %bb61

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr68)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define i64 @Vec_T_init() {
bb62:
  %struct_alloc = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 24, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %gep = getelementptr %Vec_T, ptr %data_ptr, i64 0, i32 0
  store ptr null, ptr %gep, align 8
  %gep1 = getelementptr %Vec_T, ptr %data_ptr, i64 0, i32 1
  store i64 0, ptr %gep1, align 4
  %gep2 = getelementptr %Vec_T, ptr %data_ptr, i64 0, i32 2
  store i64 0, ptr %gep2, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define void @Vec_T_push(ptr %0, ptr %1) {
bb63:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %item = alloca i64, align 8
  %store_cast_int = ptrtoint ptr %1 to i64
  store i64 %store_cast_int, ptr %item, align 4
  %is_not_null = icmp ne ptr %1, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb64:                                             ; preds = %arc.retain.cont
  %gep = getelementptr %Vec_T, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %trunc6 = trunc i64 %zext5 to i1
  br i1 %trunc6, label %bb66, label %bb67

bb65:                                             ; preds = %arc.retain.cont
  call void @panic(ptr @global_str.28)
  unreachable

bb66:                                             ; preds = %bb64
  %gep7 = getelementptr %Vec_T, ptr %load2, i64 0, i32 2
  %load8 = load i64, ptr %gep7, align 4
  %cmpeq = icmp eq i64 %load1, %load8
  %zext9 = zext i1 %cmpeq to i64
  %trunc10 = trunc i64 %zext9 to i1
  br i1 %trunc10, label %bb68, label %bb69

bb67:                                             ; preds = %bb64
  call void @panic(ptr @global_str.29)
  unreachable

bb68:                                             ; preds = %bb66
  %new_cap = alloca i64, align 8
  store i64 0, ptr %new_cap, align 4
  store i64 0, ptr %new_cap, align 4
  %load11 = load ptr, ptr %self, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %trunc15 = trunc i64 %zext14 to i1
  br i1 %trunc15, label %bb70, label %bb71

bb69:                                             ; preds = %bb88, %bb66
  %load16 = load ptr, ptr %self, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpne18 = icmp ne i64 %ptr2int17, 0
  %zext19 = zext i1 %cmpne18 to i64
  %trunc20 = trunc i64 %zext19 to i1
  br i1 %trunc20, label %bb90, label %bb91

bb70:                                             ; preds = %bb68
  %gep21 = getelementptr %Vec_T, ptr %load11, i64 0, i32 2
  %load22 = load i64, ptr %gep21, align 4
  %cmpeq23 = icmp eq i64 %load22, 0
  %zext24 = zext i1 %cmpeq23 to i64
  %trunc25 = trunc i64 %zext24 to i1
  br i1 %trunc25, label %bb72, label %bb74

bb71:                                             ; preds = %bb68
  call void @panic(ptr @global_str.30)
  unreachable

bb72:                                             ; preds = %bb70
  store i64 4, ptr %new_cap, align 4
  br label %bb73

bb73:                                             ; preds = %bb75, %bb72
  %size_in_bytes = alloca i64, align 8
  store i64 0, ptr %size_in_bytes, align 4
  %load26 = load i64, ptr %new_cap, align 4
  %mul = mul i64 %load26, 8
  store i64 %mul, ptr %size_in_bytes, align 4
  %load27 = load ptr, ptr %self, align 8
  %ptr2int28 = ptrtoint ptr %load27 to i64
  %cmpne29 = icmp ne i64 %ptr2int28, 0
  %zext30 = zext i1 %cmpne29 to i64
  %trunc31 = trunc i64 %zext30 to i1
  br i1 %trunc31, label %bb77, label %bb78

bb74:                                             ; preds = %bb70
  %load32 = load ptr, ptr %self, align 8
  %ptr2int33 = ptrtoint ptr %load32 to i64
  %cmpne34 = icmp ne i64 %ptr2int33, 0
  %zext35 = zext i1 %cmpne34 to i64
  %trunc36 = trunc i64 %zext35 to i1
  br i1 %trunc36, label %bb75, label %bb76

bb75:                                             ; preds = %bb74
  %gep37 = getelementptr %Vec_T, ptr %load32, i64 0, i32 2
  %load38 = load i64, ptr %gep37, align 4
  %mul39 = mul i64 %load38, 2
  store i64 %mul39, ptr %new_cap, align 4
  br label %bb73

bb76:                                             ; preds = %bb74
  call void @panic(ptr @global_str.31)
  unreachable

bb77:                                             ; preds = %bb73
  %gep40 = getelementptr %Vec_T, ptr %load27, i64 0, i32 2
  %load41 = load i64, ptr %gep40, align 4
  %cmpeq42 = icmp eq i64 %load41, 0
  %zext43 = zext i1 %cmpeq42 to i64
  %trunc44 = trunc i64 %zext43 to i1
  br i1 %trunc44, label %bb79, label %bb81

bb78:                                             ; preds = %bb73
  call void @panic(ptr @global_str.32)
  unreachable

bb79:                                             ; preds = %bb77
  %load45 = load ptr, ptr %self, align 8
  %ptr2int46 = ptrtoint ptr %load45 to i64
  %cmpne47 = icmp ne i64 %ptr2int46, 0
  %zext48 = zext i1 %cmpne47 to i64
  %trunc49 = trunc i64 %zext48 to i1
  br i1 %trunc49, label %bb82, label %bb83

bb80:                                             ; preds = %bb86, %bb82
  %load50 = load ptr, ptr %self, align 8
  %ptr2int51 = ptrtoint ptr %load50 to i64
  %cmpne52 = icmp ne i64 %ptr2int51, 0
  %zext53 = zext i1 %cmpne52 to i64
  %trunc54 = trunc i64 %zext53 to i1
  br i1 %trunc54, label %bb88, label %bb89

bb81:                                             ; preds = %bb77
  %load55 = load ptr, ptr %self, align 8
  %ptr2int56 = ptrtoint ptr %load55 to i64
  %cmpne57 = icmp ne i64 %ptr2int56, 0
  %zext58 = zext i1 %cmpne57 to i64
  %trunc59 = trunc i64 %zext58 to i1
  br i1 %trunc59, label %bb84, label %bb85

bb82:                                             ; preds = %bb79
  %load60 = load i64, ptr %size_in_bytes, align 4
  %call = call ptr @malloc(i64 %load60)
  %gep61 = getelementptr %Vec_T, ptr %load45, i64 0, i32 0
  store ptr %call, ptr %gep61, align 8
  br label %bb80

bb83:                                             ; preds = %bb79
  call void @panic(ptr @global_str.33)
  unreachable

bb84:                                             ; preds = %bb81
  %load62 = load ptr, ptr %self, align 8
  %ptr2int63 = ptrtoint ptr %load62 to i64
  %cmpne64 = icmp ne i64 %ptr2int63, 0
  %zext65 = zext i1 %cmpne64 to i64
  %trunc66 = trunc i64 %zext65 to i1
  br i1 %trunc66, label %bb86, label %bb87

bb85:                                             ; preds = %bb81
  call void @panic(ptr @global_str.34)
  unreachable

bb86:                                             ; preds = %bb84
  %gep67 = getelementptr %Vec_T, ptr %load62, i64 0, i32 0
  %load68 = load ptr, ptr %gep67, align 8
  %load69 = load i64, ptr %size_in_bytes, align 4
  %call70 = call ptr @realloc(ptr %load68, i64 %load69)
  %gep71 = getelementptr %Vec_T, ptr %load55, i64 0, i32 0
  store ptr %call70, ptr %gep71, align 8
  br label %bb80

bb87:                                             ; preds = %bb84
  call void @panic(ptr @global_str.35)
  unreachable

bb88:                                             ; preds = %bb80
  %load72 = load i64, ptr %new_cap, align 4
  %gep73 = getelementptr %Vec_T, ptr %load50, i64 0, i32 2
  store i64 %load72, ptr %gep73, align 4
  br label %bb69

bb89:                                             ; preds = %bb80
  call void @panic(ptr @global_str.36)
  unreachable

bb90:                                             ; preds = %bb69
  %gep74 = getelementptr %Vec_T, ptr %load16, i64 0, i32 0
  %load75 = load ptr, ptr %gep74, align 8
  %ptr2int76 = ptrtoint ptr %load75 to i64
  %cmpne77 = icmp ne i64 %ptr2int76, 0
  %zext78 = zext i1 %cmpne77 to i64
  %trunc79 = trunc i64 %zext78 to i1
  br i1 %trunc79, label %bb92, label %bb93

bb91:                                             ; preds = %bb69
  call void @panic(ptr @global_str.37)
  unreachable

bb92:                                             ; preds = %bb90
  %load80 = load ptr, ptr %self, align 8
  %ptr2int81 = ptrtoint ptr %load80 to i64
  %cmpne82 = icmp ne i64 %ptr2int81, 0
  %zext83 = zext i1 %cmpne82 to i64
  %trunc84 = trunc i64 %zext83 to i1
  br i1 %trunc84, label %bb94, label %bb95

bb93:                                             ; preds = %bb90
  call void @panic(ptr @global_str.38)
  unreachable

bb94:                                             ; preds = %bb92
  %gep85 = getelementptr %Vec_T, ptr %load80, i64 0, i32 1
  %load86 = load i64, ptr %gep85, align 4
  %gep87 = getelementptr i64, ptr %load75, i64 -1
  %load88 = load i64, ptr %gep87, align 4
  %cmplt = icmp slt i64 %load86, %load88
  %zext89 = zext i1 %cmplt to i64
  %trunc90 = trunc i64 %zext89 to i1
  br i1 %trunc90, label %bb96, label %bb97

bb95:                                             ; preds = %bb92
  call void @panic(ptr @global_str.39)
  unreachable

bb96:                                             ; preds = %bb94
  %cmpge = icmp sge i64 %load86, 0
  %zext91 = zext i1 %cmpge to i64
  %trunc92 = trunc i64 %zext91 to i1
  br i1 %trunc92, label %bb98, label %bb97

bb97:                                             ; preds = %bb96, %bb94
  call void @panic(ptr @global_str.40)
  unreachable

bb98:                                             ; preds = %bb96
  %load93 = load i64, ptr %item, align 4
  %cmpne94 = icmp ne i64 %load93, 0
  %zext95 = zext i1 %cmpne94 to i64
  %trunc96 = trunc i64 %zext95 to i1
  br i1 %trunc96, label %bb99, label %bb100

bb99:                                             ; preds = %bb98
  %inttoptr = inttoptr i64 %load93 to ptr
  %is_not_null97 = icmp ne ptr %inttoptr, null
  br i1 %is_not_null97, label %arc.retain.do98, label %arc.retain.cont99

bb100:                                            ; preds = %arc.retain.cont99, %bb98
  %gep103 = getelementptr i64, ptr %load75, i64 %load86
  store i64 %load93, ptr %gep103, align 4
  %load104 = load ptr, ptr %self, align 8
  %ptr2int105 = ptrtoint ptr %load104 to i64
  %cmpne106 = icmp ne i64 %ptr2int105, 0
  %zext107 = zext i1 %cmpne106 to i64
  %trunc108 = trunc i64 %zext107 to i1
  br i1 %trunc108, label %bb101, label %bb102

bb101:                                            ; preds = %bb100
  %load109 = load ptr, ptr %self, align 8
  %ptr2int110 = ptrtoint ptr %load109 to i64
  %cmpne111 = icmp ne i64 %ptr2int110, 0
  %zext112 = zext i1 %cmpne111 to i64
  %trunc113 = trunc i64 %zext112 to i1
  br i1 %trunc113, label %bb103, label %bb104

bb102:                                            ; preds = %bb100
  call void @panic(ptr @global_str.41)
  unreachable

bb103:                                            ; preds = %bb101
  %gep114 = getelementptr %Vec_T, ptr %load109, i64 0, i32 1
  %load115 = load i64, ptr %gep114, align 4
  %add = add i64 %load115, 1
  %gep116 = getelementptr %Vec_T, ptr %load104, i64 0, i32 1
  store i64 %add, ptr %gep116, align 4
  %load117 = load i64, ptr %item, align 4
  %cmpne118 = icmp ne i64 %load117, 0
  %zext119 = zext i1 %cmpne118 to i64
  %trunc120 = trunc i64 %zext119 to i1
  br i1 %trunc120, label %bb105, label %bb106

bb104:                                            ; preds = %bb101
  call void @panic(ptr @global_str.42)
  unreachable

bb105:                                            ; preds = %bb103
  %inttoptr121 = inttoptr i64 %load117 to ptr
  %is_not_null122 = icmp ne ptr %inttoptr121, null
  br i1 %is_not_null122, label %arc.release.do, label %arc.release.cont

bb106:                                            ; preds = %arc.release.cont, %bb103
  ret void

arc.retain.do:                                    ; preds = %bb63
  %ref_ptr = getelementptr i64, ptr %1, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb63
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb64, label %bb65

arc.retain.do98:                                  ; preds = %bb99
  %ref_ptr100 = getelementptr i64, ptr %inttoptr, i64 -2
  %current_count101 = load i64, ptr %ref_ptr100, align 4
  %new_count102 = add i64 %current_count101, 1
  store i64 %new_count102, ptr %ref_ptr100, align 4
  br label %arc.retain.cont99

arc.retain.cont99:                                ; preds = %arc.retain.do98, %bb99
  br label %bb100

arc.release.do:                                   ; preds = %bb105
  %ref_ptr123 = getelementptr i64, ptr %inttoptr121, i64 -2
  %current_count124 = load i64, ptr %ref_ptr123, align 4
  %new_count125 = sub i64 %current_count124, 1
  store i64 %new_count125, ptr %ref_ptr123, align 4
  %is_zero = icmp eq i64 %new_count125, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb105
  br label %bb106

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr123)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define ptr @Vec_T_get(ptr %0, i64 %1) {
bb107:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %index = alloca i64, align 8
  store i64 %1, ptr %index, align 4
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb108, label %bb109

bb108:                                            ; preds = %bb107
  %gep = getelementptr %Vec_T, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  %ptr2int2 = ptrtoint ptr %load1 to i64
  %cmpne3 = icmp ne i64 %ptr2int2, 0
  %zext4 = zext i1 %cmpne3 to i64
  %trunc5 = trunc i64 %zext4 to i1
  br i1 %trunc5, label %bb110, label %bb111

bb109:                                            ; preds = %bb107
  call void @panic(ptr @global_str.43)
  unreachable

bb110:                                            ; preds = %bb108
  %load6 = load i64, ptr %index, align 4
  %gep7 = getelementptr i64, ptr %load1, i64 -1
  %load8 = load i64, ptr %gep7, align 4
  %cmplt = icmp slt i64 %load6, %load8
  %zext9 = zext i1 %cmplt to i64
  %trunc10 = trunc i64 %zext9 to i1
  br i1 %trunc10, label %bb112, label %bb113

bb111:                                            ; preds = %bb108
  call void @panic(ptr @global_str.44)
  unreachable

bb112:                                            ; preds = %bb110
  %cmpge = icmp sge i64 %load6, 0
  %zext11 = zext i1 %cmpge to i64
  %trunc12 = trunc i64 %zext11 to i1
  br i1 %trunc12, label %bb114, label %bb113

bb113:                                            ; preds = %bb112, %bb110
  call void @panic(ptr @global_str.45)
  unreachable

bb114:                                            ; preds = %bb112
  %gep13 = getelementptr i64, ptr %load1, i64 %load6
  %load14 = load i64, ptr %gep13, align 4
  %ret_cast_ptr = inttoptr i64 %load14 to ptr
  ret ptr %ret_cast_ptr
}

define void @Vec_T_drop(ptr %0) {
bb115:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb116, label %bb117

bb116:                                            ; preds = %bb115
  %gep = getelementptr %Vec_T, ptr %load, i64 0, i32 2
  %load1 = load i64, ptr %gep, align 4
  %cmpgt = icmp sgt i64 %load1, 0
  %zext2 = zext i1 %cmpgt to i64
  %trunc3 = trunc i64 %zext2 to i1
  br i1 %trunc3, label %bb118, label %bb119

bb117:                                            ; preds = %bb115
  call void @panic(ptr @global_str.46)
  unreachable

bb118:                                            ; preds = %bb116
  %load4 = load ptr, ptr %self, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %trunc8 = trunc i64 %zext7 to i1
  br i1 %trunc8, label %bb120, label %bb121

bb119:                                            ; preds = %bb120, %bb116
  ret void

bb120:                                            ; preds = %bb118
  %gep9 = getelementptr %Vec_T, ptr %load4, i64 0, i32 0
  %load10 = load ptr, ptr %gep9, align 8
  call void @free(ptr %load10)
  %load11 = load ptr, ptr %self, align 8
  %gep12 = getelementptr i64, ptr %load11, i64 0
  store i64 0, ptr %gep12, align 4
  br label %bb119

bb121:                                            ; preds = %bb118
  call void @panic(ptr @global_str.47)
  unreachable
}

define i64 @Result_T_E_Ok(i64 %0) {
bb122:
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
bb123:
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

define i1 @Result_T_E_is_ok(ptr %0) {
bb124:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i1, align 1
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb126, label %bb127

bb125:                                            ; preds = %bb129, %bb128, %bb126
  %load2 = load i1, ptr %match_res, align 1
  %zext3 = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %zext3 to i1
  ret i1 %ret_trunc

bb126:                                            ; preds = %bb124
  %gep4 = getelementptr i64, ptr %load, i64 1
  %load5 = load i64, ptr %gep4, align 4
  %v = alloca i64, align 8
  store i64 %load5, ptr %v, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb125

bb127:                                            ; preds = %bb124
  %cmpeq6 = icmp eq i64 %load1, 1
  %zext7 = zext i1 %cmpeq6 to i64
  %trunc8 = trunc i64 %zext7 to i1
  br i1 %trunc8, label %bb128, label %bb129

bb128:                                            ; preds = %bb127
  %gep9 = getelementptr i64, ptr %load, i64 1
  %load10 = load i64, ptr %gep9, align 4
  %e = alloca i64, align 8
  store i64 %load10, ptr %e, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb125

bb129:                                            ; preds = %bb127
  br label %bb125
}

define i1 @Result_T_E_is_err(ptr %0) {
bb130:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i1, align 1
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb132, label %bb133

bb131:                                            ; preds = %bb135, %bb134, %bb132
  %load2 = load i1, ptr %match_res, align 1
  %zext3 = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %zext3 to i1
  ret i1 %ret_trunc

bb132:                                            ; preds = %bb130
  %gep4 = getelementptr i64, ptr %load, i64 1
  %load5 = load i64, ptr %gep4, align 4
  %v = alloca i64, align 8
  store i64 %load5, ptr %v, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb131

bb133:                                            ; preds = %bb130
  %cmpeq6 = icmp eq i64 %load1, 1
  %zext7 = zext i1 %cmpeq6 to i64
  %trunc8 = trunc i64 %zext7 to i1
  br i1 %trunc8, label %bb134, label %bb135

bb134:                                            ; preds = %bb133
  %gep9 = getelementptr i64, ptr %load, i64 1
  %load10 = load i64, ptr %gep9, align 4
  %e = alloca i64, align 8
  store i64 %load10, ptr %e, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb131

bb135:                                            ; preds = %bb133
  br label %bb131
}

define ptr @Result_T_E_unwrap(ptr %0) {
bb136:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i64, align 8
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb138, label %bb139

bb137:                                            ; preds = %bb143, %bb141, %bb138
  %load2 = load i64, ptr %match_res, align 4
  %ret_cast_ptr = inttoptr i64 %load2 to ptr
  ret ptr %ret_cast_ptr

bb138:                                            ; preds = %bb136
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  %load5 = load i64, ptr %v, align 4
  store i64 %load5, ptr %match_res, align 4
  br label %bb137

bb139:                                            ; preds = %bb136
  %cmpeq6 = icmp eq i64 %load1, 1
  %zext7 = zext i1 %cmpeq6 to i64
  %trunc8 = trunc i64 %zext7 to i1
  br i1 %trunc8, label %bb140, label %bb141

bb140:                                            ; preds = %bb139
  %gep9 = getelementptr i64, ptr %load, i64 1
  %load10 = load i64, ptr %gep9, align 4
  %e = alloca i64, align 8
  store i64 %load10, ptr %e, align 4
  %call = call i32 (ptr, ...) @printf(ptr @global_str.48)
  call void @exit(i32 1)
  %dummy = alloca i64, align 8
  store i64 0, ptr %dummy, align 4
  %load11 = load i64, ptr %dummy, align 4
  %cmpne = icmp ne i64 %load11, 0
  %zext12 = zext i1 %cmpne to i64
  %trunc13 = trunc i64 %zext12 to i1
  br i1 %trunc13, label %bb142, label %bb143

bb141:                                            ; preds = %bb139
  br label %bb137

bb142:                                            ; preds = %bb140
  %inttoptr = inttoptr i64 %load11 to ptr
  %is_not_null = icmp ne ptr %inttoptr, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb143:                                            ; preds = %arc.retain.cont, %bb140
  store i64 %load11, ptr %match_res, align 4
  br label %bb137

arc.retain.do:                                    ; preds = %bb142
  %ref_ptr = getelementptr i64, ptr %inttoptr, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb142
  br label %bb143
}

define ptr @Point3_T_add3(ptr %0) {
bb144:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb145, label %bb146

bb145:                                            ; preds = %bb144
  %gep = getelementptr %Point3_T, ptr %load, i64 0, i32 0
  %load1 = load i64, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %trunc6 = trunc i64 %zext5 to i1
  br i1 %trunc6, label %bb147, label %bb148

bb146:                                            ; preds = %bb144
  call void @panic(ptr @global_str.49)
  unreachable

bb147:                                            ; preds = %bb145
  %gep7 = getelementptr %Point3_T, ptr %load2, i64 0, i32 1
  %load8 = load i64, ptr %gep7, align 4
  %add = add i64 %load1, %load8
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %trunc13 = trunc i64 %zext12 to i1
  br i1 %trunc13, label %bb149, label %bb150

bb148:                                            ; preds = %bb145
  call void @panic(ptr @global_str.50)
  unreachable

bb149:                                            ; preds = %bb147
  %gep14 = getelementptr %Point3_T, ptr %load9, i64 0, i32 2
  %load15 = load i64, ptr %gep14, align 4
  %add16 = add i64 %add, %load15
  %ret_cast_ptr = inttoptr i64 %add16 to ptr
  ret ptr %ret_cast_ptr

bb150:                                            ; preds = %bb147
  call void @panic(ptr @global_str.51)
  unreachable
}

define ptr @Point3_T_getX(ptr %0) {
bb151:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb152, label %bb153

bb152:                                            ; preds = %bb151
  %gep = getelementptr %Point3_T, ptr %load, i64 0, i32 0
  %load1 = load i64, ptr %gep, align 4
  %ret_cast_ptr = inttoptr i64 %load1 to ptr
  ret ptr %ret_cast_ptr

bb153:                                            ; preds = %bb151
  call void @panic(ptr @global_str.52)
  unreachable
}

define ptr @Point3_T_getY(ptr %0) {
bb154:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb155, label %bb156

bb155:                                            ; preds = %bb154
  %gep = getelementptr %Point3_T, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 4
  %ret_cast_ptr = inttoptr i64 %load1 to ptr
  ret ptr %ret_cast_ptr

bb156:                                            ; preds = %bb154
  call void @panic(ptr @global_str.53)
  unreachable
}

define ptr @Point3_T_getZ(ptr %0) {
bb157:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb158, label %bb159

bb158:                                            ; preds = %bb157
  %gep = getelementptr %Point3_T, ptr %load, i64 0, i32 2
  %load1 = load i64, ptr %gep, align 4
  %ret_cast_ptr = inttoptr i64 %load1 to ptr
  ret ptr %ret_cast_ptr

bb159:                                            ; preds = %bb157
  call void @panic(ptr @global_str.54)
  unreachable
}

define half @Point3_f16_add3(ptr %0) {
bb160:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb161, label %bb162

bb161:                                            ; preds = %bb160
  %gep = getelementptr %Point3_f16, ptr %load, i64 0, i32 0
  %load1 = load half, ptr %gep, align 2
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %trunc6 = trunc i64 %zext5 to i1
  br i1 %trunc6, label %bb163, label %bb164

bb162:                                            ; preds = %bb160
  call void @panic(ptr @global_str.55)
  unreachable

bb163:                                            ; preds = %bb161
  %gep7 = getelementptr %Point3_f16, ptr %load2, i64 0, i32 1
  %load8 = load half, ptr %gep7, align 2
  %fadd = fadd half %load1, %load8
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %trunc13 = trunc i64 %zext12 to i1
  br i1 %trunc13, label %bb165, label %bb166

bb164:                                            ; preds = %bb161
  call void @panic(ptr @global_str.56)
  unreachable

bb165:                                            ; preds = %bb163
  %gep14 = getelementptr %Point3_f16, ptr %load9, i64 0, i32 2
  %load15 = load half, ptr %gep14, align 2
  %fadd16 = fadd half %fadd, %load15
  ret half %fadd16

bb166:                                            ; preds = %bb163
  call void @panic(ptr @global_str.57)
  unreachable
}

define half @Point3_f16_getX(ptr %0) {
bb167:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb168, label %bb169

bb168:                                            ; preds = %bb167
  %gep = getelementptr %Point3_f16, ptr %load, i64 0, i32 0
  %load1 = load half, ptr %gep, align 2
  ret half %load1

bb169:                                            ; preds = %bb167
  call void @panic(ptr @global_str.58)
  unreachable
}

define half @Point3_f16_getY(ptr %0) {
bb170:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb171, label %bb172

bb171:                                            ; preds = %bb170
  %gep = getelementptr %Point3_f16, ptr %load, i64 0, i32 1
  %load1 = load half, ptr %gep, align 2
  ret half %load1

bb172:                                            ; preds = %bb170
  call void @panic(ptr @global_str.59)
  unreachable
}

define half @Point3_f16_getZ(ptr %0) {
bb173:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb174, label %bb175

bb174:                                            ; preds = %bb173
  %gep = getelementptr %Point3_f16, ptr %load, i64 0, i32 2
  %load1 = load half, ptr %gep, align 2
  ret half %load1

bb175:                                            ; preds = %bb173
  call void @panic(ptr @global_str.60)
  unreachable
}

define void @__global_init() {
bb0:
  ret void
}
