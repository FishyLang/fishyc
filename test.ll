; ModuleID = 'fishy_module'
source_filename = "fishy_module"

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
@global_str.18 = private unnamed_addr constant [43 x i8] c"Null pointer dereference no loop 'for in'!\00", align 1
@global_str.19 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.20 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.21 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.22 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.23 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.24 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.25 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.26 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.27 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.28 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.29 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.30 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.31 = private unnamed_addr constant [40 x i8] c"Null pointer dereference (Array Write)!\00", align 1
@global_str.32 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.33 = private unnamed_addr constant [60 x i8] c"Array index out of bounds! Invalid memory access prevented.\00", align 1
@global_str.34 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.35 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.36 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.37 = private unnamed_addr constant [39 x i8] c"Null pointer dereference (Array Read)!\00", align 1
@global_str.38 = private unnamed_addr constant [60 x i8] c"Array index out of bounds! Invalid memory access prevented.\00", align 1
@global_str.39 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.40 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.41 = private unnamed_addr constant [38 x i8] c"Called Result#unwrap on an Err value\0A\00", align 1
@global_str.42 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.43 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.44 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.45 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.46 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.47 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1

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
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  %store_cast_int = ptrtoint ptr %load8 to i64
  store i64 %store_cast_int, ptr %gep, align 4
  %load9 = load i64, ptr %length, align 4
  %gep10 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 %load9, ptr %gep10, align 4
  %load11 = load i64, ptr %cap, align 4
  %gep12 = getelementptr i64, ptr %data_ptr, i64 2
  store i64 %load11, ptr %gep12, align 4
  ret ptr %data_ptr
}

define void @String_append_cstr(i64 %0, ptr %1) {
bb3:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %cstr = alloca ptr, align 8
  store ptr %1, ptr %cstr, align 8
  %add_len = alloca i64, align 8
  store i64 0, ptr %add_len, align 4
  %load = load ptr, ptr %cstr, align 8
  %call = call i64 @strlen(ptr %load)
  store i64 %call, ptr %add_len, align 4
  %new_len = alloca i64, align 8
  store i64 0, ptr %new_len, align 4
  %load1 = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load1, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb4, label %bb5

bb4:                                              ; preds = %bb3
  %inttoptr = inttoptr i64 %load1 to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 1
  %load2 = load i64, ptr %gep, align 4
  %load3 = load i64, ptr %add_len, align 4
  %add = add i64 %load2, %load3
  store i64 %add, ptr %new_len, align 4
  %load4 = load i64, ptr %new_len, align 4
  %add5 = add i64 %load4, 1
  %load6 = load i64, ptr %self, align 4
  %cmpne7 = icmp ne i64 %load6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %trunc9 = trunc i64 %zext8 to i1
  br i1 %trunc9, label %bb6, label %bb7

bb5:                                              ; preds = %bb3
  call void @panic(ptr @global_str.3)
  unreachable

bb6:                                              ; preds = %bb4
  %inttoptr10 = inttoptr i64 %load6 to ptr
  %gep11 = getelementptr i64, ptr %inttoptr10, i64 2
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
  %load15 = load i64, ptr %self, align 4
  %cmpne16 = icmp ne i64 %load15, 0
  %zext17 = zext i1 %cmpne16 to i64
  %trunc18 = trunc i64 %zext17 to i1
  br i1 %trunc18, label %bb10, label %bb11

bb9:                                              ; preds = %bb18, %bb6
  %load19 = load i64, ptr %self, align 4
  %cmpne20 = icmp ne i64 %load19, 0
  %zext21 = zext i1 %cmpne20 to i64
  %trunc22 = trunc i64 %zext21 to i1
  br i1 %trunc22, label %bb20, label %bb21

bb10:                                             ; preds = %bb8
  %inttoptr23 = inttoptr i64 %load15 to ptr
  %gep24 = getelementptr i64, ptr %inttoptr23, i64 2
  %load25 = load i64, ptr %gep24, align 4
  %mul = mul i64 %load25, 2
  store i64 %mul, ptr %new_cap, align 4
  %load26 = load i64, ptr %new_cap, align 4
  %load27 = load i64, ptr %new_len, align 4
  %add28 = add i64 %load27, 1
  %cmplt = icmp slt i64 %load26, %add28
  %zext29 = zext i1 %cmplt to i64
  %trunc30 = trunc i64 %zext29 to i1
  br i1 %trunc30, label %bb12, label %bb13

bb11:                                             ; preds = %bb8
  call void @panic(ptr @global_str.5)
  unreachable

bb12:                                             ; preds = %bb10
  %load31 = load i64, ptr %new_len, align 4
  %add32 = add i64 %load31, 1
  store i64 %add32, ptr %new_cap, align 4
  br label %bb13

bb13:                                             ; preds = %bb12, %bb10
  %load33 = load i64, ptr %self, align 4
  %cmpne34 = icmp ne i64 %load33, 0
  %zext35 = zext i1 %cmpne34 to i64
  %trunc36 = trunc i64 %zext35 to i1
  br i1 %trunc36, label %bb14, label %bb15

bb14:                                             ; preds = %bb13
  %load37 = load i64, ptr %self, align 4
  %cmpne38 = icmp ne i64 %load37, 0
  %zext39 = zext i1 %cmpne38 to i64
  %trunc40 = trunc i64 %zext39 to i1
  br i1 %trunc40, label %bb16, label %bb17

bb15:                                             ; preds = %bb13
  call void @panic(ptr @global_str.6)
  unreachable

bb16:                                             ; preds = %bb14
  %inttoptr41 = inttoptr i64 %load37 to ptr
  %gep42 = getelementptr i64, ptr %inttoptr41, i64 0
  %load43 = load i64, ptr %gep42, align 4
  %load44 = load i64, ptr %new_cap, align 4
  %auto_cast_ptr = inttoptr i64 %load43 to ptr
  %call45 = call ptr @realloc(ptr %auto_cast_ptr, i64 %load44)
  %inttoptr46 = inttoptr i64 %load33 to ptr
  %gep47 = getelementptr i64, ptr %inttoptr46, i64 0
  %store_cast_int = ptrtoint ptr %call45 to i64
  store i64 %store_cast_int, ptr %gep47, align 4
  %load48 = load i64, ptr %self, align 4
  %cmpne49 = icmp ne i64 %load48, 0
  %zext50 = zext i1 %cmpne49 to i64
  %trunc51 = trunc i64 %zext50 to i1
  br i1 %trunc51, label %bb18, label %bb19

bb17:                                             ; preds = %bb14
  call void @panic(ptr @global_str.7)
  unreachable

bb18:                                             ; preds = %bb16
  %load52 = load i64, ptr %new_cap, align 4
  %inttoptr53 = inttoptr i64 %load48 to ptr
  %gep54 = getelementptr i64, ptr %inttoptr53, i64 2
  store i64 %load52, ptr %gep54, align 4
  br label %bb9

bb19:                                             ; preds = %bb16
  call void @panic(ptr @global_str.8)
  unreachable

bb20:                                             ; preds = %bb9
  %inttoptr55 = inttoptr i64 %load19 to ptr
  %gep56 = getelementptr i64, ptr %inttoptr55, i64 0
  %load57 = load i64, ptr %gep56, align 4
  %load58 = load ptr, ptr %cstr, align 8
  %auto_cast_ptr59 = inttoptr i64 %load57 to ptr
  %call60 = call ptr @strcat(ptr %auto_cast_ptr59, ptr %load58)
  %load61 = load i64, ptr %self, align 4
  %cmpne62 = icmp ne i64 %load61, 0
  %zext63 = zext i1 %cmpne62 to i64
  %trunc64 = trunc i64 %zext63 to i1
  br i1 %trunc64, label %bb22, label %bb23

bb21:                                             ; preds = %bb9
  call void @panic(ptr @global_str.9)
  unreachable

bb22:                                             ; preds = %bb20
  %load65 = load i64, ptr %new_len, align 4
  %inttoptr66 = inttoptr i64 %load61 to ptr
  %gep67 = getelementptr i64, ptr %inttoptr66, i64 1
  store i64 %load65, ptr %gep67, align 4
  ret void

bb23:                                             ; preds = %bb20
  call void @panic(ptr @global_str.10)
  unreachable
}

define void @String_print(i64 %0) {
bb24:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %load = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb25, label %bb26

bb25:                                             ; preds = %bb24
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 0
  %load1 = load i64, ptr %gep, align 4
  %inttoptr2 = inttoptr i64 %load1 to ptr
  %call = call i32 (ptr, ...) @printf(ptr @global_str.11, ptr %inttoptr2)
  ret void

bb26:                                             ; preds = %bb24
  call void @panic(ptr @global_str.12)
  unreachable
}

define void @String_drop(i64 %0) {
bb27:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %load = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb28, label %bb29

bb28:                                             ; preds = %bb27
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 0
  %load1 = load i64, ptr %gep, align 4
  %auto_cast_ptr = inttoptr i64 %load1 to ptr
  call void @free(ptr %auto_cast_ptr)
  %load2 = load i64, ptr %self, align 4
  %inttoptr3 = inttoptr i64 %load2 to ptr
  %gep4 = getelementptr i64, ptr %inttoptr3, i64 0
  store i64 0, ptr %gep4, align 4
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
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  %store_cast_int = ptrtoint ptr %load2 to i64
  store i64 %store_cast_int, ptr %gep, align 4
  ret ptr %data_ptr
}

define void @File_write_string(i64 %0, ptr %1) {
bb31:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %text = alloca ptr, align 8
  store ptr %1, ptr %text, align 8
  %is_not_null = icmp ne ptr %1, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb32:                                             ; preds = %arc.retain.cont
  %is_not_null1 = icmp ne ptr %load, null
  br i1 %is_not_null1, label %arc.retain.do2, label %arc.retain.cont3

bb33:                                             ; preds = %arc.retain.cont3, %arc.retain.cont
  %ptr2int7 = ptrtoint ptr %load to i64
  %cmpne8 = icmp ne i64 %ptr2int7, 0
  %zext9 = zext i1 %cmpne8 to i64
  %trunc10 = trunc i64 %zext9 to i1
  br i1 %trunc10, label %bb34, label %bb35

bb34:                                             ; preds = %bb33
  %gep = getelementptr ptr, ptr %load, i64 0
  %load11 = load i64, ptr %gep, align 4
  %inttoptr = inttoptr i64 %load11 to ptr
  %load12 = load i64, ptr %self, align 4
  %cmpne13 = icmp ne i64 %load12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %trunc15 = trunc i64 %zext14 to i1
  br i1 %trunc15, label %bb36, label %bb37

bb35:                                             ; preds = %bb33
  call void @panic(ptr @global_str.14)
  unreachable

bb36:                                             ; preds = %bb34
  %inttoptr16 = inttoptr i64 %load12 to ptr
  %gep17 = getelementptr i64, ptr %inttoptr16, i64 0
  %load18 = load i64, ptr %gep17, align 4
  %inttoptr19 = inttoptr i64 %load18 to ptr
  %call = call i32 @fputs(ptr %inttoptr, ptr %inttoptr19)
  %load20 = load ptr, ptr %text, align 8
  %ptr2int21 = ptrtoint ptr %load20 to i64
  %cmpne22 = icmp ne i64 %ptr2int21, 0
  %zext23 = zext i1 %cmpne22 to i64
  %trunc24 = trunc i64 %zext23 to i1
  br i1 %trunc24, label %bb38, label %bb39

bb37:                                             ; preds = %bb34
  call void @panic(ptr @global_str.15)
  unreachable

bb38:                                             ; preds = %bb36
  %is_not_null25 = icmp ne ptr %load20, null
  br i1 %is_not_null25, label %arc.release.do, label %arc.release.cont

bb39:                                             ; preds = %arc.release.cont, %bb36
  ret void

arc.retain.do:                                    ; preds = %bb31
  %ref_ptr = getelementptr i64, ptr %1, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb31
  %load = load ptr, ptr %text, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb32, label %bb33

arc.retain.do2:                                   ; preds = %bb32
  %ref_ptr4 = getelementptr i64, ptr %load, i64 -2
  %current_count5 = load i64, ptr %ref_ptr4, align 4
  %new_count6 = add i64 %current_count5, 1
  store i64 %new_count6, ptr %ref_ptr4, align 4
  br label %arc.retain.cont3

arc.retain.cont3:                                 ; preds = %arc.retain.do2, %bb32
  br label %bb33

arc.release.do:                                   ; preds = %bb38
  %ref_ptr26 = getelementptr i64, ptr %load20, i64 -2
  %current_count27 = load i64, ptr %ref_ptr26, align 4
  %new_count28 = sub i64 %current_count27, 1
  store i64 %new_count28, ptr %ref_ptr26, align 4
  %is_zero = icmp eq i64 %new_count28, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb38
  br label %bb39

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr26)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define void @File_write_cstr(i64 %0, ptr %1) {
bb40:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %text = alloca ptr, align 8
  store ptr %1, ptr %text, align 8
  %load = load ptr, ptr %text, align 8
  %load1 = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load1, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb41, label %bb42

bb41:                                             ; preds = %bb40
  %inttoptr = inttoptr i64 %load1 to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 0
  %load2 = load i64, ptr %gep, align 4
  %inttoptr3 = inttoptr i64 %load2 to ptr
  %call = call i32 @fputs(ptr %load, ptr %inttoptr3)
  ret void

bb42:                                             ; preds = %bb40
  call void @panic(ptr @global_str.16)
  unreachable
}

define void @File_close(i64 %0) {
bb43:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %load = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb44, label %bb45

bb44:                                             ; preds = %bb43
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 0
  %load1 = load i64, ptr %gep, align 4
  %auto_cast_ptr = inttoptr i64 %load1 to ptr
  %call = call i32 @fclose(ptr %auto_cast_ptr)
  ret void

bb45:                                             ; preds = %bb43
  call void @panic(ptr @global_str.17)
  unreachable
}

define void @File_drop(i64 %0) {
bb46:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
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
  %arr = alloca i64, align 8
  store i64 0, ptr %arr, align 4
  %arr_alloc = call ptr @malloc(i64 96)
  store i64 1, ptr %arr_alloc, align 4
  %size_field = getelementptr i64, ptr %arr_alloc, i64 1
  store i64 10, ptr %size_field, align 4
  %data_ptr = getelementptr i64, ptr %arr_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 1, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 2, ptr %gep1, align 4
  %gep2 = getelementptr i64, ptr %data_ptr, i64 2
  store i64 3, ptr %gep2, align 4
  %gep3 = getelementptr i64, ptr %data_ptr, i64 3
  store i64 4, ptr %gep3, align 4
  %gep4 = getelementptr i64, ptr %data_ptr, i64 4
  store i64 5, ptr %gep4, align 4
  %gep5 = getelementptr i64, ptr %data_ptr, i64 5
  store i64 6, ptr %gep5, align 4
  %gep6 = getelementptr i64, ptr %data_ptr, i64 6
  store i64 7, ptr %gep6, align 4
  %gep7 = getelementptr i64, ptr %data_ptr, i64 7
  store i64 8, ptr %gep7, align 4
  %gep8 = getelementptr i64, ptr %data_ptr, i64 8
  store i64 9, ptr %gep8, align 4
  %gep9 = getelementptr i64, ptr %data_ptr, i64 9
  store i64 10, ptr %gep9, align 4
  %store_cast_int = ptrtoint ptr %data_ptr to i64
  store i64 %store_cast_int, ptr %arr, align 4
  %point3 = alloca i64, align 8
  store i64 0, ptr %point3, align 4
  %struct_alloc = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 24, ptr %meta_field, align 4
  %data_ptr10 = getelementptr i64, ptr %struct_alloc, i64 2
  %gep11 = getelementptr i64, ptr %data_ptr10, i64 0
  store i64 1, ptr %gep11, align 4
  %gep12 = getelementptr i64, ptr %data_ptr10, i64 1
  store i64 2, ptr %gep12, align 4
  %gep13 = getelementptr i64, ptr %data_ptr10, i64 2
  store i64 3, ptr %gep13, align 4
  %store_cast_int14 = ptrtoint ptr %data_ptr10 to i64
  store i64 %store_cast_int14, ptr %point3, align 4
  %load = load i64, ptr %arr, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb50, label %bb51

bb50:                                             ; preds = %bb49
  %inttoptr = inttoptr i64 %load to ptr
  %is_not_null = icmp ne ptr %inttoptr, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb51:                                             ; preds = %arc.retain.cont, %bb49
  %cmpne15 = icmp ne i64 %load, 0
  %zext16 = zext i1 %cmpne15 to i64
  %trunc17 = trunc i64 %zext16 to i1
  br i1 %trunc17, label %bb52, label %bb53

bb52:                                             ; preds = %bb51
  %inttoptr18 = inttoptr i64 %load to ptr
  %gep19 = getelementptr i64, ptr %inttoptr18, i64 -1
  %load20 = load i64, ptr %gep19, align 4
  %for_in_counter = alloca i64, align 8
  store i64 0, ptr %for_in_counter, align 4
  br label %bb54

bb53:                                             ; preds = %bb51
  call void @panic(ptr @global_str.18)
  unreachable

bb54:                                             ; preds = %bb55, %bb52
  %load21 = load i64, ptr %for_in_counter, align 4
  %cmplt = icmp slt i64 %load21, %load20
  %zext22 = zext i1 %cmplt to i64
  %trunc23 = trunc i64 %zext22 to i1
  br i1 %trunc23, label %bb55, label %bb56

bb55:                                             ; preds = %bb54
  %inttoptr24 = inttoptr i64 %load to ptr
  %gep25 = getelementptr i64, ptr %inttoptr24, i64 %load21
  %load26 = load i64, ptr %gep25, align 4
  %item = alloca i64, align 8
  store i64 %load26, ptr %item, align 4
  %load27 = load i64, ptr %item, align 4
  %call = call i32 (ptr, ...) @printf(ptr @global_str.19, i64 %load27)
  %load28 = load i64, ptr %for_in_counter, align 4
  %add = add i64 %load28, 1
  store i64 %add, ptr %for_in_counter, align 4
  br label %bb54

bb56:                                             ; preds = %bb54
  %load29 = load i64, ptr %point3, align 4
  %call30 = call i64 @Point3_i64_add3(i64 %load29)
  %call31 = call i32 (ptr, ...) @printf(ptr @global_str.20, i64 %call30)
  %load32 = load i64, ptr %arr, align 4
  %cmpne33 = icmp ne i64 %load32, 0
  %zext34 = zext i1 %cmpne33 to i64
  %trunc35 = trunc i64 %zext34 to i1
  br i1 %trunc35, label %bb57, label %bb58

bb57:                                             ; preds = %bb56
  %inttoptr36 = inttoptr i64 %load32 to ptr
  %is_not_null37 = icmp ne ptr %inttoptr36, null
  br i1 %is_not_null37, label %arc.release.do, label %arc.release.cont

bb58:                                             ; preds = %arc.release.cont, %bb56
  ret i64 0

arc.retain.do:                                    ; preds = %bb50
  %ref_ptr = getelementptr i64, ptr %inttoptr, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb50
  br label %bb51

arc.release.do:                                   ; preds = %bb57
  %ref_ptr38 = getelementptr i64, ptr %inttoptr36, i64 -2
  %current_count39 = load i64, ptr %ref_ptr38, align 4
  %new_count40 = sub i64 %current_count39, 1
  store i64 %new_count40, ptr %ref_ptr38, align 4
  %is_zero = icmp eq i64 %new_count40, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb57
  br label %bb58

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr38)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define i64 @Vec_T_init() {
bb59:
  %struct_alloc = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 24, ptr %meta_field, align 4
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %gep = getelementptr i64, ptr %data_ptr, i64 0
  store i64 0, ptr %gep, align 4
  %gep1 = getelementptr i64, ptr %data_ptr, i64 1
  store i64 0, ptr %gep1, align 4
  %gep2 = getelementptr i64, ptr %data_ptr, i64 2
  store i64 0, ptr %gep2, align 4
  %ret_cast_int = ptrtoint ptr %data_ptr to i64
  ret i64 %ret_cast_int
}

define void @Vec_T_push(i64 %0, ptr %1) {
bb60:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %item = alloca ptr, align 8
  store ptr %1, ptr %item, align 8
  %is_not_null = icmp ne ptr %1, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb61:                                             ; preds = %arc.retain.cont
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 1
  %load1 = load i64, ptr %gep, align 4
  %load2 = load i64, ptr %self, align 4
  %cmpne3 = icmp ne i64 %load2, 0
  %zext4 = zext i1 %cmpne3 to i64
  %trunc5 = trunc i64 %zext4 to i1
  br i1 %trunc5, label %bb63, label %bb64

bb62:                                             ; preds = %arc.retain.cont
  call void @panic(ptr @global_str.21)
  unreachable

bb63:                                             ; preds = %bb61
  %inttoptr6 = inttoptr i64 %load2 to ptr
  %gep7 = getelementptr i64, ptr %inttoptr6, i64 2
  %load8 = load i64, ptr %gep7, align 4
  %cmpeq = icmp eq i64 %load1, %load8
  %zext9 = zext i1 %cmpeq to i64
  %trunc10 = trunc i64 %zext9 to i1
  br i1 %trunc10, label %bb65, label %bb66

bb64:                                             ; preds = %bb61
  call void @panic(ptr @global_str.22)
  unreachable

bb65:                                             ; preds = %bb63
  %new_cap = alloca i64, align 8
  store i64 0, ptr %new_cap, align 4
  store i64 0, ptr %new_cap, align 4
  %load11 = load i64, ptr %self, align 4
  %cmpne12 = icmp ne i64 %load11, 0
  %zext13 = zext i1 %cmpne12 to i64
  %trunc14 = trunc i64 %zext13 to i1
  br i1 %trunc14, label %bb67, label %bb68

bb66:                                             ; preds = %bb85, %bb63
  %load15 = load i64, ptr %self, align 4
  %cmpne16 = icmp ne i64 %load15, 0
  %zext17 = zext i1 %cmpne16 to i64
  %trunc18 = trunc i64 %zext17 to i1
  br i1 %trunc18, label %bb87, label %bb88

bb67:                                             ; preds = %bb65
  %inttoptr19 = inttoptr i64 %load11 to ptr
  %gep20 = getelementptr i64, ptr %inttoptr19, i64 2
  %load21 = load i64, ptr %gep20, align 4
  %cmpeq22 = icmp eq i64 %load21, 0
  %zext23 = zext i1 %cmpeq22 to i64
  %trunc24 = trunc i64 %zext23 to i1
  br i1 %trunc24, label %bb69, label %bb71

bb68:                                             ; preds = %bb65
  call void @panic(ptr @global_str.23)
  unreachable

bb69:                                             ; preds = %bb67
  store i64 4, ptr %new_cap, align 4
  br label %bb70

bb70:                                             ; preds = %bb72, %bb69
  %size_in_bytes = alloca i64, align 8
  store i64 0, ptr %size_in_bytes, align 4
  %load25 = load i64, ptr %new_cap, align 4
  %mul = mul i64 %load25, 8
  store i64 %mul, ptr %size_in_bytes, align 4
  %load26 = load i64, ptr %self, align 4
  %cmpne27 = icmp ne i64 %load26, 0
  %zext28 = zext i1 %cmpne27 to i64
  %trunc29 = trunc i64 %zext28 to i1
  br i1 %trunc29, label %bb74, label %bb75

bb71:                                             ; preds = %bb67
  %load30 = load i64, ptr %self, align 4
  %cmpne31 = icmp ne i64 %load30, 0
  %zext32 = zext i1 %cmpne31 to i64
  %trunc33 = trunc i64 %zext32 to i1
  br i1 %trunc33, label %bb72, label %bb73

bb72:                                             ; preds = %bb71
  %inttoptr34 = inttoptr i64 %load30 to ptr
  %gep35 = getelementptr i64, ptr %inttoptr34, i64 2
  %load36 = load i64, ptr %gep35, align 4
  %mul37 = mul i64 %load36, 2
  store i64 %mul37, ptr %new_cap, align 4
  br label %bb70

bb73:                                             ; preds = %bb71
  call void @panic(ptr @global_str.24)
  unreachable

bb74:                                             ; preds = %bb70
  %inttoptr38 = inttoptr i64 %load26 to ptr
  %gep39 = getelementptr i64, ptr %inttoptr38, i64 2
  %load40 = load i64, ptr %gep39, align 4
  %cmpeq41 = icmp eq i64 %load40, 0
  %zext42 = zext i1 %cmpeq41 to i64
  %trunc43 = trunc i64 %zext42 to i1
  br i1 %trunc43, label %bb76, label %bb78

bb75:                                             ; preds = %bb70
  call void @panic(ptr @global_str.25)
  unreachable

bb76:                                             ; preds = %bb74
  %load44 = load i64, ptr %self, align 4
  %cmpne45 = icmp ne i64 %load44, 0
  %zext46 = zext i1 %cmpne45 to i64
  %trunc47 = trunc i64 %zext46 to i1
  br i1 %trunc47, label %bb79, label %bb80

bb77:                                             ; preds = %bb83, %bb79
  %load48 = load i64, ptr %self, align 4
  %cmpne49 = icmp ne i64 %load48, 0
  %zext50 = zext i1 %cmpne49 to i64
  %trunc51 = trunc i64 %zext50 to i1
  br i1 %trunc51, label %bb85, label %bb86

bb78:                                             ; preds = %bb74
  %load52 = load i64, ptr %self, align 4
  %cmpne53 = icmp ne i64 %load52, 0
  %zext54 = zext i1 %cmpne53 to i64
  %trunc55 = trunc i64 %zext54 to i1
  br i1 %trunc55, label %bb81, label %bb82

bb79:                                             ; preds = %bb76
  %load56 = load i64, ptr %size_in_bytes, align 4
  %call = call ptr @malloc(i64 %load56)
  %inttoptr57 = inttoptr i64 %load44 to ptr
  %gep58 = getelementptr i64, ptr %inttoptr57, i64 0
  %store_cast_int = ptrtoint ptr %call to i64
  store i64 %store_cast_int, ptr %gep58, align 4
  br label %bb77

bb80:                                             ; preds = %bb76
  call void @panic(ptr @global_str.26)
  unreachable

bb81:                                             ; preds = %bb78
  %load59 = load i64, ptr %self, align 4
  %cmpne60 = icmp ne i64 %load59, 0
  %zext61 = zext i1 %cmpne60 to i64
  %trunc62 = trunc i64 %zext61 to i1
  br i1 %trunc62, label %bb83, label %bb84

bb82:                                             ; preds = %bb78
  call void @panic(ptr @global_str.27)
  unreachable

bb83:                                             ; preds = %bb81
  %inttoptr63 = inttoptr i64 %load59 to ptr
  %gep64 = getelementptr i64, ptr %inttoptr63, i64 0
  %load65 = load i64, ptr %gep64, align 4
  %inttoptr66 = inttoptr i64 %load65 to ptr
  %load67 = load i64, ptr %size_in_bytes, align 4
  %call68 = call ptr @realloc(ptr %inttoptr66, i64 %load67)
  %inttoptr69 = inttoptr i64 %load52 to ptr
  %gep70 = getelementptr i64, ptr %inttoptr69, i64 0
  %store_cast_int71 = ptrtoint ptr %call68 to i64
  store i64 %store_cast_int71, ptr %gep70, align 4
  br label %bb77

bb84:                                             ; preds = %bb81
  call void @panic(ptr @global_str.28)
  unreachable

bb85:                                             ; preds = %bb77
  %load72 = load i64, ptr %new_cap, align 4
  %inttoptr73 = inttoptr i64 %load48 to ptr
  %gep74 = getelementptr i64, ptr %inttoptr73, i64 2
  store i64 %load72, ptr %gep74, align 4
  br label %bb66

bb86:                                             ; preds = %bb77
  call void @panic(ptr @global_str.29)
  unreachable

bb87:                                             ; preds = %bb66
  %inttoptr75 = inttoptr i64 %load15 to ptr
  %gep76 = getelementptr i64, ptr %inttoptr75, i64 0
  %load77 = load i64, ptr %gep76, align 4
  %cmpne78 = icmp ne i64 %load77, 0
  %zext79 = zext i1 %cmpne78 to i64
  %trunc80 = trunc i64 %zext79 to i1
  br i1 %trunc80, label %bb89, label %bb90

bb88:                                             ; preds = %bb66
  call void @panic(ptr @global_str.30)
  unreachable

bb89:                                             ; preds = %bb87
  %load81 = load i64, ptr %self, align 4
  %cmpne82 = icmp ne i64 %load81, 0
  %zext83 = zext i1 %cmpne82 to i64
  %trunc84 = trunc i64 %zext83 to i1
  br i1 %trunc84, label %bb91, label %bb92

bb90:                                             ; preds = %bb87
  call void @panic(ptr @global_str.31)
  unreachable

bb91:                                             ; preds = %bb89
  %inttoptr85 = inttoptr i64 %load81 to ptr
  %gep86 = getelementptr i64, ptr %inttoptr85, i64 1
  %load87 = load i64, ptr %gep86, align 4
  %inttoptr88 = inttoptr i64 %load77 to ptr
  %gep89 = getelementptr i64, ptr %inttoptr88, i64 -1
  %load90 = load i64, ptr %gep89, align 4
  %cmplt = icmp slt i64 %load87, %load90
  %zext91 = zext i1 %cmplt to i64
  %trunc92 = trunc i64 %zext91 to i1
  br i1 %trunc92, label %bb93, label %bb94

bb92:                                             ; preds = %bb89
  call void @panic(ptr @global_str.32)
  unreachable

bb93:                                             ; preds = %bb91
  %cmpge = icmp sge i64 %load87, 0
  %zext93 = zext i1 %cmpge to i64
  %trunc94 = trunc i64 %zext93 to i1
  br i1 %trunc94, label %bb95, label %bb94

bb94:                                             ; preds = %bb93, %bb91
  call void @panic(ptr @global_str.33)
  unreachable

bb95:                                             ; preds = %bb93
  %load95 = load ptr, ptr %item, align 8
  %ptr2int = ptrtoint ptr %load95 to i64
  %cmpne96 = icmp ne i64 %ptr2int, 0
  %zext97 = zext i1 %cmpne96 to i64
  %trunc98 = trunc i64 %zext97 to i1
  br i1 %trunc98, label %bb96, label %bb97

bb96:                                             ; preds = %bb95
  %is_not_null99 = icmp ne ptr %load95, null
  br i1 %is_not_null99, label %arc.retain.do100, label %arc.retain.cont101

bb97:                                             ; preds = %arc.retain.cont101, %bb95
  %inttoptr105 = inttoptr i64 %load77 to ptr
  %gep106 = getelementptr i64, ptr %inttoptr105, i64 %load87
  %store_cast_int107 = ptrtoint ptr %load95 to i64
  store i64 %store_cast_int107, ptr %gep106, align 4
  %load108 = load i64, ptr %self, align 4
  %cmpne109 = icmp ne i64 %load108, 0
  %zext110 = zext i1 %cmpne109 to i64
  %trunc111 = trunc i64 %zext110 to i1
  br i1 %trunc111, label %bb98, label %bb99

bb98:                                             ; preds = %bb97
  %load112 = load i64, ptr %self, align 4
  %cmpne113 = icmp ne i64 %load112, 0
  %zext114 = zext i1 %cmpne113 to i64
  %trunc115 = trunc i64 %zext114 to i1
  br i1 %trunc115, label %bb100, label %bb101

bb99:                                             ; preds = %bb97
  call void @panic(ptr @global_str.34)
  unreachable

bb100:                                            ; preds = %bb98
  %inttoptr116 = inttoptr i64 %load112 to ptr
  %gep117 = getelementptr i64, ptr %inttoptr116, i64 1
  %load118 = load i64, ptr %gep117, align 4
  %add = add i64 %load118, 1
  %inttoptr119 = inttoptr i64 %load108 to ptr
  %gep120 = getelementptr i64, ptr %inttoptr119, i64 1
  store i64 %add, ptr %gep120, align 4
  %load121 = load ptr, ptr %item, align 8
  %ptr2int122 = ptrtoint ptr %load121 to i64
  %cmpne123 = icmp ne i64 %ptr2int122, 0
  %zext124 = zext i1 %cmpne123 to i64
  %trunc125 = trunc i64 %zext124 to i1
  br i1 %trunc125, label %bb102, label %bb103

bb101:                                            ; preds = %bb98
  call void @panic(ptr @global_str.35)
  unreachable

bb102:                                            ; preds = %bb100
  %is_not_null126 = icmp ne ptr %load121, null
  br i1 %is_not_null126, label %arc.release.do, label %arc.release.cont

bb103:                                            ; preds = %arc.release.cont, %bb100
  ret void

arc.retain.do:                                    ; preds = %bb60
  %ref_ptr = getelementptr i64, ptr %1, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb60
  %load = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb61, label %bb62

arc.retain.do100:                                 ; preds = %bb96
  %ref_ptr102 = getelementptr i64, ptr %load95, i64 -2
  %current_count103 = load i64, ptr %ref_ptr102, align 4
  %new_count104 = add i64 %current_count103, 1
  store i64 %new_count104, ptr %ref_ptr102, align 4
  br label %arc.retain.cont101

arc.retain.cont101:                               ; preds = %arc.retain.do100, %bb96
  br label %bb97

arc.release.do:                                   ; preds = %bb102
  %ref_ptr127 = getelementptr i64, ptr %load121, i64 -2
  %current_count128 = load i64, ptr %ref_ptr127, align 4
  %new_count129 = sub i64 %current_count128, 1
  store i64 %new_count129, ptr %ref_ptr127, align 4
  %is_zero = icmp eq i64 %new_count129, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb102
  br label %bb103

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr127)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define ptr @Vec_T_get(i64 %0, i64 %1) {
bb104:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %index = alloca i64, align 8
  store i64 %1, ptr %index, align 4
  %load = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb105, label %bb106

bb105:                                            ; preds = %bb104
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpne2 = icmp ne i64 %load1, 0
  %zext3 = zext i1 %cmpne2 to i64
  %trunc4 = trunc i64 %zext3 to i1
  br i1 %trunc4, label %bb107, label %bb108

bb106:                                            ; preds = %bb104
  call void @panic(ptr @global_str.36)
  unreachable

bb107:                                            ; preds = %bb105
  %load5 = load i64, ptr %index, align 4
  %inttoptr6 = inttoptr i64 %load1 to ptr
  %gep7 = getelementptr i64, ptr %inttoptr6, i64 -1
  %load8 = load i64, ptr %gep7, align 4
  %cmplt = icmp slt i64 %load5, %load8
  %zext9 = zext i1 %cmplt to i64
  %trunc10 = trunc i64 %zext9 to i1
  br i1 %trunc10, label %bb109, label %bb110

bb108:                                            ; preds = %bb105
  call void @panic(ptr @global_str.37)
  unreachable

bb109:                                            ; preds = %bb107
  %cmpge = icmp sge i64 %load5, 0
  %zext11 = zext i1 %cmpge to i64
  %trunc12 = trunc i64 %zext11 to i1
  br i1 %trunc12, label %bb111, label %bb110

bb110:                                            ; preds = %bb109, %bb107
  call void @panic(ptr @global_str.38)
  unreachable

bb111:                                            ; preds = %bb109
  %inttoptr13 = inttoptr i64 %load1 to ptr
  %gep14 = getelementptr i64, ptr %inttoptr13, i64 %load5
  %load15 = load i64, ptr %gep14, align 4
  %ret_cast_ptr = inttoptr i64 %load15 to ptr
  ret ptr %ret_cast_ptr
}

define void @Vec_T_drop(i64 %0) {
bb112:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %load = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb113, label %bb114

bb113:                                            ; preds = %bb112
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 2
  %load1 = load i64, ptr %gep, align 4
  %cmpgt = icmp sgt i64 %load1, 0
  %zext2 = zext i1 %cmpgt to i64
  %trunc3 = trunc i64 %zext2 to i1
  br i1 %trunc3, label %bb115, label %bb116

bb114:                                            ; preds = %bb112
  call void @panic(ptr @global_str.39)
  unreachable

bb115:                                            ; preds = %bb113
  %load4 = load i64, ptr %self, align 4
  %cmpne5 = icmp ne i64 %load4, 0
  %zext6 = zext i1 %cmpne5 to i64
  %trunc7 = trunc i64 %zext6 to i1
  br i1 %trunc7, label %bb117, label %bb118

bb116:                                            ; preds = %bb117, %bb113
  ret void

bb117:                                            ; preds = %bb115
  %inttoptr8 = inttoptr i64 %load4 to ptr
  %gep9 = getelementptr i64, ptr %inttoptr8, i64 0
  %load10 = load i64, ptr %gep9, align 4
  %inttoptr11 = inttoptr i64 %load10 to ptr
  call void @free(ptr %inttoptr11)
  %load12 = load i64, ptr %self, align 4
  %inttoptr13 = inttoptr i64 %load12 to ptr
  %gep14 = getelementptr i64, ptr %inttoptr13, i64 0
  store i64 0, ptr %gep14, align 4
  br label %bb116

bb118:                                            ; preds = %bb115
  call void @panic(ptr @global_str.40)
  unreachable
}

define i64 @Result_T_E_Ok(i64 %0) {
bb119:
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
bb120:
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

define i1 @Result_T_E_is_ok(i64 %0) {
bb121:
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
  br i1 %trunc, label %bb123, label %bb124

bb122:                                            ; preds = %bb126, %bb125, %bb123
  %load2 = load i1, ptr %match_res, align 1
  %zext3 = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %zext3 to i1
  ret i1 %ret_trunc

bb123:                                            ; preds = %bb121
  %inttoptr4 = inttoptr i64 %load to ptr
  %gep5 = getelementptr i64, ptr %inttoptr4, i64 1
  %load6 = load i64, ptr %gep5, align 4
  %v = alloca i64, align 8
  store i64 %load6, ptr %v, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb122

bb124:                                            ; preds = %bb121
  %cmpeq7 = icmp eq i64 %load1, 1
  %zext8 = zext i1 %cmpeq7 to i64
  %trunc9 = trunc i64 %zext8 to i1
  br i1 %trunc9, label %bb125, label %bb126

bb125:                                            ; preds = %bb124
  %inttoptr10 = inttoptr i64 %load to ptr
  %gep11 = getelementptr i64, ptr %inttoptr10, i64 1
  %load12 = load i64, ptr %gep11, align 4
  %e = alloca i64, align 8
  store i64 %load12, ptr %e, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb122

bb126:                                            ; preds = %bb124
  br label %bb122
}

define i1 @Result_T_E_is_err(i64 %0) {
bb127:
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
  br i1 %trunc, label %bb129, label %bb130

bb128:                                            ; preds = %bb132, %bb131, %bb129
  %load2 = load i1, ptr %match_res, align 1
  %zext3 = zext i1 %load2 to i64
  %ret_trunc = trunc i64 %zext3 to i1
  ret i1 %ret_trunc

bb129:                                            ; preds = %bb127
  %inttoptr4 = inttoptr i64 %load to ptr
  %gep5 = getelementptr i64, ptr %inttoptr4, i64 1
  %load6 = load i64, ptr %gep5, align 4
  %v = alloca i64, align 8
  store i64 %load6, ptr %v, align 4
  store i1 false, ptr %match_res, align 1
  br label %bb128

bb130:                                            ; preds = %bb127
  %cmpeq7 = icmp eq i64 %load1, 1
  %zext8 = zext i1 %cmpeq7 to i64
  %trunc9 = trunc i64 %zext8 to i1
  br i1 %trunc9, label %bb131, label %bb132

bb131:                                            ; preds = %bb130
  %inttoptr10 = inttoptr i64 %load to ptr
  %gep11 = getelementptr i64, ptr %inttoptr10, i64 1
  %load12 = load i64, ptr %gep11, align 4
  %e = alloca i64, align 8
  store i64 %load12, ptr %e, align 4
  store i1 true, ptr %match_res, align 1
  br label %bb128

bb132:                                            ; preds = %bb130
  br label %bb128
}

define ptr @Result_T_E_unwrap(i64 %0) {
bb133:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %load = load i64, ptr %self, align 4
  %match_res = alloca i64, align 8
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 0
  %load1 = load i64, ptr %gep, align 4
  %cmpeq = icmp eq i64 %load1, 0
  %zext = zext i1 %cmpeq to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb135, label %bb136

bb134:                                            ; preds = %bb140, %bb138, %bb135
  %load2 = load i64, ptr %match_res, align 4
  %ret_cast_ptr = inttoptr i64 %load2 to ptr
  ret ptr %ret_cast_ptr

bb135:                                            ; preds = %bb133
  %inttoptr3 = inttoptr i64 %load to ptr
  %gep4 = getelementptr i64, ptr %inttoptr3, i64 1
  %load5 = load i64, ptr %gep4, align 4
  %v = alloca i64, align 8
  store i64 %load5, ptr %v, align 4
  %load6 = load i64, ptr %v, align 4
  store i64 %load6, ptr %match_res, align 4
  br label %bb134

bb136:                                            ; preds = %bb133
  %cmpeq7 = icmp eq i64 %load1, 1
  %zext8 = zext i1 %cmpeq7 to i64
  %trunc9 = trunc i64 %zext8 to i1
  br i1 %trunc9, label %bb137, label %bb138

bb137:                                            ; preds = %bb136
  %inttoptr10 = inttoptr i64 %load to ptr
  %gep11 = getelementptr i64, ptr %inttoptr10, i64 1
  %load12 = load i64, ptr %gep11, align 4
  %e = alloca i64, align 8
  store i64 %load12, ptr %e, align 4
  %call = call i32 (ptr, ...) @printf(ptr @global_str.41)
  call void @exit(i32 1)
  %dummy = alloca ptr, align 8
  store ptr null, ptr %dummy, align 8
  %load13 = load ptr, ptr %dummy, align 8
  %ptr2int = ptrtoint ptr %load13 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext14 = zext i1 %cmpne to i64
  %trunc15 = trunc i64 %zext14 to i1
  br i1 %trunc15, label %bb139, label %bb140

bb138:                                            ; preds = %bb136
  br label %bb134

bb139:                                            ; preds = %bb137
  %is_not_null = icmp ne ptr %load13, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb140:                                            ; preds = %arc.retain.cont, %bb137
  %store_cast_int = ptrtoint ptr %load13 to i64
  store i64 %store_cast_int, ptr %match_res, align 4
  br label %bb134

arc.retain.do:                                    ; preds = %bb139
  %ref_ptr = getelementptr i64, ptr %load13, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb139
  br label %bb140
}

define ptr @Point3_T_add3(i64 %0) {
bb141:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %load = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb142, label %bb143

bb142:                                            ; preds = %bb141
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 0
  %load1 = load i64, ptr %gep, align 4
  %load2 = load i64, ptr %self, align 4
  %cmpne3 = icmp ne i64 %load2, 0
  %zext4 = zext i1 %cmpne3 to i64
  %trunc5 = trunc i64 %zext4 to i1
  br i1 %trunc5, label %bb144, label %bb145

bb143:                                            ; preds = %bb141
  call void @panic(ptr @global_str.42)
  unreachable

bb144:                                            ; preds = %bb142
  %inttoptr6 = inttoptr i64 %load2 to ptr
  %gep7 = getelementptr i64, ptr %inttoptr6, i64 1
  %load8 = load i64, ptr %gep7, align 4
  %add = add i64 %load1, %load8
  %load9 = load i64, ptr %self, align 4
  %cmpne10 = icmp ne i64 %load9, 0
  %zext11 = zext i1 %cmpne10 to i64
  %trunc12 = trunc i64 %zext11 to i1
  br i1 %trunc12, label %bb146, label %bb147

bb145:                                            ; preds = %bb142
  call void @panic(ptr @global_str.43)
  unreachable

bb146:                                            ; preds = %bb144
  %inttoptr13 = inttoptr i64 %load9 to ptr
  %gep14 = getelementptr i64, ptr %inttoptr13, i64 2
  %load15 = load i64, ptr %gep14, align 4
  %add16 = add i64 %add, %load15
  %ret_cast_ptr = inttoptr i64 %add16 to ptr
  ret ptr %ret_cast_ptr

bb147:                                            ; preds = %bb144
  call void @panic(ptr @global_str.44)
  unreachable
}

define i64 @Point3_i64_add3(i64 %0) {
bb148:
  %self = alloca i64, align 8
  store i64 %0, ptr %self, align 4
  %load = load i64, ptr %self, align 4
  %cmpne = icmp ne i64 %load, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb149, label %bb150

bb149:                                            ; preds = %bb148
  %inttoptr = inttoptr i64 %load to ptr
  %gep = getelementptr i64, ptr %inttoptr, i64 0
  %load1 = load i64, ptr %gep, align 4
  %load2 = load i64, ptr %self, align 4
  %cmpne3 = icmp ne i64 %load2, 0
  %zext4 = zext i1 %cmpne3 to i64
  %trunc5 = trunc i64 %zext4 to i1
  br i1 %trunc5, label %bb151, label %bb152

bb150:                                            ; preds = %bb148
  call void @panic(ptr @global_str.45)
  unreachable

bb151:                                            ; preds = %bb149
  %inttoptr6 = inttoptr i64 %load2 to ptr
  %gep7 = getelementptr i64, ptr %inttoptr6, i64 1
  %load8 = load i64, ptr %gep7, align 4
  %add = add i64 %load1, %load8
  %load9 = load i64, ptr %self, align 4
  %cmpne10 = icmp ne i64 %load9, 0
  %zext11 = zext i1 %cmpne10 to i64
  %trunc12 = trunc i64 %zext11 to i1
  br i1 %trunc12, label %bb153, label %bb154

bb152:                                            ; preds = %bb149
  call void @panic(ptr @global_str.46)
  unreachable

bb153:                                            ; preds = %bb151
  %inttoptr13 = inttoptr i64 %load9 to ptr
  %gep14 = getelementptr i64, ptr %inttoptr13, i64 2
  %load15 = load i64, ptr %gep14, align 4
  %add16 = add i64 %add, %load15
  ret i64 %add16

bb154:                                            ; preds = %bb151
  call void @panic(ptr @global_str.47)
  unreachable
}

define void @__global_init() {
bb0:
  ret void
}
