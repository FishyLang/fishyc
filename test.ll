; ModuleID = 'fishy_module'
source_filename = "fishy_module"

%Point3_f16 = type { half, half, half }

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.1 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.2 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.3 = private unnamed_addr constant [43 x i8] c"Null pointer dereference no loop 'for in'!\00", align 1
@global_str.4 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.5 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.6 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.7 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.8 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.9 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.10 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.11 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.12 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.13 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.14 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.15 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.16 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.17 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.18 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.19 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.20 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.21 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.22 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.23 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.24 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.25 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.26 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.27 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.28 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.29 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.30 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.31 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.32 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.33 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.34 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.35 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.36 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.37 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.38 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.39 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.40 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.41 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.42 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.43 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.44 = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.45 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.46 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1

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
  %struct_alloc = call ptr @malloc(i64 22)
  store i64 1, ptr %struct_alloc, align 4
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 6, ptr %meta_field, align 4
  %data_ptr10 = getelementptr i64, ptr %struct_alloc, i64 2
  %gep11 = getelementptr %Point3_f16, ptr %data_ptr10, i64 0, i32 0
  store half 0xH4066, ptr %gep11, align 2
  %gep12 = getelementptr %Point3_f16, ptr %data_ptr10, i64 0, i32 1
  store half 0xH4466, ptr %gep12, align 2
  %gep13 = getelementptr %Point3_f16, ptr %data_ptr10, i64 0, i32 2
  store half 0xH469A, ptr %gep13, align 2
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
  %cmpne34 = icmp ne i64 %load33, 0
  %zext35 = zext i1 %cmpne34 to i64
  %trunc36 = trunc i64 %zext35 to i1
  br i1 %trunc36, label %bb57, label %bb58

bb57:                                             ; preds = %bb56
  %inttoptr37 = inttoptr i64 %load33 to ptr
  %is_not_null38 = icmp ne ptr %inttoptr37, null
  br i1 %is_not_null38, label %arc.retain.do39, label %arc.retain.cont40

bb58:                                             ; preds = %arc.retain.cont40, %bb56
  %auto_cast_ptr = inttoptr i64 %load33 to ptr
  %call44 = call half @Point3_f16_add3(ptr %auto_cast_ptr)
  %vararg_fpext = fpext half %call44 to double
  %call45 = call i32 (ptr, ...) @printf(ptr @global_str.8, double %vararg_fpext)
  %load46 = load i64, ptr %point3, align 4
  %cmpne47 = icmp ne i64 %load46, 0
  %zext48 = zext i1 %cmpne47 to i64
  %trunc49 = trunc i64 %zext48 to i1
  br i1 %trunc49, label %bb59, label %bb60

bb59:                                             ; preds = %bb58
  %inttoptr50 = inttoptr i64 %load46 to ptr
  %is_not_null51 = icmp ne ptr %inttoptr50, null
  br i1 %is_not_null51, label %arc.retain.do52, label %arc.retain.cont53

bb60:                                             ; preds = %arc.retain.cont53, %bb58
  %auto_cast_ptr57 = inttoptr i64 %load46 to ptr
  %call58 = call half @Point3_f16_getX(ptr %auto_cast_ptr57)
  %vararg_fpext59 = fpext half %call58 to double
  %call60 = call i32 (ptr, ...) @printf(ptr @global_str.9, double %vararg_fpext59)
  %load61 = load i64, ptr %point3, align 4
  %cmpne62 = icmp ne i64 %load61, 0
  %zext63 = zext i1 %cmpne62 to i64
  %trunc64 = trunc i64 %zext63 to i1
  br i1 %trunc64, label %bb61, label %bb62

bb61:                                             ; preds = %bb60
  %inttoptr65 = inttoptr i64 %load61 to ptr
  %is_not_null66 = icmp ne ptr %inttoptr65, null
  br i1 %is_not_null66, label %arc.retain.do67, label %arc.retain.cont68

bb62:                                             ; preds = %arc.retain.cont68, %bb60
  %auto_cast_ptr72 = inttoptr i64 %load61 to ptr
  %call73 = call half @Point3_f16_getY(ptr %auto_cast_ptr72)
  %vararg_fpext74 = fpext half %call73 to double
  %call75 = call i32 (ptr, ...) @printf(ptr @global_str.10, double %vararg_fpext74)
  %load76 = load i64, ptr %point3, align 4
  %cmpne77 = icmp ne i64 %load76, 0
  %zext78 = zext i1 %cmpne77 to i64
  %trunc79 = trunc i64 %zext78 to i1
  br i1 %trunc79, label %bb63, label %bb64

bb63:                                             ; preds = %bb62
  %inttoptr80 = inttoptr i64 %load76 to ptr
  %is_not_null81 = icmp ne ptr %inttoptr80, null
  br i1 %is_not_null81, label %arc.retain.do82, label %arc.retain.cont83

bb64:                                             ; preds = %arc.retain.cont83, %bb62
  %auto_cast_ptr87 = inttoptr i64 %load76 to ptr
  %call88 = call half @Point3_f16_getZ(ptr %auto_cast_ptr87)
  %vararg_fpext89 = fpext half %call88 to double
  %call90 = call i32 (ptr, ...) @printf(ptr @global_str.11, double %vararg_fpext89)
  %call91 = call i32 (ptr, ...) @printf(ptr @global_str.12, i64 1)
  %call92 = call i32 (ptr, ...) @printf(ptr @global_str.13, i64 1)
  %call93 = call i32 (ptr, ...) @printf(ptr @global_str.14, i64 1)
  %call94 = call i32 (ptr, ...) @printf(ptr @global_str.15, i64 1)
  %call95 = call i32 (ptr, ...) @printf(ptr @global_str.16, i64 1)
  %call96 = call i32 (ptr, ...) @printf(ptr @global_str.17, i64 1)
  %call97 = call i32 (ptr, ...) @printf(ptr @global_str.18, i64 1)
  %call98 = call i32 (ptr, ...) @printf(ptr @global_str.19, i64 1)
  %call99 = call i32 (ptr, ...) @printf(ptr @global_str.20, double 1.100000e+00)
  %call100 = call i32 (ptr, ...) @printf(ptr @global_str.21, double 1.100000e+00)
  %call101 = call i32 (ptr, ...) @printf(ptr @global_str.22, double 1.100000e+00)
  %load102 = load i64, ptr %arr, align 4
  %cmpne103 = icmp ne i64 %load102, 0
  %zext104 = zext i1 %cmpne103 to i64
  %trunc105 = trunc i64 %zext104 to i1
  br i1 %trunc105, label %bb65, label %bb66

bb65:                                             ; preds = %bb64
  %inttoptr106 = inttoptr i64 %load102 to ptr
  %is_not_null107 = icmp ne ptr %inttoptr106, null
  br i1 %is_not_null107, label %arc.release.do, label %arc.release.cont

bb66:                                             ; preds = %arc.release.cont, %bb64
  %load111 = load i64, ptr %point3, align 4
  %cmpne112 = icmp ne i64 %load111, 0
  %zext113 = zext i1 %cmpne112 to i64
  %trunc114 = trunc i64 %zext113 to i1
  br i1 %trunc114, label %bb67, label %bb68

bb67:                                             ; preds = %bb66
  %inttoptr115 = inttoptr i64 %load111 to ptr
  %is_not_null116 = icmp ne ptr %inttoptr115, null
  br i1 %is_not_null116, label %arc.release.do117, label %arc.release.cont118

bb68:                                             ; preds = %arc.release.cont118, %bb66
  ret i64 0

arc.retain.do:                                    ; preds = %bb50
  %ref_ptr = getelementptr i64, ptr %inttoptr, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb50
  br label %bb51

arc.retain.do39:                                  ; preds = %bb57
  %ref_ptr41 = getelementptr i64, ptr %inttoptr37, i64 -2
  %current_count42 = load i64, ptr %ref_ptr41, align 4
  %new_count43 = add i64 %current_count42, 1
  store i64 %new_count43, ptr %ref_ptr41, align 4
  br label %arc.retain.cont40

arc.retain.cont40:                                ; preds = %arc.retain.do39, %bb57
  br label %bb58

arc.retain.do52:                                  ; preds = %bb59
  %ref_ptr54 = getelementptr i64, ptr %inttoptr50, i64 -2
  %current_count55 = load i64, ptr %ref_ptr54, align 4
  %new_count56 = add i64 %current_count55, 1
  store i64 %new_count56, ptr %ref_ptr54, align 4
  br label %arc.retain.cont53

arc.retain.cont53:                                ; preds = %arc.retain.do52, %bb59
  br label %bb60

arc.retain.do67:                                  ; preds = %bb61
  %ref_ptr69 = getelementptr i64, ptr %inttoptr65, i64 -2
  %current_count70 = load i64, ptr %ref_ptr69, align 4
  %new_count71 = add i64 %current_count70, 1
  store i64 %new_count71, ptr %ref_ptr69, align 4
  br label %arc.retain.cont68

arc.retain.cont68:                                ; preds = %arc.retain.do67, %bb61
  br label %bb62

arc.retain.do82:                                  ; preds = %bb63
  %ref_ptr84 = getelementptr i64, ptr %inttoptr80, i64 -2
  %current_count85 = load i64, ptr %ref_ptr84, align 4
  %new_count86 = add i64 %current_count85, 1
  store i64 %new_count86, ptr %ref_ptr84, align 4
  br label %arc.retain.cont83

arc.retain.cont83:                                ; preds = %arc.retain.do82, %bb63
  br label %bb64

arc.release.do:                                   ; preds = %bb65
  %ref_ptr108 = getelementptr i64, ptr %inttoptr106, i64 -2
  %current_count109 = load i64, ptr %ref_ptr108, align 4
  %new_count110 = sub i64 %current_count109, 1
  store i64 %new_count110, ptr %ref_ptr108, align 4
  %is_zero = icmp eq i64 %new_count110, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb65
  br label %bb66

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr108)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont

arc.release.do117:                                ; preds = %bb67
  %ref_ptr119 = getelementptr i64, ptr %inttoptr115, i64 -2
  %current_count120 = load i64, ptr %ref_ptr119, align 4
  %new_count121 = sub i64 %current_count120, 1
  store i64 %new_count121, ptr %ref_ptr119, align 4
  %is_zero122 = icmp eq i64 %new_count121, 0
  br i1 %is_zero122, label %arc.free123, label %arc.end124

arc.release.cont118:                              ; preds = %arc.end124, %bb67
  br label %bb68

arc.free123:                                      ; preds = %arc.release.do117
  call void @free(ptr %ref_ptr119)
  br label %arc.end124

arc.end124:                                       ; preds = %arc.free123, %arc.release.do117
  br label %arc.release.cont118
}

define half @Point3_f16_add3(ptr %0) {
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
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %trunc6 = trunc i64 %zext5 to i1
  br i1 %trunc6, label %bb170, label %bb171

bb169:                                            ; preds = %bb167
  %message = alloca ptr, align 8
  store ptr @global_str.23, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str.24)
  %load7 = load ptr, ptr %message, align 8
  %call8 = call i32 (ptr, ...) @printf(ptr @global_str.25, ptr %load7)
  %call9 = call i32 (ptr, ...) @printf(ptr @global_str.26)
  call void @exit(i32 1)
  unreachable

bb170:                                            ; preds = %bb168
  %gep10 = getelementptr %Point3_f16, ptr %load2, i64 0, i32 1
  %load11 = load half, ptr %gep10, align 2
  %fadd = fadd half %load1, %load11
  %load12 = load ptr, ptr %self, align 8
  %ptr2int13 = ptrtoint ptr %load12 to i64
  %cmpne14 = icmp ne i64 %ptr2int13, 0
  %zext15 = zext i1 %cmpne14 to i64
  %trunc16 = trunc i64 %zext15 to i1
  br i1 %trunc16, label %bb172, label %bb173

bb171:                                            ; preds = %bb168
  %message17 = alloca ptr, align 8
  store ptr @global_str.27, ptr %message17, align 8
  %call18 = call i32 (ptr, ...) @printf(ptr @global_str.28)
  %load19 = load ptr, ptr %message17, align 8
  %call20 = call i32 (ptr, ...) @printf(ptr @global_str.29, ptr %load19)
  %call21 = call i32 (ptr, ...) @printf(ptr @global_str.30)
  call void @exit(i32 1)
  unreachable

bb172:                                            ; preds = %bb170
  %gep22 = getelementptr %Point3_f16, ptr %load12, i64 0, i32 2
  %load23 = load half, ptr %gep22, align 2
  %fadd24 = fadd half %fadd, %load23
  ret half %fadd24

bb173:                                            ; preds = %bb170
  %message25 = alloca ptr, align 8
  store ptr @global_str.31, ptr %message25, align 8
  %call26 = call i32 (ptr, ...) @printf(ptr @global_str.32)
  %load27 = load ptr, ptr %message25, align 8
  %call28 = call i32 (ptr, ...) @printf(ptr @global_str.33, ptr %load27)
  %call29 = call i32 (ptr, ...) @printf(ptr @global_str.34)
  call void @exit(i32 1)
  unreachable
}

define half @Point3_f16_getX(ptr %0) {
bb174:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb175, label %bb176

bb175:                                            ; preds = %bb174
  %gep = getelementptr %Point3_f16, ptr %load, i64 0, i32 0
  %load1 = load half, ptr %gep, align 2
  ret half %load1

bb176:                                            ; preds = %bb174
  %message = alloca ptr, align 8
  store ptr @global_str.35, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str.36)
  %load2 = load ptr, ptr %message, align 8
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.37, ptr %load2)
  %call4 = call i32 (ptr, ...) @printf(ptr @global_str.38)
  call void @exit(i32 1)
  unreachable
}

define half @Point3_f16_getY(ptr %0) {
bb177:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb178, label %bb179

bb178:                                            ; preds = %bb177
  %gep = getelementptr %Point3_f16, ptr %load, i64 0, i32 1
  %load1 = load half, ptr %gep, align 2
  ret half %load1

bb179:                                            ; preds = %bb177
  %message = alloca ptr, align 8
  store ptr @global_str.39, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str.40)
  %load2 = load ptr, ptr %message, align 8
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.41, ptr %load2)
  %call4 = call i32 (ptr, ...) @printf(ptr @global_str.42)
  call void @exit(i32 1)
  unreachable
}

define half @Point3_f16_getZ(ptr %0) {
bb180:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %trunc = trunc i64 %zext to i1
  br i1 %trunc, label %bb181, label %bb182

bb181:                                            ; preds = %bb180
  %gep = getelementptr %Point3_f16, ptr %load, i64 0, i32 2
  %load1 = load half, ptr %gep, align 2
  ret half %load1

bb182:                                            ; preds = %bb180
  %message = alloca ptr, align 8
  store ptr @global_str.43, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str.44)
  %load2 = load ptr, ptr %message, align 8
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.45, ptr %load2)
  %call4 = call i32 (ptr, ...) @printf(ptr @global_str.46)
  call void @exit(i32 1)
  unreachable
}

define void @__global_init() {
bb0:
  ret void
}
