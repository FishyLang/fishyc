; ModuleID = 'fishy_module'
source_filename = "fishy_module"

%String = type { ptr, i64, i64 }
%File = type { ptr }
%Point3_f32 = type { float, float, float }
%Vec_T = type { ptr, i64, i64 }
%Point3_T = type { i64, i64, i64 }

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.1 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.2 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.3 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.4 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.5 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@global_str.6 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.7 = private unnamed_addr constant [41 x i8] c"Null pointer dereference on Method Call!\00", align 1
@global_str.8 = private unnamed_addr constant [14 x i8] c"Hello, World!\00", align 1
@global_str.9 = private unnamed_addr constant [40 x i8] c"Null pointer dereference (Array Write)!\00", align 1
@global_str.10 = private unnamed_addr constant [60 x i8] c"Array index out of bounds! Invalid memory access prevented.\00", align 1
@global_str.11 = private unnamed_addr constant [39 x i8] c"Null pointer dereference (Array Read)!\00", align 1
@global_str.12 = private unnamed_addr constant [38 x i8] c"Called Result#unwrap on an Err value\0A\00", align 1

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
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb4, label %bb5

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
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb6, label %bb7

bb5:                                              ; preds = %bb3
  call void @panic(ptr @global_str.3)
  unreachable

bb6:                                              ; preds = %bb4
  %gep11 = getelementptr %String, ptr %load6, i64 0, i32 2
  %load12 = load i64, ptr %gep11, align 4
  %cmpgt = icmp sgt i64 %add5, %load12
  %zext13 = zext i1 %cmpgt to i64
  %cond_true14 = icmp ne i64 %zext13, 0
  br i1 %cond_true14, label %bb8, label %bb9

bb7:                                              ; preds = %bb4
  call void @panic(ptr @global_str.3)
  unreachable

bb8:                                              ; preds = %bb6
  %new_cap = alloca i64, align 8
  store i64 0, ptr %new_cap, align 4
  %load15 = load ptr, ptr %self, align 8
  %ptr2int16 = ptrtoint ptr %load15 to i64
  %cmpne17 = icmp ne i64 %ptr2int16, 0
  %zext18 = zext i1 %cmpne17 to i64
  %cond_true19 = icmp ne i64 %zext18, 0
  br i1 %cond_true19, label %bb10, label %bb11

bb9:                                              ; preds = %bb18, %bb6
  %load20 = load ptr, ptr %self, align 8
  %ptr2int21 = ptrtoint ptr %load20 to i64
  %cmpne22 = icmp ne i64 %ptr2int21, 0
  %zext23 = zext i1 %cmpne22 to i64
  %cond_true24 = icmp ne i64 %zext23, 0
  br i1 %cond_true24, label %bb20, label %bb21

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
  %cond_true31 = icmp ne i64 %zext30, 0
  br i1 %cond_true31, label %bb12, label %bb13

bb11:                                             ; preds = %bb8
  call void @panic(ptr @global_str.3)
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
  %cond_true38 = icmp ne i64 %zext37, 0
  br i1 %cond_true38, label %bb14, label %bb15

bb14:                                             ; preds = %bb13
  %load39 = load ptr, ptr %self, align 8
  %ptr2int40 = ptrtoint ptr %load39 to i64
  %cmpne41 = icmp ne i64 %ptr2int40, 0
  %zext42 = zext i1 %cmpne41 to i64
  %cond_true43 = icmp ne i64 %zext42, 0
  br i1 %cond_true43, label %bb16, label %bb17

bb15:                                             ; preds = %bb13
  call void @panic(ptr @global_str.4)
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
  %cond_true53 = icmp ne i64 %zext52, 0
  br i1 %cond_true53, label %bb18, label %bb19

bb17:                                             ; preds = %bb14
  call void @panic(ptr @global_str.3)
  unreachable

bb18:                                             ; preds = %bb16
  %load54 = load i64, ptr %new_cap, align 4
  %gep55 = getelementptr %String, ptr %load49, i64 0, i32 2
  store i64 %load54, ptr %gep55, align 4
  br label %bb9

bb19:                                             ; preds = %bb16
  call void @panic(ptr @global_str.4)
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
  %cond_true64 = icmp ne i64 %zext63, 0
  br i1 %cond_true64, label %bb22, label %bb23

bb21:                                             ; preds = %bb9
  call void @panic(ptr @global_str.3)
  unreachable

bb22:                                             ; preds = %bb20
  %load65 = load i64, ptr %new_len, align 4
  %gep66 = getelementptr %String, ptr %load60, i64 0, i32 1
  store i64 %load65, ptr %gep66, align 4
  ret void

bb23:                                             ; preds = %bb20
  call void @panic(ptr @global_str.4)
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
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb25, label %bb26

bb25:                                             ; preds = %bb24
  %gep = getelementptr %String, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str.5, ptr %load1)
  ret void

bb26:                                             ; preds = %bb24
  call void @panic(ptr @global_str.3)
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
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb28, label %bb29

bb28:                                             ; preds = %bb27
  %gep = getelementptr %String, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  call void @free(ptr %load1)
  %load2 = load ptr, ptr %self, align 8
  %gep3 = getelementptr i64, ptr %load2, i64 0
  store i64 0, ptr %gep3, align 4
  ret void

bb29:                                             ; preds = %bb27
  call void @panic(ptr @global_str.3)
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
  %cond_true12 = icmp ne i64 %zext11, 0
  br i1 %cond_true12, label %bb34, label %bb35

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
  %cond_true19 = icmp ne i64 %zext18, 0
  br i1 %cond_true19, label %bb36, label %bb37

bb35:                                             ; preds = %bb33
  call void @panic(ptr @global_str.3)
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
  %cond_true27 = icmp ne i64 %zext26, 0
  br i1 %cond_true27, label %bb38, label %bb39

bb37:                                             ; preds = %bb34
  call void @panic(ptr @global_str.3)
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
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb32, label %bb33

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
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb41, label %bb42

bb41:                                             ; preds = %bb40
  %gep = getelementptr %File, ptr %load1, i64 0, i32 0
  %load2 = load ptr, ptr %gep, align 8
  %call = call i32 @fputs(ptr %load, ptr %load2)
  ret void

bb42:                                             ; preds = %bb40
  call void @panic(ptr @global_str.3)
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
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb44, label %bb45

bb44:                                             ; preds = %bb43
  %gep = getelementptr %File, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  %call = call i32 @fclose(ptr %load1)
  ret void

bb45:                                             ; preds = %bb43
  call void @panic(ptr @global_str.3)
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
  %store_cast_int = ptrtoint ptr %data_ptr to i64
  store i64 %store_cast_int, ptr %point3, align 4
  %load = load i64, ptr %point3, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb50, label %bb51

bb50:                                             ; preds = %bb49
  %int_to_ptr = inttoptr i64 %load to ptr
  %is_not_null = icmp ne ptr %int_to_ptr, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb51:                                             ; preds = %arc.retain.cont, %bb49
  %cmpne3 = icmp ne i64 %load, 0
  %zext4 = zext i1 %cmpne3 to i64
  %cond_true5 = icmp ne i64 %zext4, 0
  br i1 %cond_true5, label %bb52, label %bb53

bb52:                                             ; preds = %bb51
  %auto_cast_ptr = inttoptr i64 %load to ptr
  %call = call float @Point3_f32_add3(ptr %auto_cast_ptr)
  %vararg_fpext = fpext float %call to double
  %call6 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext)
  %load7 = load i64, ptr %point3, align 4
  %cmpne8 = icmp ne i64 %load7, 0
  %zext9 = zext i1 %cmpne8 to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb54, label %bb55

bb53:                                             ; preds = %bb51
  call void @panic(ptr @global_str.7)
  unreachable

bb54:                                             ; preds = %bb52
  %int_to_ptr11 = inttoptr i64 %load7 to ptr
  %is_not_null12 = icmp ne ptr %int_to_ptr11, null
  br i1 %is_not_null12, label %arc.retain.do13, label %arc.retain.cont14

bb55:                                             ; preds = %arc.retain.cont14, %bb52
  %cmpne18 = icmp ne i64 %load7, 0
  %zext19 = zext i1 %cmpne18 to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb56, label %bb57

bb56:                                             ; preds = %bb55
  %auto_cast_ptr21 = inttoptr i64 %load7 to ptr
  %call22 = call float @Point3_f32_getX(ptr %auto_cast_ptr21)
  %vararg_fpext23 = fpext float %call22 to double
  %call24 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext23)
  %load25 = load i64, ptr %point3, align 4
  %cmpne26 = icmp ne i64 %load25, 0
  %zext27 = zext i1 %cmpne26 to i64
  %cond_true28 = icmp ne i64 %zext27, 0
  br i1 %cond_true28, label %bb58, label %bb59

bb57:                                             ; preds = %bb55
  call void @panic(ptr @global_str.7)
  unreachable

bb58:                                             ; preds = %bb56
  %int_to_ptr29 = inttoptr i64 %load25 to ptr
  %is_not_null30 = icmp ne ptr %int_to_ptr29, null
  br i1 %is_not_null30, label %arc.retain.do31, label %arc.retain.cont32

bb59:                                             ; preds = %arc.retain.cont32, %bb56
  %cmpne36 = icmp ne i64 %load25, 0
  %zext37 = zext i1 %cmpne36 to i64
  %cond_true38 = icmp ne i64 %zext37, 0
  br i1 %cond_true38, label %bb60, label %bb61

bb60:                                             ; preds = %bb59
  %auto_cast_ptr39 = inttoptr i64 %load25 to ptr
  %call40 = call float @Point3_f32_getY(ptr %auto_cast_ptr39)
  %vararg_fpext41 = fpext float %call40 to double
  %call42 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext41)
  %load43 = load i64, ptr %point3, align 4
  %cmpne44 = icmp ne i64 %load43, 0
  %zext45 = zext i1 %cmpne44 to i64
  %cond_true46 = icmp ne i64 %zext45, 0
  br i1 %cond_true46, label %bb62, label %bb63

bb61:                                             ; preds = %bb59
  call void @panic(ptr @global_str.7)
  unreachable

bb62:                                             ; preds = %bb60
  %int_to_ptr47 = inttoptr i64 %load43 to ptr
  %is_not_null48 = icmp ne ptr %int_to_ptr47, null
  br i1 %is_not_null48, label %arc.retain.do49, label %arc.retain.cont50

bb63:                                             ; preds = %arc.retain.cont50, %bb60
  %cmpne54 = icmp ne i64 %load43, 0
  %zext55 = zext i1 %cmpne54 to i64
  %cond_true56 = icmp ne i64 %zext55, 0
  br i1 %cond_true56, label %bb64, label %bb65

bb64:                                             ; preds = %bb63
  %auto_cast_ptr57 = inttoptr i64 %load43 to ptr
  %call58 = call float @Point3_f32_getZ(ptr %auto_cast_ptr57)
  %vararg_fpext59 = fpext float %call58 to double
  %call60 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext59)
  %call61 = call i32 (ptr, ...) @printf(ptr @global_str.6, double 0x3FF1980000000000)
  %call62 = call i32 (ptr, ...) @printf(ptr @global_str.6, double 0x3FF19999A0000000)
  %call63 = call i32 (ptr, ...) @printf(ptr @global_str.6, double 1.100000e+00)
  %str = alloca i64, align 8
  store i64 0, ptr %str, align 4
  %call64 = call ptr @String_from_cstr(ptr @global_str.8)
  %store_cast_int65 = ptrtoint ptr %call64 to i64
  store i64 %store_cast_int65, ptr %str, align 4
  %load66 = load i64, ptr %str, align 4
  %cmpne67 = icmp ne i64 %load66, 0
  %zext68 = zext i1 %cmpne67 to i64
  %cond_true69 = icmp ne i64 %zext68, 0
  br i1 %cond_true69, label %bb66, label %bb67

bb65:                                             ; preds = %bb63
  call void @panic(ptr @global_str.7)
  unreachable

bb66:                                             ; preds = %bb64
  %inttoptr = inttoptr i64 %load66 to ptr
  %gep70 = getelementptr i64, ptr %inttoptr, i64 0
  %load71 = load i64, ptr %gep70, align 4
  %call72 = call i32 (ptr, ...) @printf(ptr @global_str.5, i64 %load71)
  %load73 = load i64, ptr %point3, align 4
  %cmpne74 = icmp ne i64 %load73, 0
  %zext75 = zext i1 %cmpne74 to i64
  %cond_true76 = icmp ne i64 %zext75, 0
  br i1 %cond_true76, label %bb68, label %bb69

bb67:                                             ; preds = %bb64
  call void @panic(ptr @global_str.3)
  unreachable

bb68:                                             ; preds = %bb66
  %int_to_ptr77 = inttoptr i64 %load73 to ptr
  %is_not_null78 = icmp ne ptr %int_to_ptr77, null
  br i1 %is_not_null78, label %arc.release.do, label %arc.release.cont

bb69:                                             ; preds = %arc.release.cont, %bb66
  ret i64 0

arc.retain.do:                                    ; preds = %bb50
  %ref_ptr = getelementptr i64, ptr %int_to_ptr, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb50
  br label %bb51

arc.retain.do13:                                  ; preds = %bb54
  %ref_ptr15 = getelementptr i64, ptr %int_to_ptr11, i64 -2
  %current_count16 = load i64, ptr %ref_ptr15, align 4
  %new_count17 = add i64 %current_count16, 1
  store i64 %new_count17, ptr %ref_ptr15, align 4
  br label %arc.retain.cont14

arc.retain.cont14:                                ; preds = %arc.retain.do13, %bb54
  br label %bb55

arc.retain.do31:                                  ; preds = %bb58
  %ref_ptr33 = getelementptr i64, ptr %int_to_ptr29, i64 -2
  %current_count34 = load i64, ptr %ref_ptr33, align 4
  %new_count35 = add i64 %current_count34, 1
  store i64 %new_count35, ptr %ref_ptr33, align 4
  br label %arc.retain.cont32

arc.retain.cont32:                                ; preds = %arc.retain.do31, %bb58
  br label %bb59

arc.retain.do49:                                  ; preds = %bb62
  %ref_ptr51 = getelementptr i64, ptr %int_to_ptr47, i64 -2
  %current_count52 = load i64, ptr %ref_ptr51, align 4
  %new_count53 = add i64 %current_count52, 1
  store i64 %new_count53, ptr %ref_ptr51, align 4
  br label %arc.retain.cont50

arc.retain.cont50:                                ; preds = %arc.retain.do49, %bb62
  br label %bb63

arc.release.do:                                   ; preds = %bb68
  %ref_ptr79 = getelementptr i64, ptr %int_to_ptr77, i64 -2
  %current_count80 = load i64, ptr %ref_ptr79, align 4
  %new_count81 = sub i64 %current_count80, 1
  store i64 %new_count81, ptr %ref_ptr79, align 4
  %is_zero = icmp eq i64 %new_count81, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb68
  br label %bb69

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr79)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define i64 @Vec_T_init() {
bb70:
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
bb71:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %item = alloca i64, align 8
  %store_cast_int = ptrtoint ptr %1 to i64
  store i64 %store_cast_int, ptr %item, align 4
  %is_not_null = icmp ne ptr %1, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb72:                                             ; preds = %arc.retain.cont
  %gep = getelementptr %Vec_T, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb74, label %bb75

bb73:                                             ; preds = %arc.retain.cont
  call void @panic(ptr @global_str.3)
  unreachable

bb74:                                             ; preds = %bb72
  %gep7 = getelementptr %Vec_T, ptr %load2, i64 0, i32 2
  %load8 = load i64, ptr %gep7, align 4
  %cmpeq = icmp eq i64 %load1, %load8
  %zext9 = zext i1 %cmpeq to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb76, label %bb77

bb75:                                             ; preds = %bb72
  call void @panic(ptr @global_str.3)
  unreachable

bb76:                                             ; preds = %bb74
  %new_cap = alloca i64, align 8
  store i64 0, ptr %new_cap, align 4
  store i64 0, ptr %new_cap, align 4
  %load11 = load ptr, ptr %self, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb78, label %bb79

bb77:                                             ; preds = %bb96, %bb74
  %load16 = load ptr, ptr %self, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpne18 = icmp ne i64 %ptr2int17, 0
  %zext19 = zext i1 %cmpne18 to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb98, label %bb99

bb78:                                             ; preds = %bb76
  %gep21 = getelementptr %Vec_T, ptr %load11, i64 0, i32 2
  %load22 = load i64, ptr %gep21, align 4
  %cmpeq23 = icmp eq i64 %load22, 0
  %zext24 = zext i1 %cmpeq23 to i64
  %cond_true25 = icmp ne i64 %zext24, 0
  br i1 %cond_true25, label %bb80, label %bb82

bb79:                                             ; preds = %bb76
  call void @panic(ptr @global_str.3)
  unreachable

bb80:                                             ; preds = %bb78
  store i64 4, ptr %new_cap, align 4
  br label %bb81

bb81:                                             ; preds = %bb83, %bb80
  %size_in_bytes = alloca i64, align 8
  store i64 0, ptr %size_in_bytes, align 4
  %load26 = load i64, ptr %new_cap, align 4
  %mul = mul i64 %load26, 8
  store i64 %mul, ptr %size_in_bytes, align 4
  %load27 = load ptr, ptr %self, align 8
  %ptr2int28 = ptrtoint ptr %load27 to i64
  %cmpne29 = icmp ne i64 %ptr2int28, 0
  %zext30 = zext i1 %cmpne29 to i64
  %cond_true31 = icmp ne i64 %zext30, 0
  br i1 %cond_true31, label %bb85, label %bb86

bb82:                                             ; preds = %bb78
  %load32 = load ptr, ptr %self, align 8
  %ptr2int33 = ptrtoint ptr %load32 to i64
  %cmpne34 = icmp ne i64 %ptr2int33, 0
  %zext35 = zext i1 %cmpne34 to i64
  %cond_true36 = icmp ne i64 %zext35, 0
  br i1 %cond_true36, label %bb83, label %bb84

bb83:                                             ; preds = %bb82
  %gep37 = getelementptr %Vec_T, ptr %load32, i64 0, i32 2
  %load38 = load i64, ptr %gep37, align 4
  %mul39 = mul i64 %load38, 2
  store i64 %mul39, ptr %new_cap, align 4
  br label %bb81

bb84:                                             ; preds = %bb82
  call void @panic(ptr @global_str.3)
  unreachable

bb85:                                             ; preds = %bb81
  %gep40 = getelementptr %Vec_T, ptr %load27, i64 0, i32 2
  %load41 = load i64, ptr %gep40, align 4
  %cmpeq42 = icmp eq i64 %load41, 0
  %zext43 = zext i1 %cmpeq42 to i64
  %cond_true44 = icmp ne i64 %zext43, 0
  br i1 %cond_true44, label %bb87, label %bb89

bb86:                                             ; preds = %bb81
  call void @panic(ptr @global_str.3)
  unreachable

bb87:                                             ; preds = %bb85
  %load45 = load ptr, ptr %self, align 8
  %ptr2int46 = ptrtoint ptr %load45 to i64
  %cmpne47 = icmp ne i64 %ptr2int46, 0
  %zext48 = zext i1 %cmpne47 to i64
  %cond_true49 = icmp ne i64 %zext48, 0
  br i1 %cond_true49, label %bb90, label %bb91

bb88:                                             ; preds = %bb94, %bb90
  %load50 = load ptr, ptr %self, align 8
  %ptr2int51 = ptrtoint ptr %load50 to i64
  %cmpne52 = icmp ne i64 %ptr2int51, 0
  %zext53 = zext i1 %cmpne52 to i64
  %cond_true54 = icmp ne i64 %zext53, 0
  br i1 %cond_true54, label %bb96, label %bb97

bb89:                                             ; preds = %bb85
  %load55 = load ptr, ptr %self, align 8
  %ptr2int56 = ptrtoint ptr %load55 to i64
  %cmpne57 = icmp ne i64 %ptr2int56, 0
  %zext58 = zext i1 %cmpne57 to i64
  %cond_true59 = icmp ne i64 %zext58, 0
  br i1 %cond_true59, label %bb92, label %bb93

bb90:                                             ; preds = %bb87
  %load60 = load i64, ptr %size_in_bytes, align 4
  %call = call ptr @malloc(i64 %load60)
  %gep61 = getelementptr %Vec_T, ptr %load45, i64 0, i32 0
  store ptr %call, ptr %gep61, align 8
  br label %bb88

bb91:                                             ; preds = %bb87
  call void @panic(ptr @global_str.4)
  unreachable

bb92:                                             ; preds = %bb89
  %load62 = load ptr, ptr %self, align 8
  %ptr2int63 = ptrtoint ptr %load62 to i64
  %cmpne64 = icmp ne i64 %ptr2int63, 0
  %zext65 = zext i1 %cmpne64 to i64
  %cond_true66 = icmp ne i64 %zext65, 0
  br i1 %cond_true66, label %bb94, label %bb95

bb93:                                             ; preds = %bb89
  call void @panic(ptr @global_str.4)
  unreachable

bb94:                                             ; preds = %bb92
  %gep67 = getelementptr %Vec_T, ptr %load62, i64 0, i32 0
  %load68 = load ptr, ptr %gep67, align 8
  %load69 = load i64, ptr %size_in_bytes, align 4
  %call70 = call ptr @realloc(ptr %load68, i64 %load69)
  %gep71 = getelementptr %Vec_T, ptr %load55, i64 0, i32 0
  store ptr %call70, ptr %gep71, align 8
  br label %bb88

bb95:                                             ; preds = %bb92
  call void @panic(ptr @global_str.3)
  unreachable

bb96:                                             ; preds = %bb88
  %load72 = load i64, ptr %new_cap, align 4
  %gep73 = getelementptr %Vec_T, ptr %load50, i64 0, i32 2
  store i64 %load72, ptr %gep73, align 4
  br label %bb77

bb97:                                             ; preds = %bb88
  call void @panic(ptr @global_str.4)
  unreachable

bb98:                                             ; preds = %bb77
  %gep74 = getelementptr %Vec_T, ptr %load16, i64 0, i32 0
  %load75 = load ptr, ptr %gep74, align 8
  %ptr2int76 = ptrtoint ptr %load75 to i64
  %cmpne77 = icmp ne i64 %ptr2int76, 0
  %zext78 = zext i1 %cmpne77 to i64
  %cond_true79 = icmp ne i64 %zext78, 0
  br i1 %cond_true79, label %bb100, label %bb101

bb99:                                             ; preds = %bb77
  call void @panic(ptr @global_str.3)
  unreachable

bb100:                                            ; preds = %bb98
  %load80 = load ptr, ptr %self, align 8
  %ptr2int81 = ptrtoint ptr %load80 to i64
  %cmpne82 = icmp ne i64 %ptr2int81, 0
  %zext83 = zext i1 %cmpne82 to i64
  %cond_true84 = icmp ne i64 %zext83, 0
  br i1 %cond_true84, label %bb102, label %bb103

bb101:                                            ; preds = %bb98
  call void @panic(ptr @global_str.9)
  unreachable

bb102:                                            ; preds = %bb100
  %gep85 = getelementptr %Vec_T, ptr %load80, i64 0, i32 1
  %load86 = load i64, ptr %gep85, align 4
  %gep87 = getelementptr i64, ptr %load75, i64 -1
  %load88 = load i64, ptr %gep87, align 4
  %cmplt = icmp slt i64 %load86, %load88
  %zext89 = zext i1 %cmplt to i64
  %cond_true90 = icmp ne i64 %zext89, 0
  br i1 %cond_true90, label %bb104, label %bb105

bb103:                                            ; preds = %bb100
  call void @panic(ptr @global_str.3)
  unreachable

bb104:                                            ; preds = %bb102
  %cmpge = icmp sge i64 %load86, 0
  %zext91 = zext i1 %cmpge to i64
  %cond_true92 = icmp ne i64 %zext91, 0
  br i1 %cond_true92, label %bb106, label %bb105

bb105:                                            ; preds = %bb104, %bb102
  call void @panic(ptr @global_str.10)
  unreachable

bb106:                                            ; preds = %bb104
  %load93 = load i64, ptr %item, align 4
  %cmpne94 = icmp ne i64 %load93, 0
  %zext95 = zext i1 %cmpne94 to i64
  %cond_true96 = icmp ne i64 %zext95, 0
  br i1 %cond_true96, label %bb107, label %bb108

bb107:                                            ; preds = %bb106
  %int_to_ptr = inttoptr i64 %load93 to ptr
  %is_not_null97 = icmp ne ptr %int_to_ptr, null
  br i1 %is_not_null97, label %arc.retain.do98, label %arc.retain.cont99

bb108:                                            ; preds = %arc.retain.cont99, %bb106
  %gep103 = getelementptr i64, ptr %load75, i64 %load86
  store i64 %load93, ptr %gep103, align 4
  %load104 = load ptr, ptr %self, align 8
  %ptr2int105 = ptrtoint ptr %load104 to i64
  %cmpne106 = icmp ne i64 %ptr2int105, 0
  %zext107 = zext i1 %cmpne106 to i64
  %cond_true108 = icmp ne i64 %zext107, 0
  br i1 %cond_true108, label %bb109, label %bb110

bb109:                                            ; preds = %bb108
  %load109 = load ptr, ptr %self, align 8
  %ptr2int110 = ptrtoint ptr %load109 to i64
  %cmpne111 = icmp ne i64 %ptr2int110, 0
  %zext112 = zext i1 %cmpne111 to i64
  %cond_true113 = icmp ne i64 %zext112, 0
  br i1 %cond_true113, label %bb111, label %bb112

bb110:                                            ; preds = %bb108
  call void @panic(ptr @global_str.4)
  unreachable

bb111:                                            ; preds = %bb109
  %gep114 = getelementptr %Vec_T, ptr %load109, i64 0, i32 1
  %load115 = load i64, ptr %gep114, align 4
  %add = add i64 %load115, 1
  %gep116 = getelementptr %Vec_T, ptr %load104, i64 0, i32 1
  store i64 %add, ptr %gep116, align 4
  %load117 = load i64, ptr %item, align 4
  %cmpne118 = icmp ne i64 %load117, 0
  %zext119 = zext i1 %cmpne118 to i64
  %cond_true120 = icmp ne i64 %zext119, 0
  br i1 %cond_true120, label %bb113, label %bb114

bb112:                                            ; preds = %bb109
  call void @panic(ptr @global_str.3)
  unreachable

bb113:                                            ; preds = %bb111
  %int_to_ptr121 = inttoptr i64 %load117 to ptr
  %is_not_null122 = icmp ne ptr %int_to_ptr121, null
  br i1 %is_not_null122, label %arc.release.do, label %arc.release.cont

bb114:                                            ; preds = %arc.release.cont, %bb111
  ret void

arc.retain.do:                                    ; preds = %bb71
  %ref_ptr = getelementptr i64, ptr %1, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb71
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb72, label %bb73

arc.retain.do98:                                  ; preds = %bb107
  %ref_ptr100 = getelementptr i64, ptr %int_to_ptr, i64 -2
  %current_count101 = load i64, ptr %ref_ptr100, align 4
  %new_count102 = add i64 %current_count101, 1
  store i64 %new_count102, ptr %ref_ptr100, align 4
  br label %arc.retain.cont99

arc.retain.cont99:                                ; preds = %arc.retain.do98, %bb107
  br label %bb108

arc.release.do:                                   ; preds = %bb113
  %ref_ptr123 = getelementptr i64, ptr %int_to_ptr121, i64 -2
  %current_count124 = load i64, ptr %ref_ptr123, align 4
  %new_count125 = sub i64 %current_count124, 1
  store i64 %new_count125, ptr %ref_ptr123, align 4
  %is_zero = icmp eq i64 %new_count125, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb113
  br label %bb114

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr123)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define ptr @Vec_T_get(ptr %0, i64 %1) {
bb115:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %index = alloca i64, align 8
  store i64 %1, ptr %index, align 4
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb116, label %bb117

bb116:                                            ; preds = %bb115
  %gep = getelementptr %Vec_T, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  %ptr2int2 = ptrtoint ptr %load1 to i64
  %cmpne3 = icmp ne i64 %ptr2int2, 0
  %zext4 = zext i1 %cmpne3 to i64
  %cond_true5 = icmp ne i64 %zext4, 0
  br i1 %cond_true5, label %bb118, label %bb119

bb117:                                            ; preds = %bb115
  call void @panic(ptr @global_str.3)
  unreachable

bb118:                                            ; preds = %bb116
  %load6 = load i64, ptr %index, align 4
  %gep7 = getelementptr i64, ptr %load1, i64 -1
  %load8 = load i64, ptr %gep7, align 4
  %cmplt = icmp slt i64 %load6, %load8
  %zext9 = zext i1 %cmplt to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb120, label %bb121

bb119:                                            ; preds = %bb116
  call void @panic(ptr @global_str.11)
  unreachable

bb120:                                            ; preds = %bb118
  %cmpge = icmp sge i64 %load6, 0
  %zext11 = zext i1 %cmpge to i64
  %cond_true12 = icmp ne i64 %zext11, 0
  br i1 %cond_true12, label %bb122, label %bb121

bb121:                                            ; preds = %bb120, %bb118
  call void @panic(ptr @global_str.10)
  unreachable

bb122:                                            ; preds = %bb120
  %gep13 = getelementptr i64, ptr %load1, i64 %load6
  %load14 = load i64, ptr %gep13, align 4
  %ret_cast_ptr = inttoptr i64 %load14 to ptr
  ret ptr %ret_cast_ptr
}

define void @Vec_T_drop(ptr %0) {
bb123:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb124, label %bb125

bb124:                                            ; preds = %bb123
  %gep = getelementptr %Vec_T, ptr %load, i64 0, i32 2
  %load1 = load i64, ptr %gep, align 4
  %cmpgt = icmp sgt i64 %load1, 0
  %zext2 = zext i1 %cmpgt to i64
  %cond_true3 = icmp ne i64 %zext2, 0
  br i1 %cond_true3, label %bb126, label %bb127

bb125:                                            ; preds = %bb123
  call void @panic(ptr @global_str.3)
  unreachable

bb126:                                            ; preds = %bb124
  %load4 = load ptr, ptr %self, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb128, label %bb129

bb127:                                            ; preds = %bb128, %bb124
  ret void

bb128:                                            ; preds = %bb126
  %gep9 = getelementptr %Vec_T, ptr %load4, i64 0, i32 0
  %load10 = load ptr, ptr %gep9, align 8
  call void @free(ptr %load10)
  %load11 = load ptr, ptr %self, align 8
  %gep12 = getelementptr i64, ptr %load11, i64 0
  store i64 0, ptr %gep12, align 4
  br label %bb127

bb129:                                            ; preds = %bb126
  call void @panic(ptr @global_str.3)
  unreachable
}

define i64 @Result_T_E_Ok(i64 %0) {
bb130:
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
bb131:
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
bb132:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i1, align 1
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb134, label %bb135

bb133:                                            ; preds = %bb137, %bb136, %bb134
  %load2 = load i1, ptr %match_res, align 1
  %bool_zext = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %bool_zext to i1
  ret i1 %ret_trunc

bb134:                                            ; preds = %bb132
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb133

bb135:                                            ; preds = %bb132
  %cmpeq5 = icmp eq i64 %load1, 1
  %zext6 = zext i1 %cmpeq5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb136, label %bb137

bb136:                                            ; preds = %bb135
  %gep8 = getelementptr i64, ptr %load, i64 1
  %load9 = load i64, ptr %gep8, align 4
  %e = alloca i64, align 8
  store i64 %load9, ptr %e, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb133

bb137:                                            ; preds = %bb135
  br label %bb133
}

define i1 @Result_T_E_is_err(ptr %0) {
bb138:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i1, align 1
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb140, label %bb141

bb139:                                            ; preds = %bb143, %bb142, %bb140
  %load2 = load i1, ptr %match_res, align 1
  %bool_zext = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %bool_zext to i1
  ret i1 %ret_trunc

bb140:                                            ; preds = %bb138
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb139

bb141:                                            ; preds = %bb138
  %cmpeq5 = icmp eq i64 %load1, 1
  %zext6 = zext i1 %cmpeq5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb142, label %bb143

bb142:                                            ; preds = %bb141
  %gep8 = getelementptr i64, ptr %load, i64 1
  %load9 = load i64, ptr %gep8, align 4
  %e = alloca i64, align 8
  store i64 %load9, ptr %e, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb139

bb143:                                            ; preds = %bb141
  br label %bb139
}

define ptr @Result_T_E_unwrap(ptr %0) {
bb144:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %match_res = alloca i64, align 8
  %gep = getelementptr i64, ptr %load, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb146, label %bb147

bb145:                                            ; preds = %bb151, %bb149, %bb146
  %load2 = load i64, ptr %match_res, align 4
  %ret_cast_ptr = inttoptr i64 %load2 to ptr
  ret ptr %ret_cast_ptr

bb146:                                            ; preds = %bb144
  %gep3 = getelementptr i64, ptr %load, i64 1
  %load4 = load i64, ptr %gep3, align 4
  %v = alloca i64, align 8
  store i64 %load4, ptr %v, align 4
  %load5 = load i64, ptr %v, align 4
  store i64 %load5, ptr %match_res, align 4
  br label %bb145

bb147:                                            ; preds = %bb144
  %cmpeq6 = icmp eq i64 %load1, 1
  %zext7 = zext i1 %cmpeq6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb148, label %bb149

bb148:                                            ; preds = %bb147
  %gep9 = getelementptr i64, ptr %load, i64 1
  %load10 = load i64, ptr %gep9, align 4
  %e = alloca i64, align 8
  store i64 %load10, ptr %e, align 4
  %call = call i32 (ptr, ...) @printf(ptr @global_str.12)
  call void @exit(i32 1)
  %dummy = alloca i64, align 8
  store i64 0, ptr %dummy, align 4
  %load11 = load i64, ptr %dummy, align 4
  %cmpne = icmp ne i64 %load11, 0
  %zext12 = zext i1 %cmpne to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb150, label %bb151

bb149:                                            ; preds = %bb147
  br label %bb145

bb150:                                            ; preds = %bb148
  %int_to_ptr = inttoptr i64 %load11 to ptr
  %is_not_null = icmp ne ptr %int_to_ptr, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb151:                                            ; preds = %arc.retain.cont, %bb148
  store i64 %load11, ptr %match_res, align 4
  br label %bb145

arc.retain.do:                                    ; preds = %bb150
  %ref_ptr = getelementptr i64, ptr %int_to_ptr, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb150
  br label %bb151
}

define ptr @Point3_T_add3(ptr %0) {
bb152:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb153, label %bb154

bb153:                                            ; preds = %bb152
  %gep = getelementptr %Point3_T, ptr %load, i64 0, i32 0
  %load1 = load i64, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb155, label %bb156

bb154:                                            ; preds = %bb152
  call void @panic(ptr @global_str.3)
  unreachable

bb155:                                            ; preds = %bb153
  %gep7 = getelementptr %Point3_T, ptr %load2, i64 0, i32 1
  %load8 = load i64, ptr %gep7, align 4
  %add = add i64 %load1, %load8
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb157, label %bb158

bb156:                                            ; preds = %bb153
  call void @panic(ptr @global_str.3)
  unreachable

bb157:                                            ; preds = %bb155
  %gep14 = getelementptr %Point3_T, ptr %load9, i64 0, i32 2
  %load15 = load i64, ptr %gep14, align 4
  %add16 = add i64 %add, %load15
  %ret_cast_ptr = inttoptr i64 %add16 to ptr
  ret ptr %ret_cast_ptr

bb158:                                            ; preds = %bb155
  call void @panic(ptr @global_str.3)
  unreachable
}

define ptr @Point3_T_getX(ptr %0) {
bb159:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb160, label %bb161

bb160:                                            ; preds = %bb159
  %gep = getelementptr %Point3_T, ptr %load, i64 0, i32 0
  %load1 = load i64, ptr %gep, align 4
  %ret_cast_ptr = inttoptr i64 %load1 to ptr
  ret ptr %ret_cast_ptr

bb161:                                            ; preds = %bb159
  call void @panic(ptr @global_str.3)
  unreachable
}

define ptr @Point3_T_getY(ptr %0) {
bb162:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb163, label %bb164

bb163:                                            ; preds = %bb162
  %gep = getelementptr %Point3_T, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 4
  %ret_cast_ptr = inttoptr i64 %load1 to ptr
  ret ptr %ret_cast_ptr

bb164:                                            ; preds = %bb162
  call void @panic(ptr @global_str.3)
  unreachable
}

define ptr @Point3_T_getZ(ptr %0) {
bb165:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb166, label %bb167

bb166:                                            ; preds = %bb165
  %gep = getelementptr %Point3_T, ptr %load, i64 0, i32 2
  %load1 = load i64, ptr %gep, align 4
  %ret_cast_ptr = inttoptr i64 %load1 to ptr
  ret ptr %ret_cast_ptr

bb167:                                            ; preds = %bb165
  call void @panic(ptr @global_str.3)
  unreachable
}

define float @Point3_f32_add3(ptr %0) {
bb168:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb169, label %bb170

bb169:                                            ; preds = %bb168
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb171, label %bb172

bb170:                                            ; preds = %bb168
  call void @panic(ptr @global_str.3)
  unreachable

bb171:                                            ; preds = %bb169
  %gep7 = getelementptr %Point3_f32, ptr %load2, i64 0, i32 1
  %load8 = load float, ptr %gep7, align 4
  %fadd = fadd float %load1, %load8
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb173, label %bb174

bb172:                                            ; preds = %bb169
  call void @panic(ptr @global_str.3)
  unreachable

bb173:                                            ; preds = %bb171
  %gep14 = getelementptr %Point3_f32, ptr %load9, i64 0, i32 2
  %load15 = load float, ptr %gep14, align 4
  %fadd16 = fadd float %fadd, %load15
  ret float %fadd16

bb174:                                            ; preds = %bb171
  call void @panic(ptr @global_str.3)
  unreachable
}

define float @Point3_f32_getX(ptr %0) {
bb175:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb176, label %bb177

bb176:                                            ; preds = %bb175
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb177:                                            ; preds = %bb175
  call void @panic(ptr @global_str.3)
  unreachable
}

define float @Point3_f32_getY(ptr %0) {
bb178:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb179, label %bb180

bb179:                                            ; preds = %bb178
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 1
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb180:                                            ; preds = %bb178
  call void @panic(ptr @global_str.3)
  unreachable
}

define float @Point3_f32_getZ(ptr %0) {
bb181:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb182, label %bb183

bb182:                                            ; preds = %bb181
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 2
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb183:                                            ; preds = %bb181
  call void @panic(ptr @global_str.3)
  unreachable
}

define void @__global_init() {
bb0:
  ret void
}
