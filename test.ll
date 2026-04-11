; ModuleID = 'fishy_module'
source_filename = "fishy_module"

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.1 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.2 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.3 = private unnamed_addr constant [43 x i8] c"Null pointer dereference no loop 'for in'!\00", align 1
@global_str.4 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.5 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.6 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.7 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.8 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.9 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.10 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.11 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.12 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.13 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.14 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.15 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.16 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.17 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.18 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.19 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.20 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1

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
  store double 1.100000e+00, ptr %gep11, align 8
  %gep12 = getelementptr i64, ptr %data_ptr10, i64 1
  store double 2.200000e+00, ptr %gep12, align 8
  %gep13 = getelementptr i64, ptr %data_ptr10, i64 2
  store double 3.300000e+00, ptr %gep13, align 8
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
  %message = alloca ptr, align 8
  store ptr @global_str.3, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str.4)
  %load21 = load ptr, ptr %message, align 8
  %call22 = call i32 (ptr, ...) @printf(ptr @global_str.5, ptr %load21)
  %call23 = call i32 (ptr, ...) @printf(ptr @global_str.6)
  call void @exit(i32 1)
  unreachable

bb54:                                             ; preds = %bb55, %bb52
  %load24 = load i64, ptr %for_in_counter, align 4
  %cmplt = icmp slt i64 %load24, %load20
  %zext25 = zext i1 %cmplt to i64
  %trunc26 = trunc i64 %zext25 to i1
  br i1 %trunc26, label %bb55, label %bb56

bb55:                                             ; preds = %bb54
  %inttoptr27 = inttoptr i64 %load to ptr
  %gep28 = getelementptr i64, ptr %inttoptr27, i64 %load24
  %load29 = load i64, ptr %gep28, align 4
  %item = alloca i64, align 8
  store i64 %load29, ptr %item, align 4
  %load30 = load i64, ptr %item, align 4
  %call31 = call i32 (ptr, ...) @printf(ptr @global_str.7, i64 %load30)
  %load32 = load i64, ptr %for_in_counter, align 4
  %add = add i64 %load32, 1
  store i64 %add, ptr %for_in_counter, align 4
  br label %bb54

bb56:                                             ; preds = %bb54
  %load33 = load i64, ptr %point3, align 4
  %auto_cast_ptr = inttoptr i64 %load33 to ptr
  %call34 = call double @Point3_f32_add3(ptr %auto_cast_ptr)
  %call35 = call i32 (ptr, ...) @printf(ptr @global_str.8, double %call34)
  %load36 = load i64, ptr %arr, align 4
  %cmpne37 = icmp ne i64 %load36, 0
  %zext38 = zext i1 %cmpne37 to i64
  %trunc39 = trunc i64 %zext38 to i1
  br i1 %trunc39, label %bb57, label %bb58

bb57:                                             ; preds = %bb56
  %inttoptr40 = inttoptr i64 %load36 to ptr
  %is_not_null41 = icmp ne ptr %inttoptr40, null
  br i1 %is_not_null41, label %arc.release.do, label %arc.release.cont

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
  %ref_ptr42 = getelementptr i64, ptr %inttoptr40, i64 -2
  %current_count43 = load i64, ptr %ref_ptr42, align 4
  %new_count44 = sub i64 %current_count43, 1
  store i64 %new_count44, ptr %ref_ptr42, align 4
  %is_zero = icmp eq i64 %new_count44, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb57
  br label %bb58

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr42)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define double @Point3_f32_add3(ptr %0) {
bb148:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb149, label %bb150

bb149:                                            ; preds = %bb148
  %gep = getelementptr ptr, ptr %load, i64 0
  %load1 = load double, ptr %gep, align 8
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %trunc6 = trunc i64 %zext5 to i1
  br i1 %trunc6, label %bb151, label %bb152

bb150:                                            ; preds = %bb148
  %message = alloca ptr, align 8
  store ptr @global_str.9, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str.10)
  %load7 = load ptr, ptr %message, align 8
  %call8 = call i32 (ptr, ...) @printf(ptr @global_str.11, ptr %load7)
  %call9 = call i32 (ptr, ...) @printf(ptr @global_str.12)
  call void @exit(i32 1)
  unreachable

bb151:                                            ; preds = %bb149
  %gep10 = getelementptr ptr, ptr %load2, i64 1
  %load11 = load double, ptr %gep10, align 8
  %fadd = fadd double %load1, %load11
  %load12 = load ptr, ptr %self, align 8
  %ptr2int13 = ptrtoint ptr %load12 to i64
  %cmpne14 = icmp ne i64 %ptr2int13, 0
  %zext15 = zext i1 %cmpne14 to i64
  %trunc16 = trunc i64 %zext15 to i1
  br i1 %trunc16, label %bb153, label %bb154

bb152:                                            ; preds = %bb149
  %message17 = alloca ptr, align 8
  store ptr @global_str.13, ptr %message17, align 8
  %call18 = call i32 (ptr, ...) @printf(ptr @global_str.14)
  %load19 = load ptr, ptr %message17, align 8
  %call20 = call i32 (ptr, ...) @printf(ptr @global_str.15, ptr %load19)
  %call21 = call i32 (ptr, ...) @printf(ptr @global_str.16)
  call void @exit(i32 1)
  unreachable

bb153:                                            ; preds = %bb151
  %gep22 = getelementptr ptr, ptr %load12, i64 2
  %load23 = load double, ptr %gep22, align 8
  %fadd24 = fadd double %fadd, %load23
  ret double %fadd24

bb154:                                            ; preds = %bb151
  %message25 = alloca ptr, align 8
  store ptr @global_str.17, ptr %message25, align 8
  %call26 = call i32 (ptr, ...) @printf(ptr @global_str.18)
  %load27 = load ptr, ptr %message25, align 8
  %call28 = call i32 (ptr, ...) @printf(ptr @global_str.19, ptr %load27)
  %call29 = call i32 (ptr, ...) @printf(ptr @global_str.20)
  call void @exit(i32 1)
  unreachable
}

define void @__global_init() {
bb0:
  ret void
}
