; ModuleID = 'fishy_module'
source_filename = "fishy_module"

%String = type { ptr, i64, i64 }
%File = type { ptr }
%Point3_f32 = type { float, float, float }

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.1 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.2 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.3 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.4 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.5 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@global_str.6 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.7 = private unnamed_addr constant [41 x i8] c"Null pointer dereference on Method Call!\00", align 1
@global_str.8 = private unnamed_addr constant [14 x i8] c"Hello, World!\00", align 1

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
  %len = alloca i64, align 8
  store i64 0, ptr %len, align 4
  %load = load ptr, ptr %cstr, align 8
  %call = call i64 @strlen(ptr %load)
  store i64 %call, ptr %len, align 4
  %capacity = alloca i64, align 8
  store i64 0, ptr %capacity, align 4
  %load1 = load i64, ptr %len, align 4
  %add = add i64 %load1, 1
  store i64 %add, ptr %capacity, align 4
  %ptr = alloca ptr, align 8
  store ptr null, ptr %ptr, align 8
  %load2 = load i64, ptr %capacity, align 4
  %call3 = call ptr @malloc(i64 %load2)
  store ptr %call3, ptr %ptr, align 8
  %load4 = load ptr, ptr %ptr, align 8
  %load5 = load ptr, ptr %cstr, align 8
  %load6 = load i64, ptr %capacity, align 4
  %call7 = call ptr @memcpy(ptr %load4, ptr %load5, i64 %load6)
  %struct_alloc = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 24, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %load8 = load ptr, ptr %ptr, align 8
  %gep = getelementptr %String, ptr %data_ptr, i64 0, i32 0
  store ptr %load8, ptr %gep, align 8
  %load9 = load i64, ptr %len, align 4
  %gep10 = getelementptr %String, ptr %data_ptr, i64 0, i32 1
  store i64 %load9, ptr %gep10, align 4
  %load11 = load i64, ptr %capacity, align 4
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

define i8 @u8_min() {
bb30:
  ret i8 0
}

define i8 @u8_max() {
bb31:
  ret i8 -1
}

define i16 @u16_min() {
bb32:
  ret i16 0
}

define i16 @u16_max() {
bb33:
  ret i16 -1
}

define i32 @u32_min() {
bb34:
  ret i32 0
}

define i32 @u32_max() {
bb35:
  ret i32 -1
}

define i64 @u64_min() {
bb36:
  ret i64 0
}

define i64 @u64_max() {
bb37:
  ret i64 -1
}

define i8 @i8_min() {
bb38:
  ret i8 -128
}

define i8 @i8_max() {
bb39:
  ret i8 127
}

define i16 @i16_min() {
bb40:
  ret i16 -32768
}

define i16 @i16_max() {
bb41:
  ret i16 32767
}

define i32 @i32_min() {
bb42:
  ret i32 -2147483648
}

define i32 @i32_max() {
bb43:
  ret i32 2147483647
}

define i64 @i64_min() {
bb44:
  ret i64 -9223372036854775808
}

define i64 @i64_max() {
bb45:
  ret i64 9223372036854775807
}

define half @f16_min() {
bb46:
  ret half 0xHFBFF
}

define half @f16_max() {
bb47:
  ret half 0xH7BFF
}

define ptr @File_open(ptr %0, ptr %1) {
bb48:
  %filename = alloca ptr, align 8
  store ptr %0, ptr %filename, align 8
  %mode = alloca ptr, align 8
  store ptr %1, ptr %mode, align 8
  %handle = alloca ptr, align 8
  store ptr null, ptr %handle, align 8
  %load = load ptr, ptr %filename, align 8
  %load1 = load ptr, ptr %mode, align 8
  %call = call ptr @fopen(ptr %load, ptr %load1)
  store ptr %call, ptr %handle, align 8
  %struct_alloc = call ptr @malloc(i64 24)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 8, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %load2 = load ptr, ptr %handle, align 8
  %gep = getelementptr %File, ptr %data_ptr, i64 0, i32 0
  store ptr %load2, ptr %gep, align 8
  ret ptr %data_ptr
}

define void @File_write_string(ptr %0, ptr %1) {
bb49:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %text = alloca %String, align 8
  store ptr %1, ptr %text, align 8
  %is_not_null = icmp ne ptr %1, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb50:                                             ; preds = %arc.retain.cont
  %struct_first_field1 = extractvalue %String %load, 0
  %struct_ptr2int2 = ptrtoint ptr %struct_first_field1 to i64
  %inttoptr = inttoptr i64 %struct_ptr2int2 to ptr
  %gep = getelementptr %String, ptr %inttoptr, i64 0, i32 0
  %load3 = load ptr, ptr %gep, align 8
  %load4 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load4 to i64
  %cmpne5 = icmp ne i64 %ptr2int, 0
  %zext6 = zext i1 %cmpne5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb52, label %bb53

bb51:                                             ; preds = %arc.retain.cont
  call void @panic(ptr @global_str.3)
  unreachable

bb52:                                             ; preds = %bb50
  %gep8 = getelementptr %File, ptr %load4, i64 0, i32 0
  %load9 = load ptr, ptr %gep8, align 8
  %call = call i32 @fputs(ptr %load3, ptr %load9)
  %load10 = load %String, ptr %text, align 8
  %struct_first_field11 = extractvalue %String %load10, 0
  %struct_ptr2int12 = ptrtoint ptr %struct_first_field11 to i64
  %cmpne13 = icmp ne i64 %struct_ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb54, label %bb55

bb53:                                             ; preds = %bb50
  call void @panic(ptr @global_str.3)
  unreachable

bb54:                                             ; preds = %bb52
  %struct_first_field16 = extractvalue %String %load10, 0
  %is_not_null17 = icmp ne ptr %struct_first_field16, null
  br i1 %is_not_null17, label %arc.release.do, label %arc.release.cont

bb55:                                             ; preds = %arc.release.cont, %bb52
  ret void

arc.retain.do:                                    ; preds = %bb49
  %ref_ptr = getelementptr i64, ptr %1, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb49
  %load = load %String, ptr %text, align 8
  %struct_first_field = extractvalue %String %load, 0
  %struct_ptr2int = ptrtoint ptr %struct_first_field to i64
  %cmpne = icmp ne i64 %struct_ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb50, label %bb51

arc.release.do:                                   ; preds = %bb54
  %ref_ptr18 = getelementptr i64, ptr %struct_first_field16, i64 -2
  %current_count19 = load i64, ptr %ref_ptr18, align 4
  %new_count20 = sub i64 %current_count19, 1
  store i64 %new_count20, ptr %ref_ptr18, align 4
  %is_zero = icmp eq i64 %new_count20, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb54
  br label %bb55

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr18)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define void @File_write_cstr(ptr %0, ptr %1) {
bb56:
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
  br i1 %cond_true, label %bb57, label %bb58

bb57:                                             ; preds = %bb56
  %gep = getelementptr %File, ptr %load1, i64 0, i32 0
  %load2 = load ptr, ptr %gep, align 8
  %call = call i32 @fputs(ptr %load, ptr %load2)
  ret void

bb58:                                             ; preds = %bb56
  call void @panic(ptr @global_str.3)
  unreachable
}

define void @File_close(ptr %0) {
bb59:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb60, label %bb61

bb60:                                             ; preds = %bb59
  %gep = getelementptr %File, ptr %load, i64 0, i32 0
  %load1 = load ptr, ptr %gep, align 8
  %call = call i32 @fclose(ptr %load1)
  ret void

bb61:                                             ; preds = %bb59
  call void @panic(ptr @global_str.3)
  unreachable
}

define void @File_drop(ptr %0) {
bb62:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  ret void
}

define i64 @Result_Ok(i64 %0) {
bb63:
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
bb64:
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
bb65:
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
  br i1 %cond_true, label %bb66, label %bb67

bb66:                                             ; preds = %bb65
  %call = call float @Point3_f32_add3(ptr %load)
  %vararg_fpext = fpext float %call to double
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext)
  %load4 = load ptr, ptr %point3, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb68, label %bb69

bb67:                                             ; preds = %bb65
  call void @panic(ptr @global_str.7)
  unreachable

bb68:                                             ; preds = %bb66
  %call9 = call float @Point3_f32_getX(ptr %load4)
  %vararg_fpext10 = fpext float %call9 to double
  %call11 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext10)
  %load12 = load ptr, ptr %point3, align 8
  %ptr2int13 = ptrtoint ptr %load12 to i64
  %cmpne14 = icmp ne i64 %ptr2int13, 0
  %zext15 = zext i1 %cmpne14 to i64
  %cond_true16 = icmp ne i64 %zext15, 0
  br i1 %cond_true16, label %bb70, label %bb71

bb69:                                             ; preds = %bb66
  call void @panic(ptr @global_str.7)
  unreachable

bb70:                                             ; preds = %bb68
  %call17 = call float @Point3_f32_getY(ptr %load12)
  %vararg_fpext18 = fpext float %call17 to double
  %call19 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext18)
  %load20 = load ptr, ptr %point3, align 8
  %ptr2int21 = ptrtoint ptr %load20 to i64
  %cmpne22 = icmp ne i64 %ptr2int21, 0
  %zext23 = zext i1 %cmpne22 to i64
  %cond_true24 = icmp ne i64 %zext23, 0
  br i1 %cond_true24, label %bb72, label %bb73

bb71:                                             ; preds = %bb68
  call void @panic(ptr @global_str.7)
  unreachable

bb72:                                             ; preds = %bb70
  %call25 = call float @Point3_f32_getZ(ptr %load20)
  %vararg_fpext26 = fpext float %call25 to double
  %call27 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext26)
  %call28 = call i32 (ptr, ...) @printf(ptr @global_str.6, double 0x3FF1980000000000)
  %call29 = call i32 (ptr, ...) @printf(ptr @global_str.6, double 0x3FF19999A0000000)
  %call30 = call i32 (ptr, ...) @printf(ptr @global_str.6, double 1.100000e+00)
  %str = alloca i64, align 8
  store i64 0, ptr %str, align 4
  %call31 = call ptr @String_from_cstr(ptr @global_str.8)
  %store_cast_int = ptrtoint ptr %call31 to i64
  store i64 %store_cast_int, ptr %str, align 4
  %load32 = load i64, ptr %str, align 4
  %cmpne33 = icmp ne i64 %load32, 0
  %zext34 = zext i1 %cmpne33 to i64
  %cond_true35 = icmp ne i64 %zext34, 0
  br i1 %cond_true35, label %bb74, label %bb75

bb73:                                             ; preds = %bb70
  call void @panic(ptr @global_str.7)
  unreachable

bb74:                                             ; preds = %bb72
  %inttoptr = inttoptr i64 %load32 to ptr
  %gep36 = getelementptr i64, ptr %inttoptr, i64 0
  %load37 = load i64, ptr %gep36, align 4
  %call38 = call i32 (ptr, ...) @printf(ptr @global_str.5, i64 %load37)
  %a = alloca i64, align 8
  store i64 0, ptr %a, align 4
  store i64 1, ptr %a, align 4
  %call39 = call half @f16_min()
  %vararg_fpext40 = fpext half %call39 to double
  %call41 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext40)
  %load42 = load ptr, ptr %point3, align 8
  %ptr2int43 = ptrtoint ptr %load42 to i64
  %cmpne44 = icmp ne i64 %ptr2int43, 0
  %zext45 = zext i1 %cmpne44 to i64
  %cond_true46 = icmp ne i64 %zext45, 0
  br i1 %cond_true46, label %bb76, label %bb77

bb75:                                             ; preds = %bb72
  call void @panic(ptr @global_str.3)
  unreachable

bb76:                                             ; preds = %bb74
  %is_not_null = icmp ne ptr %load42, null
  br i1 %is_not_null, label %arc.release.do, label %arc.release.cont

bb77:                                             ; preds = %arc.release.cont, %bb74
  ret i64 0

arc.release.do:                                   ; preds = %bb76
  %ref_ptr = getelementptr i64, ptr %load42, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = sub i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  %is_zero = icmp eq i64 %new_count, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb76
  br label %bb77

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define i64 @Result_T_E_Ok(i64 %0) {
bb78:
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
bb79:
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

define float @Point3_f32_add3(ptr %0) {
bb80:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb81, label %bb82

bb81:                                             ; preds = %bb80
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb83, label %bb84

bb82:                                             ; preds = %bb80
  call void @panic(ptr @global_str.3)
  unreachable

bb83:                                             ; preds = %bb81
  %gep7 = getelementptr %Point3_f32, ptr %load2, i64 0, i32 1
  %load8 = load float, ptr %gep7, align 4
  %fadd = fadd float %load1, %load8
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb85, label %bb86

bb84:                                             ; preds = %bb81
  call void @panic(ptr @global_str.3)
  unreachable

bb85:                                             ; preds = %bb83
  %gep14 = getelementptr %Point3_f32, ptr %load9, i64 0, i32 2
  %load15 = load float, ptr %gep14, align 4
  %fadd16 = fadd float %fadd, %load15
  ret float %fadd16

bb86:                                             ; preds = %bb83
  call void @panic(ptr @global_str.3)
  unreachable
}

define float @Point3_f32_getX(ptr %0) {
bb87:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb88, label %bb89

bb88:                                             ; preds = %bb87
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb89:                                             ; preds = %bb87
  call void @panic(ptr @global_str.3)
  unreachable
}

define float @Point3_f32_getY(ptr %0) {
bb90:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb91, label %bb92

bb91:                                             ; preds = %bb90
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 1
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb92:                                             ; preds = %bb90
  call void @panic(ptr @global_str.3)
  unreachable
}

define float @Point3_f32_getZ(ptr %0) {
bb93:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb94, label %bb95

bb94:                                             ; preds = %bb93
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 2
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb95:                                             ; preds = %bb93
  call void @panic(ptr @global_str.3)
  unreachable
}

define void @__global_init() {
bb0:
  ret void
}
