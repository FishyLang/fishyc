; ModuleID = 'fishy_module'
source_filename = "fishy_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%__fishy_generic_3_Vec_5b537472696e675d = type { ptr, i64, i64 }
%__fishy_generic_6_Point3_5b4633325d = type { float, float, float }

@global_str = private constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00"
@global_str.1 = private constant [11 x i8] c"FATAL: %s\0A\00"
@global_str.2 = private constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00"
@global_str.3 = private constant [39 x i8] c"read_line buffer size must be positive\00"
@global_str.4 = private constant [27 x i8] c"Out of memory in read_line\00"
@global_str.5 = private constant [40 x i8] c"Null pointer dereference (Array Write)!\00"
@global_str.6 = private constant [39 x i8] c"Null pointer dereference (Array Read)!\00"
@global_str.7 = private constant [41 x i8] c"Null pointer dereference on Method Call!\00"
@global_str.8 = private constant [27 x i8] c"Out of memory in substring\00"
@oom.message = private unnamed_addr constant [14 x i8] c"Out of memory\00", align 1
@global_str.9 = private constant [24 x i8] c"Out of memory in concat\00"
@oom.message.10 = private unnamed_addr constant [14 x i8] c"Out of memory\00", align 1
@global_str.11 = private constant [4 x i8] c"%f\0A\00"
@global_str.12 = private constant [43 x i8] c"Null pointer dereference on Property Read!\00"
@global_str.13 = private constant [5 x i8] c"John\00"
@global_str.14 = private constant [4 x i8] c"%d\0A\00"
@global_str.15 = private constant [4 x i8] c"%s\0A\00"
@global_str.16 = private constant [5 x i8] c"true\00"
@global_str.17 = private constant [6 x i8] c"false\00"
@global_str.18 = private constant [2 x i8] c"h\00"
@global_str.19 = private constant [2 x i8] c"J\00"
@global_str.20 = private constant [2 x i8] c"n\00"
@global_str.21 = private constant [59 x i8] c"write down your favorite fruits separated by a dash (-): \0A\00"
@global_str.22 = private constant [2 x i8] c"-\00"
@global_str.23 = private constant [19 x i8] c"\0Ayour fruits are:\0A\00"
@global_str.24 = private constant [6 x i8] c"- %s\0A\00"
@global_str.25 = private constant [6 x i8] c"%lld\0A\00"
@global_str.26 = private constant [38 x i8] c"Integer division or remainder by zero\00"
@global_str.27 = private constant [22 x i8] c"Vec capacity overflow\00"
@global_str.28 = private constant [44 x i8] c"Null pointer dereference on Property Write!\00"
@global_str.29 = private constant [33 x i8] c"out of memory during Vec#reserve\00"
@global_str.30 = private constant [19 x i8] c"Null field address\00"
@global_str.31 = private constant [31 x i8] c"index out of bounds in Vec#get\00"

declare ptr @malloc(i64)

declare void @free(ptr)

declare ptr @realloc(ptr, i64)

declare ptr @memcpy(ptr, ptr, i64)

declare ptr @memset(ptr, i32, i64)

declare i32 @printf(ptr, ...)

declare ptr @fopen(ptr, ptr)

declare i32 @fclose(ptr)

declare i32 @fputs(ptr, ptr)

declare i32 @fprintf(ptr, ptr, ...)

declare void @exit(i32)

declare i32 @getchar()

declare i64 @strlen(ptr)

declare ptr @strcat(ptr, ptr)

declare ptr @strstr(ptr, ptr)

declare i32 @strcmp(ptr, ptr)

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
  unreachable
}

define ptr @read_line(i64 %0) {
bb2:
  %j = alloca i64, align 8
  %final_str = alloca ptr, align 8
  %c = alloca i32, align 4
  %reading = alloca i1, align 1
  %i = alloca i64, align 8
  %buffer = alloca ptr, align 8
  %buf_size = alloca i64, align 8
  store i64 %0, ptr %buf_size, align 8
  %load = load i64, ptr %buf_size, align 8
  %cmpeq = icmp eq i64 %load, 0
  %zext = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true = icmp ne i64 %cast_zext, 0
  br i1 %cond_true, label %bb3, label %bb4

bb3:                                              ; preds = %bb2
  call void @panic(ptr @global_str.3)
  unreachable

bb4:                                              ; preds = %bb2
  store ptr null, ptr %buffer, align 8
  %load1 = load i64, ptr %buf_size, align 8
  %call = call ptr @malloc(i64 %load1)
  store ptr %call, ptr %buffer, align 8
  %load2 = load ptr, ptr %buffer, align 8
  %ptr2int = ptrtoint ptr %load2 to i64
  %cmpeq3 = icmp eq i64 %ptr2int, 0
  %zext4 = zext i1 %cmpeq3 to i64
  %cast_trunc5 = trunc i64 %zext4 to i1
  %cast_zext6 = zext i1 %cast_trunc5 to i64
  %cond_true7 = icmp ne i64 %cast_zext6, 0
  br i1 %cond_true7, label %bb5, label %bb6

bb5:                                              ; preds = %bb4
  call void @panic(ptr @global_str.4)
  unreachable

bb6:                                              ; preds = %bb4
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  store i1 false, ptr %reading, align 1
  store i1 true, ptr %reading, align 1
  br label %bb7

bb7:                                              ; preds = %bb18, %bb15, %bb13, %bb10, %bb6
  %load8 = load i1, ptr %reading, align 1
  %load_zext = zext i1 %load8 to i64
  %cast_trunc9 = trunc i64 %load_zext to i1
  %cast_zext10 = zext i1 %cast_trunc9 to i64
  %cond_true11 = icmp ne i64 %cast_zext10, 0
  br i1 %cond_true11, label %bb8, label %bb9

bb8:                                              ; preds = %bb7
  store i32 0, ptr %c, align 4
  %call12 = call i32 @getchar()
  %cast_sext = sext i32 %call12 to i64
  %trunc = trunc i64 %cast_sext to i32
  store i32 %trunc, ptr %c, align 4
  %load13 = load i32, ptr %c, align 4
  %load_sext = sext i32 %load13 to i64
  %cast_trunc14 = trunc i64 %load_sext to i32
  %cast_sext15 = sext i32 %cast_trunc14 to i64
  %cmpeq16 = icmp eq i64 %cast_sext15, -1
  %zext17 = zext i1 %cmpeq16 to i64
  %cast_trunc18 = trunc i64 %zext17 to i1
  %cast_zext19 = zext i1 %cast_trunc18 to i64
  %cond_true20 = icmp ne i64 %cast_zext19, 0
  br i1 %cond_true20, label %bb10, label %bb12

bb9:                                              ; preds = %bb7
  %load21 = load ptr, ptr %buffer, align 8
  %ptr2int22 = ptrtoint ptr %load21 to i64
  %cmpne = icmp ne i64 %ptr2int22, 0
  %zext23 = zext i1 %cmpne to i64
  %cond_true24 = icmp ne i64 %zext23, 0
  br i1 %cond_true24, label %bb20, label %bb21

bb10:                                             ; preds = %bb8
  store i1 false, ptr %reading, align 1
  br label %bb7

bb12:                                             ; preds = %bb8
  %load25 = load i32, ptr %c, align 4
  %load_sext26 = sext i32 %load25 to i64
  %cast_trunc27 = trunc i64 %load_sext26 to i32
  %cast_sext28 = sext i32 %cast_trunc27 to i64
  %cmpeq29 = icmp eq i64 %cast_sext28, 10
  %zext30 = zext i1 %cmpeq29 to i64
  %cast_trunc31 = trunc i64 %zext30 to i1
  %cast_zext32 = zext i1 %cast_trunc31 to i64
  %cond_true33 = icmp ne i64 %cast_zext32, 0
  br i1 %cond_true33, label %bb13, label %bb15

bb13:                                             ; preds = %bb12
  store i1 false, ptr %reading, align 1
  br label %bb7

bb15:                                             ; preds = %bb12
  %load34 = load i64, ptr %i, align 8
  %load35 = load i64, ptr %buf_size, align 8
  %sub = sub i64 %load35, 1
  %cmplt = icmp ult i64 %load34, %sub
  %zext36 = zext i1 %cmplt to i64
  %cast_trunc37 = trunc i64 %zext36 to i1
  %cast_zext38 = zext i1 %cast_trunc37 to i64
  %cond_true39 = icmp ne i64 %cast_zext38, 0
  br i1 %cond_true39, label %bb16, label %bb7

bb16:                                             ; preds = %bb15
  %load40 = load ptr, ptr %buffer, align 8
  %ptr2int41 = ptrtoint ptr %load40 to i64
  %cmpne42 = icmp ne i64 %ptr2int41, 0
  %zext43 = zext i1 %cmpne42 to i64
  %cond_true44 = icmp ne i64 %zext43, 0
  br i1 %cond_true44, label %bb18, label %bb19

bb18:                                             ; preds = %bb16
  %load45 = load i64, ptr %i, align 8
  %load46 = load i32, ptr %c, align 4
  %load_sext47 = sext i32 %load46 to i64
  %cast_trunc48 = trunc i64 %load_sext47 to i32
  %cast_sext49 = sext i32 %cast_trunc48 to i64
  %cast_trunc50 = trunc i64 %cast_sext49 to i8
  %cast_zext51 = zext i8 %cast_trunc50 to i64
  %gep = getelementptr i8, ptr %load40, i64 %load45
  %trunc52 = trunc i64 %cast_zext51 to i8
  store i8 %trunc52, ptr %gep, align 1
  %load53 = load i64, ptr %i, align 8
  %add = add i64 %load53, 1
  store i64 %add, ptr %i, align 8
  br label %bb7

bb19:                                             ; preds = %bb16
  call void @panic(ptr @global_str.5)
  unreachable

bb20:                                             ; preds = %bb9
  %load54 = load i64, ptr %i, align 8
  %gep55 = getelementptr i8, ptr %load21, i64 %load54
  store i8 0, ptr %gep55, align 1
  store ptr null, ptr %final_str, align 8
  %load56 = load i64, ptr %i, align 8
  %add57 = add i64 %load56, 1
  %call58 = call ptr @malloc(i64 %add57)
  store ptr %call58, ptr %final_str, align 8
  %load59 = load ptr, ptr %final_str, align 8
  %ptr2int60 = ptrtoint ptr %load59 to i64
  %cmpeq61 = icmp eq i64 %ptr2int60, 0
  %zext62 = zext i1 %cmpeq61 to i64
  %cast_trunc63 = trunc i64 %zext62 to i1
  %cast_zext64 = zext i1 %cast_trunc63 to i64
  %cond_true65 = icmp ne i64 %cast_zext64, 0
  br i1 %cond_true65, label %bb22, label %bb23

bb21:                                             ; preds = %bb9
  call void @panic(ptr @global_str.5)
  unreachable

bb22:                                             ; preds = %bb20
  call void @panic(ptr @global_str.4)
  unreachable

bb23:                                             ; preds = %bb20
  store i64 0, ptr %j, align 8
  store i64 0, ptr %j, align 8
  br label %bb24

bb24:                                             ; preds = %bb29, %bb23
  %load66 = load i64, ptr %j, align 8
  %load67 = load i64, ptr %i, align 8
  %cmplt68 = icmp ult i64 %load66, %load67
  %zext69 = zext i1 %cmplt68 to i64
  %cast_trunc70 = trunc i64 %zext69 to i1
  %cast_zext71 = zext i1 %cast_trunc70 to i64
  %cond_true72 = icmp ne i64 %cast_zext71, 0
  br i1 %cond_true72, label %bb25, label %bb26

bb25:                                             ; preds = %bb24
  %load73 = load ptr, ptr %final_str, align 8
  %ptr2int74 = ptrtoint ptr %load73 to i64
  %cmpne75 = icmp ne i64 %ptr2int74, 0
  %zext76 = zext i1 %cmpne75 to i64
  %cond_true77 = icmp ne i64 %zext76, 0
  br i1 %cond_true77, label %bb27, label %bb28

bb26:                                             ; preds = %bb24
  %load78 = load ptr, ptr %final_str, align 8
  %ptr2int79 = ptrtoint ptr %load78 to i64
  %cmpne80 = icmp ne i64 %ptr2int79, 0
  %zext81 = zext i1 %cmpne80 to i64
  %cond_true82 = icmp ne i64 %zext81, 0
  br i1 %cond_true82, label %bb31, label %bb32

bb27:                                             ; preds = %bb25
  %load83 = load i64, ptr %j, align 8
  %load84 = load ptr, ptr %buffer, align 8
  %ptr2int85 = ptrtoint ptr %load84 to i64
  %cmpne86 = icmp ne i64 %ptr2int85, 0
  %zext87 = zext i1 %cmpne86 to i64
  %cond_true88 = icmp ne i64 %zext87, 0
  br i1 %cond_true88, label %bb29, label %bb30

bb28:                                             ; preds = %bb25
  call void @panic(ptr @global_str.5)
  unreachable

bb29:                                             ; preds = %bb27
  %load89 = load i64, ptr %j, align 8
  %gep90 = getelementptr i8, ptr %load84, i64 %load89
  %load91 = load i8, ptr %gep90, align 1
  %load_zext92 = zext i8 %load91 to i64
  %cast_trunc93 = trunc i64 %load_zext92 to i8
  %cast_zext94 = zext i8 %cast_trunc93 to i64
  %gep95 = getelementptr i8, ptr %load73, i64 %load83
  %trunc96 = trunc i64 %cast_zext94 to i8
  store i8 %trunc96, ptr %gep95, align 1
  %load97 = load i64, ptr %j, align 8
  %add98 = add i64 %load97, 1
  store i64 %add98, ptr %j, align 8
  br label %bb24

bb30:                                             ; preds = %bb27
  call void @panic(ptr @global_str.6)
  unreachable

bb31:                                             ; preds = %bb26
  %load99 = load i64, ptr %i, align 8
  %gep100 = getelementptr i8, ptr %load78, i64 %load99
  store i8 0, ptr %gep100, align 1
  %load101 = load ptr, ptr %buffer, align 8
  call void @free(ptr %load101)
  store ptr null, ptr %buffer, align 8
  %load102 = load ptr, ptr %final_str, align 8
  ret ptr %load102

bb32:                                             ; preds = %bb26
  call void @panic(ptr @global_str.5)
  unreachable
}

define i64 @__fishy_member_6_string_3_len(ptr %0) {
bb106:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %call = call i64 @strlen(ptr %load)
  ret i64 %call
}

define i1 @__fishy_member_6_string_8_contains(ptr %0, ptr %1) {
bb110:
  %self1 = alloca ptr, align 8
  %sub = alloca ptr, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  store ptr %1, ptr %sub, align 8
  %load = load ptr, ptr %sub, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb111, label %bb112

bb111:                                            ; preds = %bb110
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  %cmpeq = icmp eq i64 %call, 0
  %zext3 = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext3 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true4 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true4, label %bb113, label %bb114

bb112:                                            ; preds = %bb110
  call void @panic(ptr @global_str.7)
  unreachable

bb113:                                            ; preds = %bb111
  ret i1 false

bb114:                                            ; preds = %bb111
  %load5 = load ptr, ptr %self, align 8
  %load6 = load ptr, ptr %sub, align 8
  %call7 = call ptr @strstr(ptr %load5, ptr %load6)
  %ptr2int8 = ptrtoint ptr %call7 to i64
  %cmpne9 = icmp ne i64 %ptr2int8, 0
  %zext10 = zext i1 %cmpne9 to i64
  %cast_trunc11 = trunc i64 %zext10 to i1
  %cast_zext12 = zext i1 %cast_trunc11 to i64
  %ret_trunc = trunc i64 %cast_zext12 to i1
  ret i1 %ret_trunc
}

define i1 @__fishy_member_6_string_11_starts_with(ptr %0, ptr %1) {
bb115:
  %i = alloca i64, align 8
  %self8 = alloca ptr, align 8
  %self1 = alloca ptr, align 8
  %p_len = alloca i64, align 8
  %prefix = alloca ptr, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  store ptr %1, ptr %prefix, align 8
  store i64 0, ptr %p_len, align 8
  %load = load ptr, ptr %prefix, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb116, label %bb117

bb116:                                            ; preds = %bb115
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  store i64 %call, ptr %p_len, align 8
  %load3 = load ptr, ptr %self, align 8
  %ptr2int4 = ptrtoint ptr %load3 to i64
  %cmpne5 = icmp ne i64 %ptr2int4, 0
  %zext6 = zext i1 %cmpne5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb118, label %bb119

bb117:                                            ; preds = %bb115
  call void @panic(ptr @global_str.7)
  unreachable

bb118:                                            ; preds = %bb116
  store ptr %load3, ptr %self8, align 8
  %load9 = load ptr, ptr %self8, align 8
  %call10 = call i64 @strlen(ptr %load9)
  %load11 = load i64, ptr %p_len, align 8
  %cmplt = icmp ult i64 %call10, %load11
  %zext12 = zext i1 %cmplt to i64
  %cast_trunc = trunc i64 %zext12 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true13 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true13, label %bb120, label %bb121

bb119:                                            ; preds = %bb116
  call void @panic(ptr @global_str.7)
  unreachable

bb120:                                            ; preds = %bb118
  ret i1 false

bb121:                                            ; preds = %bb118
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  br label %bb122

bb122:                                            ; preds = %bb130, %bb121
  %load14 = load i64, ptr %i, align 8
  %load15 = load i64, ptr %p_len, align 8
  %cmplt16 = icmp ult i64 %load14, %load15
  %zext17 = zext i1 %cmplt16 to i64
  %cast_trunc18 = trunc i64 %zext17 to i1
  %cast_zext19 = zext i1 %cast_trunc18 to i64
  %cond_true20 = icmp ne i64 %cast_zext19, 0
  br i1 %cond_true20, label %bb123, label %bb124

bb123:                                            ; preds = %bb122
  %load21 = load ptr, ptr %self, align 8
  %ptr2int22 = ptrtoint ptr %load21 to i64
  %cmpne23 = icmp ne i64 %ptr2int22, 0
  %zext24 = zext i1 %cmpne23 to i64
  %cond_true25 = icmp ne i64 %zext24, 0
  br i1 %cond_true25, label %bb125, label %bb126

bb124:                                            ; preds = %bb122
  ret i1 true

bb125:                                            ; preds = %bb123
  %load26 = load i64, ptr %i, align 8
  %gep = getelementptr i8, ptr %load21, i64 %load26
  %load27 = load i8, ptr %gep, align 1
  %load_sext = sext i8 %load27 to i64
  %cast_trunc28 = trunc i64 %load_sext to i8
  %cast_zext29 = zext i8 %cast_trunc28 to i64
  %load30 = load ptr, ptr %prefix, align 8
  %ptr2int31 = ptrtoint ptr %load30 to i64
  %cmpne32 = icmp ne i64 %ptr2int31, 0
  %zext33 = zext i1 %cmpne32 to i64
  %cond_true34 = icmp ne i64 %zext33, 0
  br i1 %cond_true34, label %bb127, label %bb128

bb126:                                            ; preds = %bb123
  call void @panic(ptr @global_str.6)
  unreachable

bb127:                                            ; preds = %bb125
  %load35 = load i64, ptr %i, align 8
  %gep36 = getelementptr i8, ptr %load30, i64 %load35
  %load37 = load i8, ptr %gep36, align 1
  %load_sext38 = sext i8 %load37 to i64
  %cast_trunc39 = trunc i64 %load_sext38 to i8
  %cast_zext40 = zext i8 %cast_trunc39 to i64
  %cmpne41 = icmp ne i64 %cast_zext29, %cast_zext40
  %zext42 = zext i1 %cmpne41 to i64
  %cast_trunc43 = trunc i64 %zext42 to i1
  %cast_zext44 = zext i1 %cast_trunc43 to i64
  %cond_true45 = icmp ne i64 %cast_zext44, 0
  br i1 %cond_true45, label %bb129, label %bb130

bb128:                                            ; preds = %bb125
  call void @panic(ptr @global_str.6)
  unreachable

bb129:                                            ; preds = %bb127
  ret i1 false

bb130:                                            ; preds = %bb127
  %load46 = load i64, ptr %i, align 8
  %add = add i64 %load46, 1
  store i64 %add, ptr %i, align 8
  br label %bb122
}

define i1 @__fishy_member_6_string_9_ends_with(ptr %0, ptr %1) {
bb131:
  %i = alloca i64, align 8
  %offset = alloca i64, align 8
  %self8 = alloca ptr, align 8
  %sub_len = alloca i64, align 8
  %self1 = alloca ptr, align 8
  %s_len = alloca i64, align 8
  %suffix = alloca ptr, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  store ptr %1, ptr %suffix, align 8
  store i64 0, ptr %s_len, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb132, label %bb133

bb132:                                            ; preds = %bb131
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  store i64 %call, ptr %s_len, align 8
  store i64 0, ptr %sub_len, align 8
  %load3 = load ptr, ptr %suffix, align 8
  %ptr2int4 = ptrtoint ptr %load3 to i64
  %cmpne5 = icmp ne i64 %ptr2int4, 0
  %zext6 = zext i1 %cmpne5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb134, label %bb135

bb133:                                            ; preds = %bb131
  call void @panic(ptr @global_str.7)
  unreachable

bb134:                                            ; preds = %bb132
  store ptr %load3, ptr %self8, align 8
  %load9 = load ptr, ptr %self8, align 8
  %call10 = call i64 @strlen(ptr %load9)
  store i64 %call10, ptr %sub_len, align 8
  %load11 = load i64, ptr %sub_len, align 8
  %load12 = load i64, ptr %s_len, align 8
  %cmpgt = icmp ugt i64 %load11, %load12
  %zext13 = zext i1 %cmpgt to i64
  %cast_trunc = trunc i64 %zext13 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true14 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true14, label %bb136, label %bb137

bb135:                                            ; preds = %bb132
  call void @panic(ptr @global_str.7)
  unreachable

bb136:                                            ; preds = %bb134
  ret i1 false

bb137:                                            ; preds = %bb134
  store i64 0, ptr %offset, align 8
  %load15 = load i64, ptr %s_len, align 8
  %load16 = load i64, ptr %sub_len, align 8
  %sub = sub i64 %load15, %load16
  store i64 %sub, ptr %offset, align 8
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  br label %bb138

bb138:                                            ; preds = %bb146, %bb137
  %load17 = load i64, ptr %i, align 8
  %load18 = load i64, ptr %sub_len, align 8
  %cmplt = icmp ult i64 %load17, %load18
  %zext19 = zext i1 %cmplt to i64
  %cast_trunc20 = trunc i64 %zext19 to i1
  %cast_zext21 = zext i1 %cast_trunc20 to i64
  %cond_true22 = icmp ne i64 %cast_zext21, 0
  br i1 %cond_true22, label %bb139, label %bb140

bb139:                                            ; preds = %bb138
  %load23 = load ptr, ptr %self, align 8
  %ptr2int24 = ptrtoint ptr %load23 to i64
  %cmpne25 = icmp ne i64 %ptr2int24, 0
  %zext26 = zext i1 %cmpne25 to i64
  %cond_true27 = icmp ne i64 %zext26, 0
  br i1 %cond_true27, label %bb141, label %bb142

bb140:                                            ; preds = %bb138
  ret i1 true

bb141:                                            ; preds = %bb139
  %load28 = load i64, ptr %offset, align 8
  %load29 = load i64, ptr %i, align 8
  %add = add i64 %load28, %load29
  %gep = getelementptr i8, ptr %load23, i64 %add
  %load30 = load i8, ptr %gep, align 1
  %load_sext = sext i8 %load30 to i64
  %cast_trunc31 = trunc i64 %load_sext to i8
  %cast_zext32 = zext i8 %cast_trunc31 to i64
  %load33 = load ptr, ptr %suffix, align 8
  %ptr2int34 = ptrtoint ptr %load33 to i64
  %cmpne35 = icmp ne i64 %ptr2int34, 0
  %zext36 = zext i1 %cmpne35 to i64
  %cond_true37 = icmp ne i64 %zext36, 0
  br i1 %cond_true37, label %bb143, label %bb144

bb142:                                            ; preds = %bb139
  call void @panic(ptr @global_str.6)
  unreachable

bb143:                                            ; preds = %bb141
  %load38 = load i64, ptr %i, align 8
  %gep39 = getelementptr i8, ptr %load33, i64 %load38
  %load40 = load i8, ptr %gep39, align 1
  %load_sext41 = sext i8 %load40 to i64
  %cast_trunc42 = trunc i64 %load_sext41 to i8
  %cast_zext43 = zext i8 %cast_trunc42 to i64
  %cmpne44 = icmp ne i64 %cast_zext32, %cast_zext43
  %zext45 = zext i1 %cmpne44 to i64
  %cast_trunc46 = trunc i64 %zext45 to i1
  %cast_zext47 = zext i1 %cast_trunc46 to i64
  %cond_true48 = icmp ne i64 %cast_zext47, 0
  br i1 %cond_true48, label %bb145, label %bb146

bb144:                                            ; preds = %bb141
  call void @panic(ptr @global_str.6)
  unreachable

bb145:                                            ; preds = %bb143
  ret i1 false

bb146:                                            ; preds = %bb143
  %load49 = load i64, ptr %i, align 8
  %add50 = add i64 %load49, 1
  store i64 %add50, ptr %i, align 8
  br label %bb138
}

define ptr @__fishy_member_6_string_9_substring(ptr %0, i64 %1, i64 %2) {
bb148:
  %src_offset = alloca ptr, align 8
  %dest = alloca ptr, align 8
  %len = alloca i64, align 8
  %empty = alloca ptr, align 8
  %safe_end = alloca i64, align 8
  %self1 = alloca ptr, align 8
  %s_len = alloca i64, align 8
  %end = alloca i64, align 8
  %start = alloca i64, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  store i64 %1, ptr %start, align 8
  store i64 %2, ptr %end, align 8
  store i64 0, ptr %s_len, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb149, label %bb150

bb149:                                            ; preds = %bb148
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  store i64 %call, ptr %s_len, align 8
  store i64 0, ptr %safe_end, align 8
  %load3 = load i64, ptr %end, align 8
  store i64 %load3, ptr %safe_end, align 8
  %load4 = load i64, ptr %safe_end, align 8
  %load5 = load i64, ptr %s_len, align 8
  %cmpgt = icmp ugt i64 %load4, %load5
  %zext6 = zext i1 %cmpgt to i64
  %cast_trunc = trunc i64 %zext6 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true7 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true7, label %bb151, label %bb152

bb150:                                            ; preds = %bb148
  call void @panic(ptr @global_str.7)
  unreachable

bb151:                                            ; preds = %bb149
  %load8 = load i64, ptr %s_len, align 8
  store i64 %load8, ptr %safe_end, align 8
  br label %bb152

bb152:                                            ; preds = %bb151, %bb149
  %load9 = load i64, ptr %start, align 8
  %load10 = load i64, ptr %safe_end, align 8
  %cmpge = icmp uge i64 %load9, %load10
  %zext11 = zext i1 %cmpge to i64
  %cast_trunc12 = trunc i64 %zext11 to i1
  %cast_zext13 = zext i1 %cast_trunc12 to i64
  %cond_true14 = icmp ne i64 %cast_zext13, 0
  br i1 %cond_true14, label %bb153, label %bb154

bb153:                                            ; preds = %bb152
  store ptr null, ptr %empty, align 8
  %call15 = call ptr @malloc(i64 1)
  store ptr %call15, ptr %empty, align 8
  %load16 = load ptr, ptr %empty, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpeq = icmp eq i64 %ptr2int17, 0
  %zext18 = zext i1 %cmpeq to i64
  %cast_trunc19 = trunc i64 %zext18 to i1
  %cast_zext20 = zext i1 %cast_trunc19 to i64
  %cond_true21 = icmp ne i64 %cast_zext20, 0
  br i1 %cond_true21, label %bb155, label %bb156

bb154:                                            ; preds = %bb152
  store i64 0, ptr %len, align 8
  %load22 = load i64, ptr %safe_end, align 8
  %load23 = load i64, ptr %start, align 8
  %sub = sub i64 %load22, %load23
  store i64 %sub, ptr %len, align 8
  store ptr null, ptr %dest, align 8
  %load24 = load i64, ptr %len, align 8
  %add = add i64 %load24, 1
  %call25 = call ptr @malloc(i64 %add)
  store ptr %call25, ptr %dest, align 8
  %load26 = load ptr, ptr %dest, align 8
  %ptr2int27 = ptrtoint ptr %load26 to i64
  %cmpeq28 = icmp eq i64 %ptr2int27, 0
  %zext29 = zext i1 %cmpeq28 to i64
  %cast_trunc30 = trunc i64 %zext29 to i1
  %cast_zext31 = zext i1 %cast_trunc30 to i64
  %cond_true32 = icmp ne i64 %cast_zext31, 0
  br i1 %cond_true32, label %bb159, label %bb160

bb155:                                            ; preds = %bb153
  call void @panic(ptr @global_str.8)
  unreachable

bb156:                                            ; preds = %bb153
  %load33 = load ptr, ptr %empty, align 8
  %ptr2int34 = ptrtoint ptr %load33 to i64
  %cmpne35 = icmp ne i64 %ptr2int34, 0
  %zext36 = zext i1 %cmpne35 to i64
  %cond_true37 = icmp ne i64 %zext36, 0
  br i1 %cond_true37, label %bb157, label %bb158

bb157:                                            ; preds = %bb156
  %gep = getelementptr i8, ptr %load33, i64 0
  store i8 0, ptr %gep, align 1
  %load38 = load ptr, ptr %empty, align 8
  ret ptr %load38

bb158:                                            ; preds = %bb156
  call void @panic(ptr @global_str.5)
  unreachable

bb159:                                            ; preds = %bb154
  call void @panic(ptr @global_str.8)
  unreachable

bb160:                                            ; preds = %bb154
  store ptr null, ptr %src_offset, align 8
  %load39 = load ptr, ptr %self, align 8
  %ptrtoint = ptrtoint ptr %load39 to i64
  %load40 = load i64, ptr %start, align 8
  %add41 = add i64 %ptrtoint, %load40
  %inttoptr = inttoptr i64 %add41 to ptr
  store ptr %inttoptr, ptr %src_offset, align 8
  %load42 = load ptr, ptr %dest, align 8
  %load43 = load ptr, ptr %src_offset, align 8
  %load44 = load i64, ptr %len, align 8
  %call45 = call ptr @strncpy(ptr %load42, ptr %load43, i64 %load44)
  %load46 = load ptr, ptr %dest, align 8
  %ptr2int47 = ptrtoint ptr %load46 to i64
  %cmpne48 = icmp ne i64 %ptr2int47, 0
  %zext49 = zext i1 %cmpne48 to i64
  %cond_true50 = icmp ne i64 %zext49, 0
  br i1 %cond_true50, label %bb161, label %bb162

bb161:                                            ; preds = %bb160
  %load51 = load i64, ptr %len, align 8
  %gep52 = getelementptr i8, ptr %load46, i64 %load51
  store i8 0, ptr %gep52, align 1
  %load53 = load ptr, ptr %dest, align 8
  ret ptr %load53

bb162:                                            ; preds = %bb160
  call void @panic(ptr @global_str.5)
  unreachable
}

define ptr @__fishy_member_6_string_5_split(ptr %0, ptr %1) {
bb163:
  %j = alloca i64, align 8
  %match_found = alloca i1, align 1
  %i = alloca i64, align 8
  %start = alloca i64, align 8
  %self10 = alloca ptr, align 8
  %d_len = alloca i64, align 8
  %self3 = alloca ptr, align 8
  %s_len = alloca i64, align 8
  %result = alloca ptr, align 8
  %delim = alloca ptr, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  store ptr %1, ptr %delim, align 8
  store ptr null, ptr %result, align 8
  %managed.alloc = call ptr @malloc(i64 48)
  %allocated = icmp ne ptr %managed.alloc, null
  br i1 %allocated, label %allocation.ok, label %allocation.failed

bb164:                                            ; preds = %allocation.ok
  store ptr %load, ptr %self3, align 8
  %load4 = load ptr, ptr %self3, align 8
  %call = call i64 @strlen(ptr %load4)
  store i64 %call, ptr %s_len, align 8
  store i64 0, ptr %d_len, align 8
  %load5 = load ptr, ptr %delim, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb166, label %bb167

bb165:                                            ; preds = %allocation.ok
  call void @panic(ptr @global_str.7)
  unreachable

bb166:                                            ; preds = %bb164
  store ptr %load5, ptr %self10, align 8
  %load11 = load ptr, ptr %self10, align 8
  %call12 = call i64 @strlen(ptr %load11)
  store i64 %call12, ptr %d_len, align 8
  %load13 = load i64, ptr %d_len, align 8
  %cmpeq = icmp eq i64 %load13, 0
  %zext14 = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext14 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true15 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true15, label %bb168, label %bb169

bb167:                                            ; preds = %bb164
  call void @panic(ptr @global_str.7)
  unreachable

bb168:                                            ; preds = %bb166
  %load16 = load ptr, ptr %self, align 8
  %load17 = load ptr, ptr %result, align 8
  %ptr2int18 = ptrtoint ptr %load17 to i64
  %cmpne19 = icmp ne i64 %ptr2int18, 0
  %zext20 = zext i1 %cmpne19 to i64
  %cond_true21 = icmp ne i64 %zext20, 0
  br i1 %cond_true21, label %bb170, label %bb171

bb169:                                            ; preds = %bb166
  store i64 0, ptr %start, align 8
  store i64 0, ptr %start, align 8
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  br label %bb174

bb170:                                            ; preds = %bb168
  call void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_4_push(ptr %load17, ptr %load16)
  %load22 = load ptr, ptr %result, align 8
  call void @__fishy_retain(ptr %load22)
  %load23 = load ptr, ptr %result, align 8
  %ptr2int24 = ptrtoint ptr %load23 to i64
  %cmpne25 = icmp ne i64 %ptr2int24, 0
  %zext26 = zext i1 %cmpne25 to i64
  %cond_true27 = icmp ne i64 %zext26, 0
  br i1 %cond_true27, label %bb172, label %bb173

bb171:                                            ; preds = %bb168
  call void @panic(ptr @global_str.7)
  unreachable

bb172:                                            ; preds = %bb170
  call void @__fishy_release(ptr %load23)
  br label %bb173

bb173:                                            ; preds = %bb172, %bb170
  ret ptr %load22

bb174:                                            ; preds = %bb191, %bb188, %bb169
  %load28 = load i64, ptr %i, align 8
  %load29 = load i64, ptr %d_len, align 8
  %add = add i64 %load28, %load29
  %load30 = load i64, ptr %s_len, align 8
  %cmple = icmp ule i64 %add, %load30
  %zext31 = zext i1 %cmple to i64
  %cast_trunc32 = trunc i64 %zext31 to i1
  %cast_zext33 = zext i1 %cast_trunc32 to i64
  %cond_true34 = icmp ne i64 %cast_zext33, 0
  br i1 %cond_true34, label %bb175, label %bb176

bb175:                                            ; preds = %bb174
  store i1 false, ptr %match_found, align 1
  store i1 true, ptr %match_found, align 1
  store i64 0, ptr %j, align 8
  store i64 0, ptr %j, align 8
  br label %bb177

bb176:                                            ; preds = %bb174
  %load35 = load i64, ptr %start, align 8
  %load36 = load i64, ptr %s_len, align 8
  %load37 = load ptr, ptr %self, align 8
  %ptr2int38 = ptrtoint ptr %load37 to i64
  %cmpne39 = icmp ne i64 %ptr2int38, 0
  %zext40 = zext i1 %cmpne39 to i64
  %cond_true41 = icmp ne i64 %zext40, 0
  br i1 %cond_true41, label %bb193, label %bb194

bb177:                                            ; preds = %bb185, %bb175
  %load42 = load i64, ptr %j, align 8
  %load43 = load i64, ptr %d_len, align 8
  %cmplt = icmp ult i64 %load42, %load43
  %zext44 = zext i1 %cmplt to i64
  %cast_trunc45 = trunc i64 %zext44 to i1
  %cast_zext46 = zext i1 %cast_trunc45 to i64
  %cond_true47 = icmp ne i64 %cast_zext46, 0
  br i1 %cond_true47, label %bb178, label %bb179

bb178:                                            ; preds = %bb177
  %load48 = load ptr, ptr %self, align 8
  %ptr2int49 = ptrtoint ptr %load48 to i64
  %cmpne50 = icmp ne i64 %ptr2int49, 0
  %zext51 = zext i1 %cmpne50 to i64
  %cond_true52 = icmp ne i64 %zext51, 0
  br i1 %cond_true52, label %bb180, label %bb181

bb179:                                            ; preds = %bb177
  %load53 = load i1, ptr %match_found, align 1
  %load_zext = zext i1 %load53 to i64
  %cast_trunc54 = trunc i64 %load_zext to i1
  %cast_zext55 = zext i1 %cast_trunc54 to i64
  %cond_true56 = icmp ne i64 %cast_zext55, 0
  br i1 %cond_true56, label %bb186, label %bb188

bb180:                                            ; preds = %bb178
  %load57 = load i64, ptr %i, align 8
  %load58 = load i64, ptr %j, align 8
  %add59 = add i64 %load57, %load58
  %gep60 = getelementptr i8, ptr %load48, i64 %add59
  %load61 = load i8, ptr %gep60, align 1
  %load_sext = sext i8 %load61 to i64
  %cast_trunc62 = trunc i64 %load_sext to i8
  %cast_zext63 = zext i8 %cast_trunc62 to i64
  %load64 = load ptr, ptr %delim, align 8
  %ptr2int65 = ptrtoint ptr %load64 to i64
  %cmpne66 = icmp ne i64 %ptr2int65, 0
  %zext67 = zext i1 %cmpne66 to i64
  %cond_true68 = icmp ne i64 %zext67, 0
  br i1 %cond_true68, label %bb182, label %bb183

bb181:                                            ; preds = %bb178
  call void @panic(ptr @global_str.6)
  unreachable

bb182:                                            ; preds = %bb180
  %load69 = load i64, ptr %j, align 8
  %gep70 = getelementptr i8, ptr %load64, i64 %load69
  %load71 = load i8, ptr %gep70, align 1
  %load_sext72 = sext i8 %load71 to i64
  %cast_trunc73 = trunc i64 %load_sext72 to i8
  %cast_zext74 = zext i8 %cast_trunc73 to i64
  %cmpne75 = icmp ne i64 %cast_zext63, %cast_zext74
  %zext76 = zext i1 %cmpne75 to i64
  %cast_trunc77 = trunc i64 %zext76 to i1
  %cast_zext78 = zext i1 %cast_trunc77 to i64
  %cond_true79 = icmp ne i64 %cast_zext78, 0
  br i1 %cond_true79, label %bb184, label %bb185

bb183:                                            ; preds = %bb180
  call void @panic(ptr @global_str.6)
  unreachable

bb184:                                            ; preds = %bb182
  store i1 false, ptr %match_found, align 1
  br label %bb185

bb185:                                            ; preds = %bb184, %bb182
  %load80 = load i64, ptr %j, align 8
  %add81 = add i64 %load80, 1
  store i64 %add81, ptr %j, align 8
  br label %bb177

bb186:                                            ; preds = %bb179
  %load82 = load i64, ptr %start, align 8
  %load83 = load i64, ptr %i, align 8
  %load84 = load ptr, ptr %self, align 8
  %ptr2int85 = ptrtoint ptr %load84 to i64
  %cmpne86 = icmp ne i64 %ptr2int85, 0
  %zext87 = zext i1 %cmpne86 to i64
  %cond_true88 = icmp ne i64 %zext87, 0
  br i1 %cond_true88, label %bb189, label %bb190

bb188:                                            ; preds = %bb179
  %load89 = load i64, ptr %i, align 8
  %add90 = add i64 %load89, 1
  store i64 %add90, ptr %i, align 8
  br label %bb174

bb189:                                            ; preds = %bb186
  %call91 = call ptr @__fishy_member_6_string_9_substring(ptr %load84, i64 %load82, i64 %load83)
  %load92 = load ptr, ptr %result, align 8
  %ptr2int93 = ptrtoint ptr %load92 to i64
  %cmpne94 = icmp ne i64 %ptr2int93, 0
  %zext95 = zext i1 %cmpne94 to i64
  %cond_true96 = icmp ne i64 %zext95, 0
  br i1 %cond_true96, label %bb191, label %bb192

bb190:                                            ; preds = %bb186
  call void @panic(ptr @global_str.7)
  unreachable

bb191:                                            ; preds = %bb189
  call void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_4_push(ptr %load92, ptr %call91)
  %load97 = load i64, ptr %i, align 8
  %load98 = load i64, ptr %d_len, align 8
  %add99 = add i64 %load97, %load98
  store i64 %add99, ptr %i, align 8
  %load100 = load i64, ptr %i, align 8
  store i64 %load100, ptr %start, align 8
  br label %bb174

bb192:                                            ; preds = %bb189
  call void @panic(ptr @global_str.7)
  unreachable

bb193:                                            ; preds = %bb176
  %call101 = call ptr @__fishy_member_6_string_9_substring(ptr %load37, i64 %load35, i64 %load36)
  %load102 = load ptr, ptr %result, align 8
  %ptr2int103 = ptrtoint ptr %load102 to i64
  %cmpne104 = icmp ne i64 %ptr2int103, 0
  %zext105 = zext i1 %cmpne104 to i64
  %cond_true106 = icmp ne i64 %zext105, 0
  br i1 %cond_true106, label %bb195, label %bb196

bb194:                                            ; preds = %bb176
  call void @panic(ptr @global_str.7)
  unreachable

bb195:                                            ; preds = %bb193
  call void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_4_push(ptr %load102, ptr %call101)
  %load107 = load ptr, ptr %result, align 8
  call void @__fishy_retain(ptr %load107)
  %load108 = load ptr, ptr %result, align 8
  %ptr2int109 = ptrtoint ptr %load108 to i64
  %cmpne110 = icmp ne i64 %ptr2int109, 0
  %zext111 = zext i1 %cmpne110 to i64
  %cond_true112 = icmp ne i64 %zext111, 0
  br i1 %cond_true112, label %bb197, label %bb198

bb196:                                            ; preds = %bb193
  call void @panic(ptr @global_str.7)
  unreachable

bb197:                                            ; preds = %bb195
  call void @__fishy_release(ptr %load108)
  br label %bb198

bb198:                                            ; preds = %bb197, %bb195
  ret ptr %load107

allocation.ok:                                    ; preds = %bb163
  store ptr null, ptr %managed.alloc, align 8
  %rc = getelementptr i64, ptr %managed.alloc, i64 1
  store i64 1, ptr %rc, align 8
  %length = getelementptr i64, ptr %managed.alloc, i64 2
  store i64 0, ptr %length, align 8
  %data = getelementptr i64, ptr %managed.alloc, i64 3
  %gep = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %data, i64 0, i32 0
  store ptr null, ptr %gep, align 8
  %gep1 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %data, i64 0, i32 1
  store i64 0, ptr %gep1, align 8
  %gep2 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %data, i64 0, i32 2
  store i64 0, ptr %gep2, align 8
  store ptr %data, ptr %result, align 8
  store i64 0, ptr %s_len, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb164, label %bb165

allocation.failed:                                ; preds = %bb163
  call void @panic(ptr @oom.message)
  unreachable
}

define ptr @__fishy_member_6_string_6_concat(ptr %0, ptr %1) {
bb199:
  %offset_ptr = alloca ptr, align 8
  %new_str = alloca ptr, align 8
  %self8 = alloca ptr, align 8
  %len2 = alloca i64, align 8
  %self1 = alloca ptr, align 8
  %len1 = alloca i64, align 8
  %other = alloca ptr, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  store ptr %1, ptr %other, align 8
  store i64 0, ptr %len1, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb200, label %bb201

bb200:                                            ; preds = %bb199
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  store i64 %call, ptr %len1, align 8
  store i64 0, ptr %len2, align 8
  %load3 = load ptr, ptr %other, align 8
  %ptr2int4 = ptrtoint ptr %load3 to i64
  %cmpne5 = icmp ne i64 %ptr2int4, 0
  %zext6 = zext i1 %cmpne5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb202, label %bb203

bb201:                                            ; preds = %bb199
  call void @panic(ptr @global_str.7)
  unreachable

bb202:                                            ; preds = %bb200
  store ptr %load3, ptr %self8, align 8
  %load9 = load ptr, ptr %self8, align 8
  %call10 = call i64 @strlen(ptr %load9)
  store i64 %call10, ptr %len2, align 8
  store ptr null, ptr %new_str, align 8
  %load11 = load i64, ptr %len1, align 8
  %load12 = load i64, ptr %len2, align 8
  %add = add i64 %load11, %load12
  %add13 = add i64 %add, 1
  %call14 = call ptr @malloc(i64 %add13)
  store ptr %call14, ptr %new_str, align 8
  %load15 = load ptr, ptr %new_str, align 8
  %ptr2int16 = ptrtoint ptr %load15 to i64
  %cmpeq = icmp eq i64 %ptr2int16, 0
  %zext17 = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext17 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true18 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true18, label %bb204, label %bb205

bb203:                                            ; preds = %bb200
  call void @panic(ptr @global_str.7)
  unreachable

bb204:                                            ; preds = %bb202
  call void @panic(ptr @global_str.9)
  unreachable

bb205:                                            ; preds = %bb202
  %load19 = load ptr, ptr %new_str, align 8
  %load20 = load ptr, ptr %self, align 8
  %load21 = load i64, ptr %len1, align 8
  %call22 = call ptr @strncpy(ptr %load19, ptr %load20, i64 %load21)
  store ptr null, ptr %offset_ptr, align 8
  %load23 = load ptr, ptr %new_str, align 8
  %ptrtoint = ptrtoint ptr %load23 to i64
  %load24 = load i64, ptr %len1, align 8
  %add25 = add i64 %ptrtoint, %load24
  %inttoptr = inttoptr i64 %add25 to ptr
  store ptr %inttoptr, ptr %offset_ptr, align 8
  %load26 = load ptr, ptr %offset_ptr, align 8
  %load27 = load ptr, ptr %other, align 8
  %load28 = load i64, ptr %len2, align 8
  %call29 = call ptr @strncpy(ptr %load26, ptr %load27, i64 %load28)
  %load30 = load ptr, ptr %new_str, align 8
  %ptr2int31 = ptrtoint ptr %load30 to i64
  %cmpne32 = icmp ne i64 %ptr2int31, 0
  %zext33 = zext i1 %cmpne32 to i64
  %cond_true34 = icmp ne i64 %zext33, 0
  br i1 %cond_true34, label %bb206, label %bb207

bb206:                                            ; preds = %bb205
  %load35 = load i64, ptr %len1, align 8
  %load36 = load i64, ptr %len2, align 8
  %add37 = add i64 %load35, %load36
  %gep = getelementptr i8, ptr %load30, i64 %add37
  store i8 0, ptr %gep, align 1
  %load38 = load ptr, ptr %new_str, align 8
  ret ptr %load38

bb207:                                            ; preds = %bb205
  call void @panic(ptr @global_str.5)
  unreachable
}

define i64 @main() {
bb208:
  %self239 = alloca ptr, align 8
  %__inline_ret238 = alloca i64, align 8
  %i = alloca i64, align 8
  %fruits = alloca ptr, align 8
  %stdin = alloca ptr, align 8
  %n2 = alloca i8, align 1
  %n1 = alloca i8, align 1
  %self176 = alloca i1, align 1
  %__inline_ret175 = alloca ptr, align 8
  %other = alloca ptr, align 8
  %self165 = alloca ptr, align 8
  %self150 = alloca i1, align 1
  %__inline_ret149 = alloca ptr, align 8
  %self133 = alloca i1, align 1
  %__inline_ret132 = alloca ptr, align 8
  %self116 = alloca i1, align 1
  %__inline_ret115 = alloca ptr, align 8
  %self99 = alloca i1, align 1
  %__inline_ret98 = alloca ptr, align 8
  %self87 = alloca ptr, align 8
  %__inline_ret86 = alloca i1, align 1
  %self77 = alloca ptr, align 8
  %str1 = alloca ptr, align 8
  %a = alloca i64, align 8
  %self52 = alloca ptr, align 8
  %__inline_ret51 = alloca float, align 4
  %self30 = alloca ptr, align 8
  %__inline_ret29 = alloca float, align 4
  %self = alloca ptr, align 8
  %__inline_ret = alloca float, align 4
  %point3 = alloca ptr, align 8
  call void @__global_init()
  store ptr null, ptr %point3, align 8
  %managed.alloc = call ptr @malloc(i64 36)
  %allocated = icmp ne ptr %managed.alloc, null
  br i1 %allocated, label %allocation.ok, label %allocation.failed

bb209:                                            ; preds = %allocation.ok
  %call = call float @__fishy_member_35___fishy_generic_6_Point3_5b4633325d_4_add3(ptr %load)
  %vararg_fpext = fpext float %call to double
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.11, double %vararg_fpext)
  %load4 = load ptr, ptr %point3, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb211, label %bb212

bb210:                                            ; preds = %allocation.ok
  call void @panic(ptr @global_str.7)
  unreachable

bb211:                                            ; preds = %bb209
  store ptr %load4, ptr %self, align 8
  call void @__fishy_retain(ptr %load4)
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb635, label %bb636

bb635:                                            ; preds = %bb211
  %gep14 = getelementptr %__fishy_generic_6_Point3_5b4633325d, ptr %load9, i64 0, i32 0
  %load15 = load float, ptr %gep14, align 4
  %load16 = load ptr, ptr %self, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpne18 = icmp ne i64 %ptr2int17, 0
  %zext19 = zext i1 %cmpne18 to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb637, label %bb638

bb636:                                            ; preds = %bb211
  call void @panic(ptr @global_str.12)
  unreachable

bb637:                                            ; preds = %bb635
  call void @__fishy_release(ptr %load16)
  br label %bb638

bb638:                                            ; preds = %bb637, %bb635
  store float %load15, ptr %__inline_ret, align 4
  %load21 = load float, ptr %__inline_ret, align 4
  %vararg_fpext22 = fpext float %load21 to double
  %call23 = call i32 (ptr, ...) @printf(ptr @global_str.11, double %vararg_fpext22)
  %load24 = load ptr, ptr %point3, align 8
  %ptr2int25 = ptrtoint ptr %load24 to i64
  %cmpne26 = icmp ne i64 %ptr2int25, 0
  %zext27 = zext i1 %cmpne26 to i64
  %cond_true28 = icmp ne i64 %zext27, 0
  br i1 %cond_true28, label %bb213, label %bb214

bb212:                                            ; preds = %bb209
  call void @panic(ptr @global_str.7)
  unreachable

bb213:                                            ; preds = %bb638
  store ptr %load24, ptr %self30, align 8
  call void @__fishy_retain(ptr %load24)
  %load31 = load ptr, ptr %self30, align 8
  %ptr2int32 = ptrtoint ptr %load31 to i64
  %cmpne33 = icmp ne i64 %ptr2int32, 0
  %zext34 = zext i1 %cmpne33 to i64
  %cond_true35 = icmp ne i64 %zext34, 0
  br i1 %cond_true35, label %bb641, label %bb642

bb641:                                            ; preds = %bb213
  %gep36 = getelementptr %__fishy_generic_6_Point3_5b4633325d, ptr %load31, i64 0, i32 1
  %load37 = load float, ptr %gep36, align 4
  %load38 = load ptr, ptr %self30, align 8
  %ptr2int39 = ptrtoint ptr %load38 to i64
  %cmpne40 = icmp ne i64 %ptr2int39, 0
  %zext41 = zext i1 %cmpne40 to i64
  %cond_true42 = icmp ne i64 %zext41, 0
  br i1 %cond_true42, label %bb643, label %bb644

bb642:                                            ; preds = %bb213
  call void @panic(ptr @global_str.12)
  unreachable

bb643:                                            ; preds = %bb641
  call void @__fishy_release(ptr %load38)
  br label %bb644

bb644:                                            ; preds = %bb643, %bb641
  store float %load37, ptr %__inline_ret29, align 4
  %load43 = load float, ptr %__inline_ret29, align 4
  %vararg_fpext44 = fpext float %load43 to double
  %call45 = call i32 (ptr, ...) @printf(ptr @global_str.11, double %vararg_fpext44)
  %load46 = load ptr, ptr %point3, align 8
  %ptr2int47 = ptrtoint ptr %load46 to i64
  %cmpne48 = icmp ne i64 %ptr2int47, 0
  %zext49 = zext i1 %cmpne48 to i64
  %cond_true50 = icmp ne i64 %zext49, 0
  br i1 %cond_true50, label %bb215, label %bb216

bb214:                                            ; preds = %bb638
  call void @panic(ptr @global_str.7)
  unreachable

bb215:                                            ; preds = %bb644
  store ptr %load46, ptr %self52, align 8
  call void @__fishy_retain(ptr %load46)
  %load53 = load ptr, ptr %self52, align 8
  %ptr2int54 = ptrtoint ptr %load53 to i64
  %cmpne55 = icmp ne i64 %ptr2int54, 0
  %zext56 = zext i1 %cmpne55 to i64
  %cond_true57 = icmp ne i64 %zext56, 0
  br i1 %cond_true57, label %bb647, label %bb648

bb647:                                            ; preds = %bb215
  %gep58 = getelementptr %__fishy_generic_6_Point3_5b4633325d, ptr %load53, i64 0, i32 2
  %load59 = load float, ptr %gep58, align 4
  %load60 = load ptr, ptr %self52, align 8
  %ptr2int61 = ptrtoint ptr %load60 to i64
  %cmpne62 = icmp ne i64 %ptr2int61, 0
  %zext63 = zext i1 %cmpne62 to i64
  %cond_true64 = icmp ne i64 %zext63, 0
  br i1 %cond_true64, label %bb649, label %bb650

bb648:                                            ; preds = %bb215
  call void @panic(ptr @global_str.12)
  unreachable

bb649:                                            ; preds = %bb647
  call void @__fishy_release(ptr %load60)
  br label %bb650

bb650:                                            ; preds = %bb649, %bb647
  store float %load59, ptr %__inline_ret51, align 4
  %load65 = load float, ptr %__inline_ret51, align 4
  %vararg_fpext66 = fpext float %load65 to double
  %call67 = call i32 (ptr, ...) @printf(ptr @global_str.11, double %vararg_fpext66)
  %call68 = call i32 (ptr, ...) @printf(ptr @global_str.11, double 0x3FF1980000000000)
  %call69 = call i32 (ptr, ...) @printf(ptr @global_str.11, double 0x3FF19999A0000000)
  %call70 = call i32 (ptr, ...) @printf(ptr @global_str.11, double 1.100000e+00)
  store i64 0, ptr %a, align 8
  store i64 1, ptr %a, align 8
  %call71 = call i32 (ptr, ...) @printf(ptr @global_str.11, double -6.550400e+04)
  store ptr null, ptr %str1, align 8
  store ptr @global_str.13, ptr %str1, align 8
  %load72 = load ptr, ptr %str1, align 8
  %ptr2int73 = ptrtoint ptr %load72 to i64
  %cmpne74 = icmp ne i64 %ptr2int73, 0
  %zext75 = zext i1 %cmpne74 to i64
  %cond_true76 = icmp ne i64 %zext75, 0
  br i1 %cond_true76, label %bb217, label %bb218

bb216:                                            ; preds = %bb644
  call void @panic(ptr @global_str.7)
  unreachable

bb217:                                            ; preds = %bb650
  store ptr %load72, ptr %self77, align 8
  %load78 = load ptr, ptr %self77, align 8
  %call79 = call i64 @strlen(ptr %load78)
  %call80 = call i32 (ptr, ...) @printf(ptr @global_str.14, i64 %call79)
  %load81 = load ptr, ptr %str1, align 8
  %ptr2int82 = ptrtoint ptr %load81 to i64
  %cmpne83 = icmp ne i64 %ptr2int82, 0
  %zext84 = zext i1 %cmpne83 to i64
  %cond_true85 = icmp ne i64 %zext84, 0
  br i1 %cond_true85, label %bb219, label %bb220

bb218:                                            ; preds = %bb650
  call void @panic(ptr @global_str.7)
  unreachable

bb219:                                            ; preds = %bb217
  store ptr %load81, ptr %self87, align 8
  %load88 = load ptr, ptr %self87, align 8
  %ptr2int89 = ptrtoint ptr %load88 to i64
  %cmpne90 = icmp ne i64 %ptr2int89, 0
  %zext91 = zext i1 %cmpne90 to i64
  %cond_true92 = icmp ne i64 %zext91, 0
  br i1 %cond_true92, label %bb653, label %bb654

bb653:                                            ; preds = %bb219
  %call93 = call i64 @__fishy_member_6_string_3_len(ptr %load88)
  %cmpeq = icmp eq i64 %call93, 0
  %zext94 = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext94 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %trunc = trunc i64 %cast_zext to i1
  store i1 %trunc, ptr %__inline_ret86, align 1
  %load95 = load i1, ptr %__inline_ret86, align 1
  %load_zext = zext i1 %load95 to i64
  %cast_trunc96 = trunc i64 %load_zext to i1
  %cast_zext97 = zext i1 %cast_trunc96 to i64
  %trunc100 = trunc i64 %cast_zext97 to i1
  store i1 %trunc100, ptr %self99, align 1
  %load101 = load i1, ptr %self99, align 1
  %load_zext102 = zext i1 %load101 to i64
  %cast_trunc103 = trunc i64 %load_zext102 to i1
  %cast_zext104 = zext i1 %cast_trunc103 to i64
  %cond_true105 = icmp ne i64 %cast_zext104, 0
  br i1 %cond_true105, label %bb657, label %bb658

bb654:                                            ; preds = %bb219
  call void @panic(ptr @global_str.7)
  unreachable

bb657:                                            ; preds = %bb653
  store ptr @global_str.16, ptr %__inline_ret98, align 8
  br label %bb659

bb658:                                            ; preds = %bb653
  store ptr @global_str.17, ptr %__inline_ret98, align 8
  br label %bb659

bb659:                                            ; preds = %bb658, %bb657
  %load106 = load ptr, ptr %__inline_ret98, align 8
  %call107 = call i32 (ptr, ...) @printf(ptr @global_str.15, ptr %load106)
  %load108 = load ptr, ptr %str1, align 8
  %ptr2int109 = ptrtoint ptr %load108 to i64
  %cmpne110 = icmp ne i64 %ptr2int109, 0
  %zext111 = zext i1 %cmpne110 to i64
  %cond_true112 = icmp ne i64 %zext111, 0
  br i1 %cond_true112, label %bb221, label %bb222

bb220:                                            ; preds = %bb217
  call void @panic(ptr @global_str.7)
  unreachable

bb221:                                            ; preds = %bb659
  %call113 = call i1 @__fishy_member_6_string_8_contains(ptr %load108, ptr @global_str.18)
  %cast_zext114 = zext i1 %call113 to i64
  %trunc117 = trunc i64 %cast_zext114 to i1
  store i1 %trunc117, ptr %self116, align 1
  %load118 = load i1, ptr %self116, align 1
  %load_zext119 = zext i1 %load118 to i64
  %cast_trunc120 = trunc i64 %load_zext119 to i1
  %cast_zext121 = zext i1 %cast_trunc120 to i64
  %cond_true122 = icmp ne i64 %cast_zext121, 0
  br i1 %cond_true122, label %bb661, label %bb662

bb661:                                            ; preds = %bb221
  store ptr @global_str.16, ptr %__inline_ret115, align 8
  br label %bb663

bb662:                                            ; preds = %bb221
  store ptr @global_str.17, ptr %__inline_ret115, align 8
  br label %bb663

bb663:                                            ; preds = %bb662, %bb661
  %load123 = load ptr, ptr %__inline_ret115, align 8
  %call124 = call i32 (ptr, ...) @printf(ptr @global_str.15, ptr %load123)
  %load125 = load ptr, ptr %str1, align 8
  %ptr2int126 = ptrtoint ptr %load125 to i64
  %cmpne127 = icmp ne i64 %ptr2int126, 0
  %zext128 = zext i1 %cmpne127 to i64
  %cond_true129 = icmp ne i64 %zext128, 0
  br i1 %cond_true129, label %bb223, label %bb224

bb222:                                            ; preds = %bb659
  call void @panic(ptr @global_str.7)
  unreachable

bb223:                                            ; preds = %bb663
  %call130 = call i1 @__fishy_member_6_string_11_starts_with(ptr %load125, ptr @global_str.19)
  %cast_zext131 = zext i1 %call130 to i64
  %trunc134 = trunc i64 %cast_zext131 to i1
  store i1 %trunc134, ptr %self133, align 1
  %load135 = load i1, ptr %self133, align 1
  %load_zext136 = zext i1 %load135 to i64
  %cast_trunc137 = trunc i64 %load_zext136 to i1
  %cast_zext138 = zext i1 %cast_trunc137 to i64
  %cond_true139 = icmp ne i64 %cast_zext138, 0
  br i1 %cond_true139, label %bb665, label %bb666

bb665:                                            ; preds = %bb223
  store ptr @global_str.16, ptr %__inline_ret132, align 8
  br label %bb667

bb666:                                            ; preds = %bb223
  store ptr @global_str.17, ptr %__inline_ret132, align 8
  br label %bb667

bb667:                                            ; preds = %bb666, %bb665
  %load140 = load ptr, ptr %__inline_ret132, align 8
  %call141 = call i32 (ptr, ...) @printf(ptr @global_str.15, ptr %load140)
  %load142 = load ptr, ptr %str1, align 8
  %ptr2int143 = ptrtoint ptr %load142 to i64
  %cmpne144 = icmp ne i64 %ptr2int143, 0
  %zext145 = zext i1 %cmpne144 to i64
  %cond_true146 = icmp ne i64 %zext145, 0
  br i1 %cond_true146, label %bb225, label %bb226

bb224:                                            ; preds = %bb663
  call void @panic(ptr @global_str.7)
  unreachable

bb225:                                            ; preds = %bb667
  %call147 = call i1 @__fishy_member_6_string_9_ends_with(ptr %load142, ptr @global_str.20)
  %cast_zext148 = zext i1 %call147 to i64
  %trunc151 = trunc i64 %cast_zext148 to i1
  store i1 %trunc151, ptr %self150, align 1
  %load152 = load i1, ptr %self150, align 1
  %load_zext153 = zext i1 %load152 to i64
  %cast_trunc154 = trunc i64 %load_zext153 to i1
  %cast_zext155 = zext i1 %cast_trunc154 to i64
  %cond_true156 = icmp ne i64 %cast_zext155, 0
  br i1 %cond_true156, label %bb669, label %bb670

bb669:                                            ; preds = %bb225
  store ptr @global_str.16, ptr %__inline_ret149, align 8
  br label %bb671

bb670:                                            ; preds = %bb225
  store ptr @global_str.17, ptr %__inline_ret149, align 8
  br label %bb671

bb671:                                            ; preds = %bb670, %bb669
  %load157 = load ptr, ptr %__inline_ret149, align 8
  %call158 = call i32 (ptr, ...) @printf(ptr @global_str.15, ptr %load157)
  %load159 = load ptr, ptr %str1, align 8
  %load160 = load ptr, ptr %str1, align 8
  %ptr2int161 = ptrtoint ptr %load160 to i64
  %cmpne162 = icmp ne i64 %ptr2int161, 0
  %zext163 = zext i1 %cmpne162 to i64
  %cond_true164 = icmp ne i64 %zext163, 0
  br i1 %cond_true164, label %bb227, label %bb228

bb226:                                            ; preds = %bb667
  call void @panic(ptr @global_str.7)
  unreachable

bb227:                                            ; preds = %bb671
  store ptr %load160, ptr %self165, align 8
  store ptr %load159, ptr %other, align 8
  %load166 = load ptr, ptr %self165, align 8
  %load167 = load ptr, ptr %other, align 8
  %call168 = call i32 @strcmp(ptr %load166, ptr %load167)
  %cast_sext = sext i32 %call168 to i64
  %cmpeq169 = icmp eq i64 %cast_sext, 0
  %zext170 = zext i1 %cmpeq169 to i64
  %cast_trunc171 = trunc i64 %zext170 to i1
  %cast_zext172 = zext i1 %cast_trunc171 to i64
  %cast_trunc173 = trunc i64 %cast_zext172 to i1
  %cast_zext174 = zext i1 %cast_trunc173 to i64
  %trunc177 = trunc i64 %cast_zext174 to i1
  store i1 %trunc177, ptr %self176, align 1
  %load178 = load i1, ptr %self176, align 1
  %load_zext179 = zext i1 %load178 to i64
  %cast_trunc180 = trunc i64 %load_zext179 to i1
  %cast_zext181 = zext i1 %cast_trunc180 to i64
  %cond_true182 = icmp ne i64 %cast_zext181, 0
  br i1 %cond_true182, label %bb673, label %bb674

bb673:                                            ; preds = %bb227
  store ptr @global_str.16, ptr %__inline_ret175, align 8
  br label %bb675

bb674:                                            ; preds = %bb227
  store ptr @global_str.17, ptr %__inline_ret175, align 8
  br label %bb675

bb675:                                            ; preds = %bb674, %bb673
  %load183 = load ptr, ptr %__inline_ret175, align 8
  %call184 = call i32 (ptr, ...) @printf(ptr @global_str.15, ptr %load183)
  %load185 = load ptr, ptr %str1, align 8
  %ptr2int186 = ptrtoint ptr %load185 to i64
  %cmpne187 = icmp ne i64 %ptr2int186, 0
  %zext188 = zext i1 %cmpne187 to i64
  %cond_true189 = icmp ne i64 %zext188, 0
  br i1 %cond_true189, label %bb229, label %bb230

bb228:                                            ; preds = %bb671
  call void @panic(ptr @global_str.7)
  unreachable

bb229:                                            ; preds = %bb675
  %call190 = call ptr @__fishy_member_6_string_9_substring(ptr %load185, i64 0, i64 3)
  %call191 = call i32 (ptr, ...) @printf(ptr @global_str.15, ptr %call190)
  store i8 0, ptr %n1, align 1
  store i8 -2, ptr %n1, align 1
  store i8 0, ptr %n2, align 1
  store i8 1, ptr %n2, align 1
  %load192 = load i8, ptr %n1, align 1
  %load_zext193 = zext i8 %load192 to i64
  %cast_trunc194 = trunc i64 %load_zext193 to i8
  %cast_zext195 = zext i8 %cast_trunc194 to i64
  %load196 = load i8, ptr %n2, align 1
  %load_zext197 = zext i8 %load196 to i64
  %cast_trunc198 = trunc i64 %load_zext197 to i8
  %cast_zext199 = zext i8 %cast_trunc198 to i64
  %add = add i64 %cast_zext195, %cast_zext199
  %cast_trunc200 = trunc i64 %add to i8
  %cast_zext201 = zext i8 %cast_trunc200 to i64
  %call202 = call i32 (ptr, ...) @printf(ptr @global_str.14, i64 %cast_zext201)
  %load203 = load ptr, ptr %str1, align 8
  %load204 = load ptr, ptr %str1, align 8
  %ptr2int205 = ptrtoint ptr %load204 to i64
  %cmpne206 = icmp ne i64 %ptr2int205, 0
  %zext207 = zext i1 %cmpne206 to i64
  %cond_true208 = icmp ne i64 %zext207, 0
  br i1 %cond_true208, label %bb231, label %bb232

bb230:                                            ; preds = %bb675
  call void @panic(ptr @global_str.7)
  unreachable

bb231:                                            ; preds = %bb229
  %call209 = call ptr @__fishy_member_6_string_6_concat(ptr %load204, ptr %load203)
  %call210 = call i32 (ptr, ...) @printf(ptr @global_str.15, ptr %call209)
  %call211 = call i32 (ptr, ...) @printf(ptr @global_str.21)
  store ptr null, ptr %stdin, align 8
  %call212 = call ptr @read_line(i64 128)
  store ptr %call212, ptr %stdin, align 8
  store ptr null, ptr %fruits, align 8
  %load213 = load ptr, ptr %stdin, align 8
  %ptr2int214 = ptrtoint ptr %load213 to i64
  %cmpne215 = icmp ne i64 %ptr2int214, 0
  %zext216 = zext i1 %cmpne215 to i64
  %cond_true217 = icmp ne i64 %zext216, 0
  br i1 %cond_true217, label %bb233, label %bb234

bb232:                                            ; preds = %bb229
  call void @panic(ptr @global_str.7)
  unreachable

bb233:                                            ; preds = %bb231
  %call218 = call ptr @__fishy_member_6_string_5_split(ptr %load213, ptr @global_str.22)
  store ptr %call218, ptr %fruits, align 8
  %call219 = call i32 (ptr, ...) @printf(ptr @global_str.23)
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  br label %bb235

bb234:                                            ; preds = %bb231
  call void @panic(ptr @global_str.7)
  unreachable

bb235:                                            ; preds = %bb240, %bb233
  %load220 = load i64, ptr %i, align 8
  %load221 = load ptr, ptr %fruits, align 8
  %ptr2int222 = ptrtoint ptr %load221 to i64
  %cmpne223 = icmp ne i64 %ptr2int222, 0
  %zext224 = zext i1 %cmpne223 to i64
  %cond_true225 = icmp ne i64 %zext224, 0
  br i1 %cond_true225, label %bb238, label %bb239

bb236:                                            ; preds = %bb680
  %load226 = load i64, ptr %i, align 8
  %load227 = load ptr, ptr %fruits, align 8
  %ptr2int228 = ptrtoint ptr %load227 to i64
  %cmpne229 = icmp ne i64 %ptr2int228, 0
  %zext230 = zext i1 %cmpne229 to i64
  %cond_true231 = icmp ne i64 %zext230, 0
  br i1 %cond_true231, label %bb240, label %bb241

bb237:                                            ; preds = %bb680
  %call232 = call i32 (ptr, ...) @printf(ptr @global_str.25, i64 -128)
  %load233 = load ptr, ptr %fruits, align 8
  %ptr2int234 = ptrtoint ptr %load233 to i64
  %cmpne235 = icmp ne i64 %ptr2int234, 0
  %zext236 = zext i1 %cmpne235 to i64
  %cond_true237 = icmp ne i64 %zext236, 0
  br i1 %cond_true237, label %bb242, label %bb243

bb238:                                            ; preds = %bb235
  store ptr %load221, ptr %self239, align 8
  call void @__fishy_retain(ptr %load221)
  %load240 = load ptr, ptr %self239, align 8
  %ptr2int241 = ptrtoint ptr %load240 to i64
  %cmpne242 = icmp ne i64 %ptr2int241, 0
  %zext243 = zext i1 %cmpne242 to i64
  %cond_true244 = icmp ne i64 %zext243, 0
  br i1 %cond_true244, label %bb677, label %bb678

bb677:                                            ; preds = %bb238
  %gep245 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load240, i64 0, i32 1
  %load246 = load i64, ptr %gep245, align 8
  %load247 = load ptr, ptr %self239, align 8
  %ptr2int248 = ptrtoint ptr %load247 to i64
  %cmpne249 = icmp ne i64 %ptr2int248, 0
  %zext250 = zext i1 %cmpne249 to i64
  %cond_true251 = icmp ne i64 %zext250, 0
  br i1 %cond_true251, label %bb679, label %bb680

bb678:                                            ; preds = %bb238
  call void @panic(ptr @global_str.12)
  unreachable

bb679:                                            ; preds = %bb677
  call void @__fishy_release(ptr %load247)
  br label %bb680

bb680:                                            ; preds = %bb679, %bb677
  store i64 %load246, ptr %__inline_ret238, align 8
  %load252 = load i64, ptr %__inline_ret238, align 8
  %cmplt = icmp ult i64 %load220, %load252
  %zext253 = zext i1 %cmplt to i64
  %cast_trunc254 = trunc i64 %zext253 to i1
  %cast_zext255 = zext i1 %cast_trunc254 to i64
  %cond_true256 = icmp ne i64 %cast_zext255, 0
  br i1 %cond_true256, label %bb236, label %bb237

bb239:                                            ; preds = %bb235
  call void @panic(ptr @global_str.7)
  unreachable

bb240:                                            ; preds = %bb236
  %call257 = call ptr @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_3_get(ptr %load227, i64 %load226)
  %call258 = call i32 (ptr, ...) @printf(ptr @global_str.24, ptr %call257)
  %load259 = load i64, ptr %i, align 8
  %add260 = add i64 %load259, 1
  store i64 %add260, ptr %i, align 8
  br label %bb235

bb241:                                            ; preds = %bb236
  call void @panic(ptr @global_str.7)
  unreachable

bb242:                                            ; preds = %bb237
  call void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_4_drop(ptr %load233)
  %load261 = load ptr, ptr %stdin, align 8
  call void @free(ptr %load261)
  store ptr null, ptr %stdin, align 8
  %load262 = load ptr, ptr %fruits, align 8
  %ptr2int263 = ptrtoint ptr %load262 to i64
  %cmpne264 = icmp ne i64 %ptr2int263, 0
  %zext265 = zext i1 %cmpne264 to i64
  %cond_true266 = icmp ne i64 %zext265, 0
  br i1 %cond_true266, label %bb244, label %bb245

bb243:                                            ; preds = %bb237
  call void @panic(ptr @global_str.7)
  unreachable

bb244:                                            ; preds = %bb242
  call void @__fishy_release(ptr %load262)
  br label %bb245

bb245:                                            ; preds = %bb244, %bb242
  %load267 = load ptr, ptr %point3, align 8
  %ptr2int268 = ptrtoint ptr %load267 to i64
  %cmpne269 = icmp ne i64 %ptr2int268, 0
  %zext270 = zext i1 %cmpne269 to i64
  %cond_true271 = icmp ne i64 %zext270, 0
  br i1 %cond_true271, label %bb246, label %bb247

bb246:                                            ; preds = %bb245
  call void @__fishy_release(ptr %load267)
  br label %bb247

bb247:                                            ; preds = %bb246, %bb245
  ret i64 0

allocation.ok:                                    ; preds = %bb208
  store ptr null, ptr %managed.alloc, align 8
  %rc = getelementptr i64, ptr %managed.alloc, i64 1
  store i64 1, ptr %rc, align 8
  %length = getelementptr i64, ptr %managed.alloc, i64 2
  store i64 0, ptr %length, align 8
  %data = getelementptr i64, ptr %managed.alloc, i64 3
  %gep = getelementptr %__fishy_generic_6_Point3_5b4633325d, ptr %data, i64 0, i32 0
  store float 0x40019999A0000000, ptr %gep, align 4
  %gep1 = getelementptr %__fishy_generic_6_Point3_5b4633325d, ptr %data, i64 0, i32 1
  store float 0x40119999A0000000, ptr %gep1, align 4
  %gep2 = getelementptr %__fishy_generic_6_Point3_5b4633325d, ptr %data, i64 0, i32 2
  store float 0x401A666660000000, ptr %gep2, align 4
  store ptr %data, ptr %point3, align 8
  %load = load ptr, ptr %point3, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb209, label %bb210

allocation.failed:                                ; preds = %bb208
  call void @panic(ptr @oom.message.10)
  unreachable
}

define void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_7_reserve(ptr %0, i64 %1) {
bb317:
  %tail = alloca ptr, align 8
  %old_bytes = alloca i64, align 8
  %size_in_bytes = alloca i64, align 8
  %element_size = alloca i64, align 8
  %new_cap = alloca i64, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  call void @__fishy_retain(ptr %0)
  store i64 %1, ptr %new_cap, align 8
  %load = load i64, ptr %new_cap, align 8
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb318, label %bb319

bb318:                                            ; preds = %bb317
  %gep = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load1, i64 0, i32 2
  %load2 = load i64, ptr %gep, align 8
  %cmple = icmp ule i64 %load, %load2
  %zext3 = zext i1 %cmple to i64
  %cast_trunc = trunc i64 %zext3 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true4 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true4, label %bb320, label %bb321

bb319:                                            ; preds = %bb317
  call void @panic(ptr @global_str.12)
  unreachable

bb320:                                            ; preds = %bb318
  %load5 = load ptr, ptr %self, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb322, label %bb323

bb321:                                            ; preds = %bb318
  store i64 0, ptr %element_size, align 8
  store i64 8, ptr %element_size, align 8
  %load10 = load i64, ptr %new_cap, align 8
  %load11 = load i64, ptr %element_size, align 8
  %cmpne12 = icmp ne i64 %load11, 0
  %zext13 = zext i1 %cmpne12 to i64
  %cond_true14 = icmp ne i64 %zext13, 0
  br i1 %cond_true14, label %bb324, label %bb325

bb322:                                            ; preds = %bb320
  call void @__fishy_release(ptr %load5)
  br label %bb323

bb323:                                            ; preds = %bb322, %bb320
  ret void

bb324:                                            ; preds = %bb321
  %udiv = udiv i64 -1, %load11
  %cmpgt = icmp ugt i64 %load10, %udiv
  %zext15 = zext i1 %cmpgt to i64
  %cast_trunc16 = trunc i64 %zext15 to i1
  %cast_zext17 = zext i1 %cast_trunc16 to i64
  %cond_true18 = icmp ne i64 %cast_zext17, 0
  br i1 %cond_true18, label %bb326, label %bb327

bb325:                                            ; preds = %bb321
  call void @panic(ptr @global_str.26)
  unreachable

bb326:                                            ; preds = %bb324
  call void @panic(ptr @global_str.27)
  unreachable

bb327:                                            ; preds = %bb324
  store i64 0, ptr %size_in_bytes, align 8
  %load19 = load i64, ptr %new_cap, align 8
  %load20 = load i64, ptr %element_size, align 8
  %mul = mul i64 %load19, %load20
  store i64 %mul, ptr %size_in_bytes, align 8
  store i64 0, ptr %old_bytes, align 8
  %load21 = load ptr, ptr %self, align 8
  %ptr2int22 = ptrtoint ptr %load21 to i64
  %cmpne23 = icmp ne i64 %ptr2int22, 0
  %zext24 = zext i1 %cmpne23 to i64
  %cond_true25 = icmp ne i64 %zext24, 0
  br i1 %cond_true25, label %bb328, label %bb329

bb328:                                            ; preds = %bb327
  %gep26 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load21, i64 0, i32 2
  %load27 = load i64, ptr %gep26, align 8
  %load28 = load i64, ptr %element_size, align 8
  %mul29 = mul i64 %load27, %load28
  store i64 %mul29, ptr %old_bytes, align 8
  %load30 = load ptr, ptr %self, align 8
  %ptr2int31 = ptrtoint ptr %load30 to i64
  %cmpne32 = icmp ne i64 %ptr2int31, 0
  %zext33 = zext i1 %cmpne32 to i64
  %cond_true34 = icmp ne i64 %zext33, 0
  br i1 %cond_true34, label %bb330, label %bb331

bb329:                                            ; preds = %bb327
  call void @panic(ptr @global_str.12)
  unreachable

bb330:                                            ; preds = %bb328
  %gep35 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load30, i64 0, i32 2
  %load36 = load i64, ptr %gep35, align 8
  %cmpeq = icmp eq i64 %load36, 0
  %zext37 = zext i1 %cmpeq to i64
  %cast_trunc38 = trunc i64 %zext37 to i1
  %cast_zext39 = zext i1 %cast_trunc38 to i64
  %cond_true40 = icmp ne i64 %cast_zext39, 0
  br i1 %cond_true40, label %bb332, label %bb334

bb331:                                            ; preds = %bb328
  call void @panic(ptr @global_str.12)
  unreachable

bb332:                                            ; preds = %bb330
  %load41 = load ptr, ptr %self, align 8
  %ptr2int42 = ptrtoint ptr %load41 to i64
  %cmpne43 = icmp ne i64 %ptr2int42, 0
  %zext44 = zext i1 %cmpne43 to i64
  %cond_true45 = icmp ne i64 %zext44, 0
  br i1 %cond_true45, label %bb335, label %bb336

bb333:                                            ; preds = %bb339, %bb335
  %load46 = load ptr, ptr %self, align 8
  %ptr2int47 = ptrtoint ptr %load46 to i64
  %cmpne48 = icmp ne i64 %ptr2int47, 0
  %zext49 = zext i1 %cmpne48 to i64
  %cond_true50 = icmp ne i64 %zext49, 0
  br i1 %cond_true50, label %bb341, label %bb342

bb334:                                            ; preds = %bb330
  %load51 = load ptr, ptr %self, align 8
  %ptr2int52 = ptrtoint ptr %load51 to i64
  %cmpne53 = icmp ne i64 %ptr2int52, 0
  %zext54 = zext i1 %cmpne53 to i64
  %cond_true55 = icmp ne i64 %zext54, 0
  br i1 %cond_true55, label %bb337, label %bb338

bb335:                                            ; preds = %bb332
  %load56 = load i64, ptr %size_in_bytes, align 8
  %call = call ptr @malloc(i64 %load56)
  %gep57 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load41, i64 0, i32 0
  store ptr %call, ptr %gep57, align 8
  br label %bb333

bb336:                                            ; preds = %bb332
  call void @panic(ptr @global_str.28)
  unreachable

bb337:                                            ; preds = %bb334
  %load58 = load ptr, ptr %self, align 8
  %ptr2int59 = ptrtoint ptr %load58 to i64
  %cmpne60 = icmp ne i64 %ptr2int59, 0
  %zext61 = zext i1 %cmpne60 to i64
  %cond_true62 = icmp ne i64 %zext61, 0
  br i1 %cond_true62, label %bb339, label %bb340

bb338:                                            ; preds = %bb334
  call void @panic(ptr @global_str.28)
  unreachable

bb339:                                            ; preds = %bb337
  %gep63 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load58, i64 0, i32 0
  %load64 = load ptr, ptr %gep63, align 8
  %load65 = load i64, ptr %size_in_bytes, align 8
  %call66 = call ptr @realloc(ptr %load64, i64 %load65)
  %gep67 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load51, i64 0, i32 0
  store ptr %call66, ptr %gep67, align 8
  br label %bb333

bb340:                                            ; preds = %bb337
  call void @panic(ptr @global_str.12)
  unreachable

bb341:                                            ; preds = %bb333
  %gep68 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load46, i64 0, i32 0
  %load69 = load ptr, ptr %gep68, align 8
  %ptr2int70 = ptrtoint ptr %load69 to i64
  %cmpeq71 = icmp eq i64 %ptr2int70, 0
  %zext72 = zext i1 %cmpeq71 to i64
  %cast_trunc73 = trunc i64 %zext72 to i1
  %cast_zext74 = zext i1 %cast_trunc73 to i64
  %cond_true75 = icmp ne i64 %cast_zext74, 0
  br i1 %cond_true75, label %bb343, label %bb344

bb342:                                            ; preds = %bb333
  call void @panic(ptr @global_str.12)
  unreachable

bb343:                                            ; preds = %bb341
  call void @panic(ptr @global_str.29)
  unreachable

bb344:                                            ; preds = %bb341
  store ptr null, ptr %tail, align 8
  %load76 = load ptr, ptr %self, align 8
  %ptr2int77 = ptrtoint ptr %load76 to i64
  %cmpne78 = icmp ne i64 %ptr2int77, 0
  %zext79 = zext i1 %cmpne78 to i64
  %cond_true80 = icmp ne i64 %zext79, 0
  br i1 %cond_true80, label %bb345, label %bb346

bb345:                                            ; preds = %bb344
  %gep81 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load76, i64 0, i32 0
  %load82 = load ptr, ptr %gep81, align 8
  %ptrtoint = ptrtoint ptr %load82 to i64
  %load83 = load i64, ptr %old_bytes, align 8
  %add = add i64 %ptrtoint, %load83
  %inttoptr = inttoptr i64 %add to ptr
  store ptr %inttoptr, ptr %tail, align 8
  %load84 = load ptr, ptr %tail, align 8
  %load85 = load i64, ptr %size_in_bytes, align 8
  %load86 = load i64, ptr %old_bytes, align 8
  %sub = sub i64 %load85, %load86
  %call87 = call ptr @memset(ptr %load84, i32 0, i64 %sub)
  %load88 = load ptr, ptr %self, align 8
  %ptr2int89 = ptrtoint ptr %load88 to i64
  %cmpne90 = icmp ne i64 %ptr2int89, 0
  %zext91 = zext i1 %cmpne90 to i64
  %cond_true92 = icmp ne i64 %zext91, 0
  br i1 %cond_true92, label %bb347, label %bb348

bb346:                                            ; preds = %bb344
  call void @panic(ptr @global_str.12)
  unreachable

bb347:                                            ; preds = %bb345
  %load93 = load i64, ptr %new_cap, align 8
  %gep94 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load88, i64 0, i32 2
  store i64 %load93, ptr %gep94, align 8
  %load95 = load ptr, ptr %self, align 8
  %ptr2int96 = ptrtoint ptr %load95 to i64
  %cmpne97 = icmp ne i64 %ptr2int96, 0
  %zext98 = zext i1 %cmpne97 to i64
  %cond_true99 = icmp ne i64 %zext98, 0
  br i1 %cond_true99, label %bb349, label %bb350

bb348:                                            ; preds = %bb345
  call void @panic(ptr @global_str.28)
  unreachable

bb349:                                            ; preds = %bb347
  call void @__fishy_release(ptr %load95)
  br label %bb350

bb350:                                            ; preds = %bb349, %bb347
  ret void
}

define void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_4_drop(ptr %0) {
bb351:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  call void @__fishy_retain(ptr %0)
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb352, label %bb353

bb352:                                            ; preds = %bb351
  %gep = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load, i64 0, i32 2
  %load1 = load i64, ptr %gep, align 8
  %cmpgt = icmp ugt i64 %load1, 0
  %zext2 = zext i1 %cmpgt to i64
  %cast_trunc = trunc i64 %zext2 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true3 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true3, label %bb354, label %bb355

bb353:                                            ; preds = %bb351
  call void @panic(ptr @global_str.12)
  unreachable

bb354:                                            ; preds = %bb352
  %load4 = load ptr, ptr %self, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb356, label %bb357

bb355:                                            ; preds = %bb364, %bb352
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb366, label %bb367

bb356:                                            ; preds = %bb354
  call void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_5_clear(ptr %load4)
  %load14 = load ptr, ptr %self, align 8
  %ptr2int15 = ptrtoint ptr %load14 to i64
  %cmpne16 = icmp ne i64 %ptr2int15, 0
  %zext17 = zext i1 %cmpne16 to i64
  %cond_true18 = icmp ne i64 %zext17, 0
  br i1 %cond_true18, label %bb358, label %bb359

bb357:                                            ; preds = %bb354
  call void @panic(ptr @global_str.7)
  unreachable

bb358:                                            ; preds = %bb356
  %gep19 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load14, i64 0, i32 0
  %load20 = load ptr, ptr %gep19, align 8
  call void @free(ptr %load20)
  %load21 = load ptr, ptr %self, align 8
  %ptr2int22 = ptrtoint ptr %load21 to i64
  %cmpne23 = icmp ne i64 %ptr2int22, 0
  %zext24 = zext i1 %cmpne23 to i64
  %cond_true25 = icmp ne i64 %zext24, 0
  br i1 %cond_true25, label %bb360, label %bb361

bb359:                                            ; preds = %bb356
  call void @panic(ptr @global_str.12)
  unreachable

bb360:                                            ; preds = %bb358
  %gep26 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load21, i64 0, i32 0
  store ptr null, ptr %gep26, align 8
  %load27 = load ptr, ptr %self, align 8
  %ptr2int28 = ptrtoint ptr %load27 to i64
  %cmpne29 = icmp ne i64 %ptr2int28, 0
  %zext30 = zext i1 %cmpne29 to i64
  %cond_true31 = icmp ne i64 %zext30, 0
  br i1 %cond_true31, label %bb362, label %bb363

bb361:                                            ; preds = %bb358
  call void @panic(ptr @global_str.30)
  unreachable

bb362:                                            ; preds = %bb360
  %gep32 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load27, i64 0, i32 2
  store i64 0, ptr %gep32, align 8
  %load33 = load ptr, ptr %self, align 8
  %ptr2int34 = ptrtoint ptr %load33 to i64
  %cmpne35 = icmp ne i64 %ptr2int34, 0
  %zext36 = zext i1 %cmpne35 to i64
  %cond_true37 = icmp ne i64 %zext36, 0
  br i1 %cond_true37, label %bb364, label %bb365

bb363:                                            ; preds = %bb360
  call void @panic(ptr @global_str.28)
  unreachable

bb364:                                            ; preds = %bb362
  %gep38 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load33, i64 0, i32 1
  store i64 0, ptr %gep38, align 8
  br label %bb355

bb365:                                            ; preds = %bb362
  call void @panic(ptr @global_str.28)
  unreachable

bb366:                                            ; preds = %bb355
  call void @__fishy_release(ptr %load9)
  br label %bb367

bb367:                                            ; preds = %bb366, %bb355
  ret void
}

define void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_4_push(ptr %0, ptr %1) {
bb368:
  %next_cap = alloca i64, align 8
  %item = alloca ptr, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  call void @__fishy_retain(ptr %0)
  store ptr %1, ptr %item, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb369, label %bb370

bb369:                                            ; preds = %bb368
  %gep = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 8
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb371, label %bb372

bb370:                                            ; preds = %bb368
  call void @panic(ptr @global_str.12)
  unreachable

bb371:                                            ; preds = %bb369
  %gep7 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load2, i64 0, i32 2
  %load8 = load i64, ptr %gep7, align 8
  %cmpeq = icmp eq i64 %load1, %load8
  %zext9 = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext9 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true10 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true10, label %bb373, label %bb374

bb372:                                            ; preds = %bb369
  call void @panic(ptr @global_str.12)
  unreachable

bb373:                                            ; preds = %bb371
  store i64 0, ptr %next_cap, align 8
  store i64 4, ptr %next_cap, align 8
  %load11 = load ptr, ptr %self, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb375, label %bb376

bb374:                                            ; preds = %bb381, %bb371
  %load16 = load ptr, ptr %self, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpne18 = icmp ne i64 %ptr2int17, 0
  %zext19 = zext i1 %cmpne18 to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb383, label %bb384

bb375:                                            ; preds = %bb373
  %gep21 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load11, i64 0, i32 2
  %load22 = load i64, ptr %gep21, align 8
  %cmpgt = icmp ugt i64 %load22, 0
  %zext23 = zext i1 %cmpgt to i64
  %cast_trunc24 = trunc i64 %zext23 to i1
  %cast_zext25 = zext i1 %cast_trunc24 to i64
  %cond_true26 = icmp ne i64 %cast_zext25, 0
  br i1 %cond_true26, label %bb377, label %bb378

bb376:                                            ; preds = %bb373
  call void @panic(ptr @global_str.12)
  unreachable

bb377:                                            ; preds = %bb375
  %load27 = load ptr, ptr %self, align 8
  %ptr2int28 = ptrtoint ptr %load27 to i64
  %cmpne29 = icmp ne i64 %ptr2int28, 0
  %zext30 = zext i1 %cmpne29 to i64
  %cond_true31 = icmp ne i64 %zext30, 0
  br i1 %cond_true31, label %bb379, label %bb380

bb378:                                            ; preds = %bb379, %bb375
  %load32 = load i64, ptr %next_cap, align 8
  %load33 = load ptr, ptr %self, align 8
  %ptr2int34 = ptrtoint ptr %load33 to i64
  %cmpne35 = icmp ne i64 %ptr2int34, 0
  %zext36 = zext i1 %cmpne35 to i64
  %cond_true37 = icmp ne i64 %zext36, 0
  br i1 %cond_true37, label %bb381, label %bb382

bb379:                                            ; preds = %bb377
  %gep38 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load27, i64 0, i32 2
  %load39 = load i64, ptr %gep38, align 8
  %mul = mul i64 %load39, 2
  store i64 %mul, ptr %next_cap, align 8
  br label %bb378

bb380:                                            ; preds = %bb377
  call void @panic(ptr @global_str.12)
  unreachable

bb381:                                            ; preds = %bb378
  call void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_7_reserve(ptr %load33, i64 %load32)
  br label %bb374

bb382:                                            ; preds = %bb378
  call void @panic(ptr @global_str.7)
  unreachable

bb383:                                            ; preds = %bb374
  %gep40 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load16, i64 0, i32 0
  %load41 = load ptr, ptr %gep40, align 8
  %ptr2int42 = ptrtoint ptr %load41 to i64
  %cmpne43 = icmp ne i64 %ptr2int42, 0
  %zext44 = zext i1 %cmpne43 to i64
  %cond_true45 = icmp ne i64 %zext44, 0
  br i1 %cond_true45, label %bb385, label %bb386

bb384:                                            ; preds = %bb374
  call void @panic(ptr @global_str.12)
  unreachable

bb385:                                            ; preds = %bb383
  %load46 = load ptr, ptr %self, align 8
  %ptr2int47 = ptrtoint ptr %load46 to i64
  %cmpne48 = icmp ne i64 %ptr2int47, 0
  %zext49 = zext i1 %cmpne48 to i64
  %cond_true50 = icmp ne i64 %zext49, 0
  br i1 %cond_true50, label %bb387, label %bb388

bb386:                                            ; preds = %bb383
  call void @panic(ptr @global_str.5)
  unreachable

bb387:                                            ; preds = %bb385
  %gep51 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load46, i64 0, i32 1
  %load52 = load i64, ptr %gep51, align 8
  %load53 = load ptr, ptr %item, align 8
  %gep54 = getelementptr ptr, ptr %load41, i64 %load52
  store ptr %load53, ptr %gep54, align 8
  %load55 = load ptr, ptr %self, align 8
  %ptr2int56 = ptrtoint ptr %load55 to i64
  %cmpne57 = icmp ne i64 %ptr2int56, 0
  %zext58 = zext i1 %cmpne57 to i64
  %cond_true59 = icmp ne i64 %zext58, 0
  br i1 %cond_true59, label %bb389, label %bb390

bb388:                                            ; preds = %bb385
  call void @panic(ptr @global_str.12)
  unreachable

bb389:                                            ; preds = %bb387
  %load60 = load ptr, ptr %self, align 8
  %ptr2int61 = ptrtoint ptr %load60 to i64
  %cmpne62 = icmp ne i64 %ptr2int61, 0
  %zext63 = zext i1 %cmpne62 to i64
  %cond_true64 = icmp ne i64 %zext63, 0
  br i1 %cond_true64, label %bb391, label %bb392

bb390:                                            ; preds = %bb387
  call void @panic(ptr @global_str.28)
  unreachable

bb391:                                            ; preds = %bb389
  %gep65 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load60, i64 0, i32 1
  %load66 = load i64, ptr %gep65, align 8
  %add = add i64 %load66, 1
  %gep67 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load55, i64 0, i32 1
  store i64 %add, ptr %gep67, align 8
  %load68 = load ptr, ptr %self, align 8
  %ptr2int69 = ptrtoint ptr %load68 to i64
  %cmpne70 = icmp ne i64 %ptr2int69, 0
  %zext71 = zext i1 %cmpne70 to i64
  %cond_true72 = icmp ne i64 %zext71, 0
  br i1 %cond_true72, label %bb393, label %bb394

bb392:                                            ; preds = %bb389
  call void @panic(ptr @global_str.12)
  unreachable

bb393:                                            ; preds = %bb391
  call void @__fishy_release(ptr %load68)
  br label %bb394

bb394:                                            ; preds = %bb393, %bb391
  ret void
}

define void @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_5_clear(ptr %0) {
bb531:
  %empty = alloca ptr, align 8
  %i = alloca i64, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  call void @__fishy_retain(ptr %0)
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  store ptr null, ptr %empty, align 8
  br label %bb532

bb532:                                            ; preds = %bb539, %bb531
  %load = load i64, ptr %i, align 8
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb535, label %bb536

bb533:                                            ; preds = %bb535
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb537, label %bb538

bb534:                                            ; preds = %bb535
  %load7 = load ptr, ptr %self, align 8
  %ptr2int8 = ptrtoint ptr %load7 to i64
  %cmpne9 = icmp ne i64 %ptr2int8, 0
  %zext10 = zext i1 %cmpne9 to i64
  %cond_true11 = icmp ne i64 %zext10, 0
  br i1 %cond_true11, label %bb541, label %bb542

bb535:                                            ; preds = %bb532
  %gep = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load1, i64 0, i32 1
  %load12 = load i64, ptr %gep, align 8
  %cmplt = icmp ult i64 %load, %load12
  %zext13 = zext i1 %cmplt to i64
  %cast_trunc = trunc i64 %zext13 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true14 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true14, label %bb533, label %bb534

bb536:                                            ; preds = %bb532
  call void @panic(ptr @global_str.12)
  unreachable

bb537:                                            ; preds = %bb533
  %gep15 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load2, i64 0, i32 0
  %load16 = load ptr, ptr %gep15, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpne18 = icmp ne i64 %ptr2int17, 0
  %zext19 = zext i1 %cmpne18 to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb539, label %bb540

bb538:                                            ; preds = %bb533
  call void @panic(ptr @global_str.12)
  unreachable

bb539:                                            ; preds = %bb537
  %load21 = load i64, ptr %i, align 8
  %load22 = load ptr, ptr %empty, align 8
  %gep23 = getelementptr ptr, ptr %load16, i64 %load21
  store ptr %load22, ptr %gep23, align 8
  %load24 = load i64, ptr %i, align 8
  %add = add i64 %load24, 1
  store i64 %add, ptr %i, align 8
  br label %bb532

bb540:                                            ; preds = %bb537
  call void @panic(ptr @global_str.5)
  unreachable

bb541:                                            ; preds = %bb534
  %gep25 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load7, i64 0, i32 1
  store i64 0, ptr %gep25, align 8
  %load26 = load ptr, ptr %self, align 8
  %ptr2int27 = ptrtoint ptr %load26 to i64
  %cmpne28 = icmp ne i64 %ptr2int27, 0
  %zext29 = zext i1 %cmpne28 to i64
  %cond_true30 = icmp ne i64 %zext29, 0
  br i1 %cond_true30, label %bb543, label %bb544

bb542:                                            ; preds = %bb534
  call void @panic(ptr @global_str.28)
  unreachable

bb543:                                            ; preds = %bb541
  call void @__fishy_release(ptr %load26)
  br label %bb544

bb544:                                            ; preds = %bb543, %bb541
  ret void
}

define ptr @__fishy_member_38___fishy_generic_3_Vec_5b537472696e675d_3_get(ptr %0, i64 %1) {
bb545:
  %index = alloca i64, align 8
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  call void @__fishy_retain(ptr %0)
  store i64 %1, ptr %index, align 8
  %load = load i64, ptr %index, align 8
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb546, label %bb547

bb546:                                            ; preds = %bb545
  %gep = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load1, i64 0, i32 1
  %load2 = load i64, ptr %gep, align 8
  %cmpge = icmp uge i64 %load, %load2
  %zext3 = zext i1 %cmpge to i64
  %cast_trunc = trunc i64 %zext3 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true4 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true4, label %bb548, label %bb549

bb547:                                            ; preds = %bb545
  call void @panic(ptr @global_str.12)
  unreachable

bb548:                                            ; preds = %bb546
  call void @panic(ptr @global_str.31)
  unreachable

bb549:                                            ; preds = %bb546
  %load5 = load ptr, ptr %self, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb550, label %bb551

bb550:                                            ; preds = %bb549
  %gep10 = getelementptr %__fishy_generic_3_Vec_5b537472696e675d, ptr %load5, i64 0, i32 0
  %load11 = load ptr, ptr %gep10, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb552, label %bb553

bb551:                                            ; preds = %bb549
  call void @panic(ptr @global_str.12)
  unreachable

bb552:                                            ; preds = %bb550
  %load16 = load i64, ptr %index, align 8
  %gep17 = getelementptr ptr, ptr %load11, i64 %load16
  %load18 = load ptr, ptr %gep17, align 8
  %load19 = load ptr, ptr %self, align 8
  %ptr2int20 = ptrtoint ptr %load19 to i64
  %cmpne21 = icmp ne i64 %ptr2int20, 0
  %zext22 = zext i1 %cmpne21 to i64
  %cond_true23 = icmp ne i64 %zext22, 0
  br i1 %cond_true23, label %bb554, label %bb555

bb553:                                            ; preds = %bb550
  call void @panic(ptr @global_str.6)
  unreachable

bb554:                                            ; preds = %bb552
  call void @__fishy_release(ptr %load19)
  br label %bb555

bb555:                                            ; preds = %bb554, %bb552
  ret ptr %load18
}

define float @__fishy_member_35___fishy_generic_6_Point3_5b4633325d_4_add3(ptr %0) {
bb609:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  call void @__fishy_retain(ptr %0)
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb610, label %bb611

bb610:                                            ; preds = %bb609
  %gep = getelementptr %__fishy_generic_6_Point3_5b4633325d, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb612, label %bb613

bb611:                                            ; preds = %bb609
  call void @panic(ptr @global_str.12)
  unreachable

bb612:                                            ; preds = %bb610
  %gep7 = getelementptr %__fishy_generic_6_Point3_5b4633325d, ptr %load2, i64 0, i32 1
  %load8 = load float, ptr %gep7, align 4
  %fadd = fadd float %load1, %load8
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb614, label %bb615

bb613:                                            ; preds = %bb610
  call void @panic(ptr @global_str.12)
  unreachable

bb614:                                            ; preds = %bb612
  %gep14 = getelementptr %__fishy_generic_6_Point3_5b4633325d, ptr %load9, i64 0, i32 2
  %load15 = load float, ptr %gep14, align 4
  %fadd16 = fadd float %fadd, %load15
  %load17 = load ptr, ptr %self, align 8
  %ptr2int18 = ptrtoint ptr %load17 to i64
  %cmpne19 = icmp ne i64 %ptr2int18, 0
  %zext20 = zext i1 %cmpne19 to i64
  %cond_true21 = icmp ne i64 %zext20, 0
  br i1 %cond_true21, label %bb616, label %bb617

bb615:                                            ; preds = %bb612
  call void @panic(ptr @global_str.12)
  unreachable

bb616:                                            ; preds = %bb614
  call void @__fishy_release(ptr %load17)
  br label %bb617

bb617:                                            ; preds = %bb616, %bb614
  ret float %fadd16
}

define void @__global_init() {
bb0:
  ret void
}

define internal void @__fishy_retain(ptr %0) {
entry:
  %nonnull = icmp ne ptr %0, null
  br i1 %nonnull, label %active, label %end

active:                                           ; preds = %entry
  %rc = getelementptr i64, ptr %0, i64 -2
  %count = load i64, ptr %rc, align 8
  %retained = add i64 %count, 1
  store i64 %retained, ptr %rc, align 8
  br label %end

end:                                              ; preds = %active, %entry
  ret void
}

define internal void @__fishy_release(ptr %0) {
entry:
  %nonnull = icmp ne ptr %0, null
  br i1 %nonnull, label %active, label %end

active:                                           ; preds = %entry
  %rc = getelementptr i64, ptr %0, i64 -2
  %count = load i64, ptr %rc, align 8
  %released = sub i64 %count, 1
  store i64 %released, ptr %rc, align 8
  %zero = icmp eq i64 %released, 0
  br i1 %zero, label %destroy, label %end

end:                                              ; preds = %free, %active, %entry
  ret void

destroy:                                          ; preds = %active
  %header = getelementptr i64, ptr %0, i64 -3
  %destructor = load ptr, ptr %header, align 8
  %has_destructor = icmp ne ptr %destructor, null
  br i1 %has_destructor, label %invoke, label %free

invoke:                                           ; preds = %destroy
  call void %destructor(ptr %0)
  br label %free

free:                                             ; preds = %invoke, %destroy
  call void @free(ptr %header)
  br label %end
}
