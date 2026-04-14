; ModuleID = 'fishy_module'
source_filename = "fishy_module"

%Point3_f32 = type { float, float, float }
%String = type { ptr, i64, i64 }

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.1 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.2 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.3 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.4 = private unnamed_addr constant [41 x i8] c"Null pointer dereference on Method Call!\00", align 1
@global_str.5 = private unnamed_addr constant [14 x i8] c"Hello, World!\00", align 1
@global_str.6 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@global_str.7 = private unnamed_addr constant [6 x i8] c"%lld\0A\00", align 1
@global_str.8 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1

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
bb51:
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
  br i1 %cond_true, label %bb52, label %bb53

bb52:                                             ; preds = %bb51
  %int_to_ptr = inttoptr i64 %load to ptr
  %is_not_null = icmp ne ptr %int_to_ptr, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb53:                                             ; preds = %arc.retain.cont, %bb51
  %cmpne3 = icmp ne i64 %load, 0
  %zext4 = zext i1 %cmpne3 to i64
  %cond_true5 = icmp ne i64 %zext4, 0
  br i1 %cond_true5, label %bb54, label %bb55

bb54:                                             ; preds = %bb53
  %auto_cast_ptr = inttoptr i64 %load to ptr
  %call = call float @Point3_f32_add3(ptr %auto_cast_ptr)
  %vararg_fpext = fpext float %call to double
  %call6 = call i32 (ptr, ...) @printf(ptr @global_str.3, double %vararg_fpext)
  %load7 = load i64, ptr %point3, align 4
  %cmpne8 = icmp ne i64 %load7, 0
  %zext9 = zext i1 %cmpne8 to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb56, label %bb57

bb55:                                             ; preds = %bb53
  %message = alloca ptr, align 8
  store ptr @global_str.4, ptr %message, align 8
  %call11 = call i32 (ptr, ...) @printf(ptr @global_str)
  %load12 = load ptr, ptr %message, align 8
  %call13 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load12)
  %call14 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable

bb56:                                             ; preds = %bb54
  %int_to_ptr15 = inttoptr i64 %load7 to ptr
  %is_not_null16 = icmp ne ptr %int_to_ptr15, null
  br i1 %is_not_null16, label %arc.retain.do17, label %arc.retain.cont18

bb57:                                             ; preds = %arc.retain.cont18, %bb54
  %cmpne22 = icmp ne i64 %load7, 0
  %zext23 = zext i1 %cmpne22 to i64
  %cond_true24 = icmp ne i64 %zext23, 0
  br i1 %cond_true24, label %bb58, label %bb59

bb58:                                             ; preds = %bb57
  %auto_cast_ptr25 = inttoptr i64 %load7 to ptr
  %call26 = call float @Point3_f32_getX(ptr %auto_cast_ptr25)
  %vararg_fpext27 = fpext float %call26 to double
  %call28 = call i32 (ptr, ...) @printf(ptr @global_str.3, double %vararg_fpext27)
  %load29 = load i64, ptr %point3, align 4
  %cmpne30 = icmp ne i64 %load29, 0
  %zext31 = zext i1 %cmpne30 to i64
  %cond_true32 = icmp ne i64 %zext31, 0
  br i1 %cond_true32, label %bb60, label %bb61

bb59:                                             ; preds = %bb57
  %message33 = alloca ptr, align 8
  store ptr @global_str.4, ptr %message33, align 8
  %call34 = call i32 (ptr, ...) @printf(ptr @global_str)
  %load35 = load ptr, ptr %message33, align 8
  %call36 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load35)
  %call37 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable

bb60:                                             ; preds = %bb58
  %int_to_ptr38 = inttoptr i64 %load29 to ptr
  %is_not_null39 = icmp ne ptr %int_to_ptr38, null
  br i1 %is_not_null39, label %arc.retain.do40, label %arc.retain.cont41

bb61:                                             ; preds = %arc.retain.cont41, %bb58
  %cmpne45 = icmp ne i64 %load29, 0
  %zext46 = zext i1 %cmpne45 to i64
  %cond_true47 = icmp ne i64 %zext46, 0
  br i1 %cond_true47, label %bb62, label %bb63

bb62:                                             ; preds = %bb61
  %auto_cast_ptr48 = inttoptr i64 %load29 to ptr
  %call49 = call float @Point3_f32_getY(ptr %auto_cast_ptr48)
  %vararg_fpext50 = fpext float %call49 to double
  %call51 = call i32 (ptr, ...) @printf(ptr @global_str.3, double %vararg_fpext50)
  %load52 = load i64, ptr %point3, align 4
  %cmpne53 = icmp ne i64 %load52, 0
  %zext54 = zext i1 %cmpne53 to i64
  %cond_true55 = icmp ne i64 %zext54, 0
  br i1 %cond_true55, label %bb64, label %bb65

bb63:                                             ; preds = %bb61
  %message56 = alloca ptr, align 8
  store ptr @global_str.4, ptr %message56, align 8
  %call57 = call i32 (ptr, ...) @printf(ptr @global_str)
  %load58 = load ptr, ptr %message56, align 8
  %call59 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load58)
  %call60 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable

bb64:                                             ; preds = %bb62
  %int_to_ptr61 = inttoptr i64 %load52 to ptr
  %is_not_null62 = icmp ne ptr %int_to_ptr61, null
  br i1 %is_not_null62, label %arc.retain.do63, label %arc.retain.cont64

bb65:                                             ; preds = %arc.retain.cont64, %bb62
  %cmpne68 = icmp ne i64 %load52, 0
  %zext69 = zext i1 %cmpne68 to i64
  %cond_true70 = icmp ne i64 %zext69, 0
  br i1 %cond_true70, label %bb66, label %bb67

bb66:                                             ; preds = %bb65
  %auto_cast_ptr71 = inttoptr i64 %load52 to ptr
  %call72 = call float @Point3_f32_getZ(ptr %auto_cast_ptr71)
  %vararg_fpext73 = fpext float %call72 to double
  %call74 = call i32 (ptr, ...) @printf(ptr @global_str.3, double %vararg_fpext73)
  %call75 = call i32 (ptr, ...) @printf(ptr @global_str.3, double 1.100000e+00)
  %call76 = call i32 (ptr, ...) @printf(ptr @global_str.3, double 1.100000e+00)
  %call77 = call i32 (ptr, ...) @printf(ptr @global_str.3, double 1.100000e+00)
  %str = alloca i64, align 8
  store i64 0, ptr %str, align 4
  %cstr = alloca ptr, align 8
  store ptr @global_str.5, ptr %cstr, align 8
  %length = alloca i64, align 8
  store i64 0, ptr %length, align 4
  %load78 = load ptr, ptr %cstr, align 8
  %call79 = call i64 @strlen(ptr %load78)
  store i64 %call79, ptr %length, align 4
  %cap = alloca i64, align 8
  store i64 0, ptr %cap, align 4
  %load80 = load i64, ptr %length, align 4
  store i64 %load80, ptr %cap, align 4
  %ptr = alloca ptr, align 8
  store ptr null, ptr %ptr, align 8
  %load81 = load i64, ptr %cap, align 4
  %call82 = call ptr @malloc(i64 %load81)
  store ptr %call82, ptr %ptr, align 8
  %load83 = load ptr, ptr %ptr, align 8
  %load84 = load ptr, ptr %cstr, align 8
  %load85 = load i64, ptr %cap, align 4
  %call86 = call ptr @memcpy(ptr %load83, ptr %load84, i64 %load85)
  %struct_alloc87 = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc87, align 4
  %meta_field88 = getelementptr i64, ptr %struct_alloc87, i64 1
  store i64 24, ptr %meta_field88, align 4
  %data_ptr89 = getelementptr i64, ptr %struct_alloc87, i64 2
  %load90 = load ptr, ptr %ptr, align 8
  %gep91 = getelementptr %String, ptr %data_ptr89, i64 0, i32 0
  store ptr %load90, ptr %gep91, align 8
  %load92 = load i64, ptr %length, align 4
  %gep93 = getelementptr %String, ptr %data_ptr89, i64 0, i32 1
  store i64 %load92, ptr %gep93, align 4
  %load94 = load i64, ptr %cap, align 4
  %gep95 = getelementptr %String, ptr %data_ptr89, i64 0, i32 2
  store i64 %load94, ptr %gep95, align 4
  %store_cast_int96 = ptrtoint ptr %data_ptr89 to i64
  store i64 %store_cast_int96, ptr %str, align 4
  %load97 = load i64, ptr %str, align 4
  %cmpne98 = icmp ne i64 %load97, 0
  %zext99 = zext i1 %cmpne98 to i64
  %cond_true100 = icmp ne i64 %zext99, 0
  br i1 %cond_true100, label %bb68, label %bb69

bb67:                                             ; preds = %bb65
  %message101 = alloca ptr, align 8
  store ptr @global_str.4, ptr %message101, align 8
  %call102 = call i32 (ptr, ...) @printf(ptr @global_str)
  %load103 = load ptr, ptr %message101, align 8
  %call104 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load103)
  %call105 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable

bb68:                                             ; preds = %bb66
  %inttoptr = inttoptr i64 %load97 to ptr
  %gep106 = getelementptr i64, ptr %inttoptr, i64 0
  %load107 = load i64, ptr %gep106, align 4
  %call108 = call i32 (ptr, ...) @printf(ptr @global_str.6, i64 %load107)
  %call109 = call i32 (ptr, ...) @printf(ptr @global_str.7, i64 -9223372036854775808)
  %load110 = load i64, ptr %point3, align 4
  %cmpne111 = icmp ne i64 %load110, 0
  %zext112 = zext i1 %cmpne111 to i64
  %cond_true113 = icmp ne i64 %zext112, 0
  br i1 %cond_true113, label %bb70, label %bb71

bb69:                                             ; preds = %bb66
  %message114 = alloca ptr, align 8
  store ptr @global_str.8, ptr %message114, align 8
  %call115 = call i32 (ptr, ...) @printf(ptr @global_str)
  %load116 = load ptr, ptr %message114, align 8
  %call117 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load116)
  %call118 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable

bb70:                                             ; preds = %bb68
  %int_to_ptr119 = inttoptr i64 %load110 to ptr
  %is_not_null120 = icmp ne ptr %int_to_ptr119, null
  br i1 %is_not_null120, label %arc.release.do, label %arc.release.cont

bb71:                                             ; preds = %arc.release.cont, %bb68
  ret i64 0

arc.retain.do:                                    ; preds = %bb52
  %ref_ptr = getelementptr i64, ptr %int_to_ptr, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 4
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 4
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb52
  br label %bb53

arc.retain.do17:                                  ; preds = %bb56
  %ref_ptr19 = getelementptr i64, ptr %int_to_ptr15, i64 -2
  %current_count20 = load i64, ptr %ref_ptr19, align 4
  %new_count21 = add i64 %current_count20, 1
  store i64 %new_count21, ptr %ref_ptr19, align 4
  br label %arc.retain.cont18

arc.retain.cont18:                                ; preds = %arc.retain.do17, %bb56
  br label %bb57

arc.retain.do40:                                  ; preds = %bb60
  %ref_ptr42 = getelementptr i64, ptr %int_to_ptr38, i64 -2
  %current_count43 = load i64, ptr %ref_ptr42, align 4
  %new_count44 = add i64 %current_count43, 1
  store i64 %new_count44, ptr %ref_ptr42, align 4
  br label %arc.retain.cont41

arc.retain.cont41:                                ; preds = %arc.retain.do40, %bb60
  br label %bb61

arc.retain.do63:                                  ; preds = %bb64
  %ref_ptr65 = getelementptr i64, ptr %int_to_ptr61, i64 -2
  %current_count66 = load i64, ptr %ref_ptr65, align 4
  %new_count67 = add i64 %current_count66, 1
  store i64 %new_count67, ptr %ref_ptr65, align 4
  br label %arc.retain.cont64

arc.retain.cont64:                                ; preds = %arc.retain.do63, %bb64
  br label %bb65

arc.release.do:                                   ; preds = %bb70
  %ref_ptr121 = getelementptr i64, ptr %int_to_ptr119, i64 -2
  %current_count122 = load i64, ptr %ref_ptr121, align 4
  %new_count123 = sub i64 %current_count122, 1
  store i64 %new_count123, ptr %ref_ptr121, align 4
  %is_zero = icmp eq i64 %new_count123, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb70
  br label %bb71

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr121)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont
}

define float @Point3_f32_add3(ptr %0) {
bb74:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb75, label %bb76

bb75:                                             ; preds = %bb74
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb77, label %bb78

bb76:                                             ; preds = %bb74
  %message = alloca ptr, align 8
  store ptr @global_str.8, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str)
  %load7 = load ptr, ptr %message, align 8
  %call8 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load7)
  %call9 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable

bb77:                                             ; preds = %bb75
  %gep10 = getelementptr %Point3_f32, ptr %load2, i64 0, i32 1
  %load11 = load float, ptr %gep10, align 4
  %fadd = fadd float %load1, %load11
  %load12 = load ptr, ptr %self, align 8
  %ptr2int13 = ptrtoint ptr %load12 to i64
  %cmpne14 = icmp ne i64 %ptr2int13, 0
  %zext15 = zext i1 %cmpne14 to i64
  %cond_true16 = icmp ne i64 %zext15, 0
  br i1 %cond_true16, label %bb79, label %bb80

bb78:                                             ; preds = %bb75
  %message17 = alloca ptr, align 8
  store ptr @global_str.8, ptr %message17, align 8
  %call18 = call i32 (ptr, ...) @printf(ptr @global_str)
  %load19 = load ptr, ptr %message17, align 8
  %call20 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load19)
  %call21 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable

bb79:                                             ; preds = %bb77
  %gep22 = getelementptr %Point3_f32, ptr %load12, i64 0, i32 2
  %load23 = load float, ptr %gep22, align 4
  %fadd24 = fadd float %fadd, %load23
  ret float %fadd24

bb80:                                             ; preds = %bb77
  %message25 = alloca ptr, align 8
  store ptr @global_str.8, ptr %message25, align 8
  %call26 = call i32 (ptr, ...) @printf(ptr @global_str)
  %load27 = load ptr, ptr %message25, align 8
  %call28 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load27)
  %call29 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable
}

define float @Point3_f32_getX(ptr %0) {
bb81:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb82, label %bb83

bb82:                                             ; preds = %bb81
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb83:                                             ; preds = %bb81
  %message = alloca ptr, align 8
  store ptr @global_str.8, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str)
  %load2 = load ptr, ptr %message, align 8
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load2)
  %call4 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable
}

define float @Point3_f32_getY(ptr %0) {
bb84:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb85, label %bb86

bb85:                                             ; preds = %bb84
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 1
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb86:                                             ; preds = %bb84
  %message = alloca ptr, align 8
  store ptr @global_str.8, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str)
  %load2 = load ptr, ptr %message, align 8
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load2)
  %call4 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable
}

define float @Point3_f32_getZ(ptr %0) {
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
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 2
  %load1 = load float, ptr %gep, align 4
  ret float %load1

bb89:                                             ; preds = %bb87
  %message = alloca ptr, align 8
  store ptr @global_str.8, ptr %message, align 8
  %call = call i32 (ptr, ...) @printf(ptr @global_str)
  %load2 = load ptr, ptr %message, align 8
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.1, ptr %load2)
  %call4 = call i32 (ptr, ...) @printf(ptr @global_str.2)
  call void @exit(i32 1)
  unreachable
}

define void @__global_init() {
bb0:
  ret void
}
