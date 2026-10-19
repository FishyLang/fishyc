; ModuleID = 'fishy_module'
source_filename = "fishy_module"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%Vec_str = type { ptr, i64, i64 }
%Point3_f32 = type { float, float, float }

@global_str = private unnamed_addr constant [30 x i8] c"\0A--- FISHY RUNTIME PANIC ---\0A\00", align 1
@global_str.1 = private unnamed_addr constant [11 x i8] c"FATAL: %s\0A\00", align 1
@global_str.2 = private unnamed_addr constant [56 x i8] c"PROGRAM HAS BEEN TERMINATED TO AVOID MEMORY CORRUPTION\0A\00", align 1
@global_str.3 = private unnamed_addr constant [40 x i8] c"Null pointer dereference (Array Write)!\00", align 1
@global_str.4 = private unnamed_addr constant [39 x i8] c"Null pointer dereference (Array Read)!\00", align 1
@global_str.5 = private unnamed_addr constant [41 x i8] c"Null pointer dereference on Method Call!\00", align 1
@global_str.6 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@global_str.7 = private unnamed_addr constant [43 x i8] c"Null pointer dereference on Property Read!\00", align 1
@global_str.8 = private unnamed_addr constant [5 x i8] c"John\00", align 1
@global_str.9 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@global_str.10 = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1
@global_str.11 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@global_str.12 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@global_str.13 = private unnamed_addr constant [2 x i8] c"h\00", align 1
@global_str.14 = private unnamed_addr constant [2 x i8] c"J\00", align 1
@global_str.15 = private unnamed_addr constant [2 x i8] c"n\00", align 1
@global_str.16 = private unnamed_addr constant [59 x i8] c"write down your favorite fruits separated by a dash (-): \0A\00", align 1
@global_str.17 = private unnamed_addr constant [2 x i8] c"-\00", align 1
@global_str.18 = private unnamed_addr constant [19 x i8] c"\0Ayour fruits are:\0A\00", align 1
@global_str.19 = private unnamed_addr constant [6 x i8] c"- %s\0A\00", align 1
@global_str.20 = private unnamed_addr constant [44 x i8] c"Null pointer dereference on Property Write!\00", align 1
@global_str.21 = private unnamed_addr constant [33 x i8] c"out of memory during Vec#reserve\00", align 1
@global_str.22 = private unnamed_addr constant [31 x i8] c"index out of bounds in Vec#get\00", align 1

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
  ret void
}

define ptr @read_line(i64 %0) {
bb6:
  %buf_size = alloca i64, align 8
  store i64 %0, ptr %buf_size, align 8
  %buffer = alloca ptr, align 8
  store ptr null, ptr %buffer, align 8
  %load = load i64, ptr %buf_size, align 8
  %call = call ptr @malloc(i64 %load)
  store ptr %call, ptr %buffer, align 8
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  %reading = alloca i1, align 1
  store i1 false, ptr %reading, align 1
  store i1 true, ptr %reading, align 1
  br label %bb7

bb7:                                              ; preds = %bb18, %bb15, %bb13, %bb10, %bb6
  %load1 = load i1, ptr %reading, align 1
  %load_zext = zext i1 %load1 to i64
  %cast_trunc = trunc i64 %load_zext to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true = icmp ne i64 %cast_zext, 0
  br i1 %cond_true, label %bb8, label %bb9

bb8:                                              ; preds = %bb7
  %c = alloca i32, align 4
  store i32 0, ptr %c, align 4
  %call2 = call i32 @getchar()
  %cast_sext = sext i32 %call2 to i64
  %trunc = trunc i64 %cast_sext to i32
  store i32 %trunc, ptr %c, align 4
  %load3 = load i32, ptr %c, align 4
  %load_sext = sext i32 %load3 to i64
  %cast_trunc4 = trunc i64 %load_sext to i32
  %cast_sext5 = sext i32 %cast_trunc4 to i64
  %cmpeq = icmp eq i64 %cast_sext5, -1
  %zext = zext i1 %cmpeq to i64
  %cast_trunc6 = trunc i64 %zext to i1
  %cast_zext7 = zext i1 %cast_trunc6 to i64
  %cond_true8 = icmp ne i64 %cast_zext7, 0
  br i1 %cond_true8, label %bb10, label %bb12

bb9:                                              ; preds = %bb7
  %load9 = load ptr, ptr %buffer, align 8
  %ptr2int = ptrtoint ptr %load9 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext10 = zext i1 %cmpne to i64
  %cond_true11 = icmp ne i64 %zext10, 0
  br i1 %cond_true11, label %bb20, label %bb21

bb10:                                             ; preds = %bb8
  store i1 false, ptr %reading, align 1
  br label %bb7

bb12:                                             ; preds = %bb8
  %load12 = load i32, ptr %c, align 4
  %load_sext13 = sext i32 %load12 to i64
  %cast_trunc14 = trunc i64 %load_sext13 to i32
  %cast_sext15 = sext i32 %cast_trunc14 to i64
  %cmpeq16 = icmp eq i64 %cast_sext15, 10
  %zext17 = zext i1 %cmpeq16 to i64
  %cast_trunc18 = trunc i64 %zext17 to i1
  %cast_zext19 = zext i1 %cast_trunc18 to i64
  %cond_true20 = icmp ne i64 %cast_zext19, 0
  br i1 %cond_true20, label %bb13, label %bb15

bb13:                                             ; preds = %bb12
  store i1 false, ptr %reading, align 1
  br label %bb7

bb15:                                             ; preds = %bb12
  %load21 = load i64, ptr %i, align 8
  %load22 = load i64, ptr %buf_size, align 8
  %sub = sub i64 %load22, 1
  %cmplt = icmp ult i64 %load21, %sub
  %zext23 = zext i1 %cmplt to i64
  %cast_trunc24 = trunc i64 %zext23 to i1
  %cast_zext25 = zext i1 %cast_trunc24 to i64
  %cond_true26 = icmp ne i64 %cast_zext25, 0
  br i1 %cond_true26, label %bb16, label %bb7

bb16:                                             ; preds = %bb15
  %load27 = load ptr, ptr %buffer, align 8
  %ptr2int28 = ptrtoint ptr %load27 to i64
  %cmpne29 = icmp ne i64 %ptr2int28, 0
  %zext30 = zext i1 %cmpne29 to i64
  %cond_true31 = icmp ne i64 %zext30, 0
  br i1 %cond_true31, label %bb18, label %bb19

bb18:                                             ; preds = %bb16
  %load32 = load i64, ptr %i, align 8
  %load33 = load i32, ptr %c, align 4
  %load_sext34 = sext i32 %load33 to i64
  %cast_trunc35 = trunc i64 %load_sext34 to i32
  %cast_sext36 = sext i32 %cast_trunc35 to i64
  %cast_trunc37 = trunc i64 %cast_sext36 to i8
  %cast_zext38 = zext i8 %cast_trunc37 to i64
  %gep = getelementptr i8, ptr %load27, i64 %load32
  %trunc39 = trunc i64 %cast_zext38 to i8
  store i8 %trunc39, ptr %gep, align 1
  %load40 = load i64, ptr %i, align 8
  %add = add i64 %load40, 1
  store i64 %add, ptr %i, align 8
  br label %bb7

bb19:                                             ; preds = %bb16
  call void @panic(ptr @global_str.3)
  unreachable

bb20:                                             ; preds = %bb9
  %load41 = load i64, ptr %i, align 8
  %gep42 = getelementptr i8, ptr %load9, i64 %load41
  store i8 0, ptr %gep42, align 1
  %final_str = alloca ptr, align 8
  store ptr null, ptr %final_str, align 8
  %load43 = load i64, ptr %i, align 8
  %add44 = add i64 %load43, 1
  %call45 = call ptr @malloc(i64 %add44)
  store ptr %call45, ptr %final_str, align 8
  %j = alloca i64, align 8
  store i64 0, ptr %j, align 8
  store i64 0, ptr %j, align 8
  br label %bb22

bb21:                                             ; preds = %bb9
  call void @panic(ptr @global_str.3)
  unreachable

bb22:                                             ; preds = %bb27, %bb20
  %load46 = load i64, ptr %j, align 8
  %load47 = load i64, ptr %i, align 8
  %cmplt48 = icmp ult i64 %load46, %load47
  %zext49 = zext i1 %cmplt48 to i64
  %cast_trunc50 = trunc i64 %zext49 to i1
  %cast_zext51 = zext i1 %cast_trunc50 to i64
  %cond_true52 = icmp ne i64 %cast_zext51, 0
  br i1 %cond_true52, label %bb23, label %bb24

bb23:                                             ; preds = %bb22
  %load53 = load ptr, ptr %final_str, align 8
  %ptr2int54 = ptrtoint ptr %load53 to i64
  %cmpne55 = icmp ne i64 %ptr2int54, 0
  %zext56 = zext i1 %cmpne55 to i64
  %cond_true57 = icmp ne i64 %zext56, 0
  br i1 %cond_true57, label %bb25, label %bb26

bb24:                                             ; preds = %bb22
  %load58 = load ptr, ptr %final_str, align 8
  %ptr2int59 = ptrtoint ptr %load58 to i64
  %cmpne60 = icmp ne i64 %ptr2int59, 0
  %zext61 = zext i1 %cmpne60 to i64
  %cond_true62 = icmp ne i64 %zext61, 0
  br i1 %cond_true62, label %bb29, label %bb30

bb25:                                             ; preds = %bb23
  %load63 = load i64, ptr %j, align 8
  %load64 = load ptr, ptr %buffer, align 8
  %ptr2int65 = ptrtoint ptr %load64 to i64
  %cmpne66 = icmp ne i64 %ptr2int65, 0
  %zext67 = zext i1 %cmpne66 to i64
  %cond_true68 = icmp ne i64 %zext67, 0
  br i1 %cond_true68, label %bb27, label %bb28

bb26:                                             ; preds = %bb23
  call void @panic(ptr @global_str.3)
  unreachable

bb27:                                             ; preds = %bb25
  %load69 = load i64, ptr %j, align 8
  %gep70 = getelementptr i8, ptr %load64, i64 %load69
  %load71 = load i8, ptr %gep70, align 1
  %load_zext72 = zext i8 %load71 to i64
  %cast_trunc73 = trunc i64 %load_zext72 to i8
  %cast_zext74 = zext i8 %cast_trunc73 to i64
  %gep75 = getelementptr i8, ptr %load53, i64 %load63
  %trunc76 = trunc i64 %cast_zext74 to i8
  store i8 %trunc76, ptr %gep75, align 1
  %load77 = load i64, ptr %j, align 8
  %add78 = add i64 %load77, 1
  store i64 %add78, ptr %j, align 8
  br label %bb22

bb28:                                             ; preds = %bb25
  call void @panic(ptr @global_str.4)
  unreachable

bb29:                                             ; preds = %bb24
  %load79 = load i64, ptr %i, align 8
  %gep80 = getelementptr i8, ptr %load58, i64 %load79
  store i8 0, ptr %gep80, align 1
  %load81 = load ptr, ptr %buffer, align 8
  call void @free(ptr %load81)
  store ptr null, ptr %buffer, align 8
  %load82 = load ptr, ptr %final_str, align 8
  ret ptr %load82

bb30:                                             ; preds = %bb24
  call void @panic(ptr @global_str.3)
  unreachable
}

define i64 @string_len(ptr %0) {
bb104:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %call = call i64 @strlen(ptr %load)
  ret i64 %call
}

define i1 @string_contains(ptr %0, ptr %1) {
bb108:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %sub = alloca ptr, align 8
  store ptr %1, ptr %sub, align 8
  %load = load ptr, ptr %sub, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb109, label %bb110

bb109:                                            ; preds = %bb108
  %self1 = alloca ptr, align 8
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  %cmpeq = icmp eq i64 %call, 0
  %zext3 = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext3 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true4 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true4, label %bb111, label %bb112

bb110:                                            ; preds = %bb108
  call void @panic(ptr @global_str.5)
  unreachable

bb111:                                            ; preds = %bb109
  ret i1 false

bb112:                                            ; preds = %bb109
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

define i1 @string_starts_with(ptr %0, ptr %1) {
bb113:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %prefix = alloca ptr, align 8
  store ptr %1, ptr %prefix, align 8
  %p_len = alloca i64, align 8
  store i64 0, ptr %p_len, align 8
  %load = load ptr, ptr %prefix, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb114, label %bb115

bb114:                                            ; preds = %bb113
  %self1 = alloca ptr, align 8
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  store i64 %call, ptr %p_len, align 8
  %load3 = load ptr, ptr %self, align 8
  %ptr2int4 = ptrtoint ptr %load3 to i64
  %cmpne5 = icmp ne i64 %ptr2int4, 0
  %zext6 = zext i1 %cmpne5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb116, label %bb117

bb115:                                            ; preds = %bb113
  call void @panic(ptr @global_str.5)
  unreachable

bb116:                                            ; preds = %bb114
  %self8 = alloca ptr, align 8
  store ptr %load3, ptr %self8, align 8
  %load9 = load ptr, ptr %self8, align 8
  %call10 = call i64 @strlen(ptr %load9)
  %load11 = load i64, ptr %p_len, align 8
  %cmplt = icmp ult i64 %call10, %load11
  %zext12 = zext i1 %cmplt to i64
  %cast_trunc = trunc i64 %zext12 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true13 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true13, label %bb118, label %bb119

bb117:                                            ; preds = %bb114
  call void @panic(ptr @global_str.5)
  unreachable

bb118:                                            ; preds = %bb116
  ret i1 false

bb119:                                            ; preds = %bb116
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  br label %bb120

bb120:                                            ; preds = %bb128, %bb119
  %load14 = load i64, ptr %i, align 8
  %load15 = load i64, ptr %p_len, align 8
  %cmplt16 = icmp ult i64 %load14, %load15
  %zext17 = zext i1 %cmplt16 to i64
  %cast_trunc18 = trunc i64 %zext17 to i1
  %cast_zext19 = zext i1 %cast_trunc18 to i64
  %cond_true20 = icmp ne i64 %cast_zext19, 0
  br i1 %cond_true20, label %bb121, label %bb122

bb121:                                            ; preds = %bb120
  %load21 = load ptr, ptr %self, align 8
  %ptr2int22 = ptrtoint ptr %load21 to i64
  %cmpne23 = icmp ne i64 %ptr2int22, 0
  %zext24 = zext i1 %cmpne23 to i64
  %cond_true25 = icmp ne i64 %zext24, 0
  br i1 %cond_true25, label %bb123, label %bb124

bb122:                                            ; preds = %bb120
  ret i1 true

bb123:                                            ; preds = %bb121
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
  br i1 %cond_true34, label %bb125, label %bb126

bb124:                                            ; preds = %bb121
  call void @panic(ptr @global_str.4)
  unreachable

bb125:                                            ; preds = %bb123
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
  br i1 %cond_true45, label %bb127, label %bb128

bb126:                                            ; preds = %bb123
  call void @panic(ptr @global_str.4)
  unreachable

bb127:                                            ; preds = %bb125
  ret i1 false

bb128:                                            ; preds = %bb125
  %load46 = load i64, ptr %i, align 8
  %add = add i64 %load46, 1
  store i64 %add, ptr %i, align 8
  br label %bb120
}

define i1 @string_ends_with(ptr %0, ptr %1) {
bb129:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %suffix = alloca ptr, align 8
  store ptr %1, ptr %suffix, align 8
  %s_len = alloca i64, align 8
  store i64 0, ptr %s_len, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb130, label %bb131

bb130:                                            ; preds = %bb129
  %self1 = alloca ptr, align 8
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  store i64 %call, ptr %s_len, align 8
  %sub_len = alloca i64, align 8
  store i64 0, ptr %sub_len, align 8
  %load3 = load ptr, ptr %suffix, align 8
  %ptr2int4 = ptrtoint ptr %load3 to i64
  %cmpne5 = icmp ne i64 %ptr2int4, 0
  %zext6 = zext i1 %cmpne5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb132, label %bb133

bb131:                                            ; preds = %bb129
  call void @panic(ptr @global_str.5)
  unreachable

bb132:                                            ; preds = %bb130
  %self8 = alloca ptr, align 8
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
  br i1 %cond_true14, label %bb134, label %bb135

bb133:                                            ; preds = %bb130
  call void @panic(ptr @global_str.5)
  unreachable

bb134:                                            ; preds = %bb132
  ret i1 false

bb135:                                            ; preds = %bb132
  %offset = alloca i64, align 8
  store i64 0, ptr %offset, align 8
  %load15 = load i64, ptr %s_len, align 8
  %load16 = load i64, ptr %sub_len, align 8
  %sub = sub i64 %load15, %load16
  store i64 %sub, ptr %offset, align 8
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  br label %bb136

bb136:                                            ; preds = %bb144, %bb135
  %load17 = load i64, ptr %i, align 8
  %load18 = load i64, ptr %sub_len, align 8
  %cmplt = icmp ult i64 %load17, %load18
  %zext19 = zext i1 %cmplt to i64
  %cast_trunc20 = trunc i64 %zext19 to i1
  %cast_zext21 = zext i1 %cast_trunc20 to i64
  %cond_true22 = icmp ne i64 %cast_zext21, 0
  br i1 %cond_true22, label %bb137, label %bb138

bb137:                                            ; preds = %bb136
  %load23 = load ptr, ptr %self, align 8
  %ptr2int24 = ptrtoint ptr %load23 to i64
  %cmpne25 = icmp ne i64 %ptr2int24, 0
  %zext26 = zext i1 %cmpne25 to i64
  %cond_true27 = icmp ne i64 %zext26, 0
  br i1 %cond_true27, label %bb139, label %bb140

bb138:                                            ; preds = %bb136
  ret i1 true

bb139:                                            ; preds = %bb137
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
  br i1 %cond_true37, label %bb141, label %bb142

bb140:                                            ; preds = %bb137
  call void @panic(ptr @global_str.4)
  unreachable

bb141:                                            ; preds = %bb139
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
  br i1 %cond_true48, label %bb143, label %bb144

bb142:                                            ; preds = %bb139
  call void @panic(ptr @global_str.4)
  unreachable

bb143:                                            ; preds = %bb141
  ret i1 false

bb144:                                            ; preds = %bb141
  %load49 = load i64, ptr %i, align 8
  %add50 = add i64 %load49, 1
  store i64 %add50, ptr %i, align 8
  br label %bb136
}

define ptr @string_substring(ptr %0, i64 %1, i64 %2) {
bb146:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %start = alloca i64, align 8
  store i64 %1, ptr %start, align 8
  %end = alloca i64, align 8
  store i64 %2, ptr %end, align 8
  %s_len = alloca i64, align 8
  store i64 0, ptr %s_len, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb147, label %bb148

bb147:                                            ; preds = %bb146
  %self1 = alloca ptr, align 8
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  store i64 %call, ptr %s_len, align 8
  %safe_end = alloca i64, align 8
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
  br i1 %cond_true7, label %bb149, label %bb150

bb148:                                            ; preds = %bb146
  call void @panic(ptr @global_str.5)
  unreachable

bb149:                                            ; preds = %bb147
  %load8 = load i64, ptr %s_len, align 8
  store i64 %load8, ptr %safe_end, align 8
  br label %bb150

bb150:                                            ; preds = %bb149, %bb147
  %load9 = load i64, ptr %start, align 8
  %load10 = load i64, ptr %safe_end, align 8
  %cmpge = icmp uge i64 %load9, %load10
  %zext11 = zext i1 %cmpge to i64
  %cast_trunc12 = trunc i64 %zext11 to i1
  %cast_zext13 = zext i1 %cast_trunc12 to i64
  %cond_true14 = icmp ne i64 %cast_zext13, 0
  br i1 %cond_true14, label %bb151, label %bb152

bb151:                                            ; preds = %bb150
  %empty = alloca ptr, align 8
  store ptr null, ptr %empty, align 8
  %call15 = call ptr @malloc(i64 1)
  store ptr %call15, ptr %empty, align 8
  %load16 = load ptr, ptr %empty, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpne18 = icmp ne i64 %ptr2int17, 0
  %zext19 = zext i1 %cmpne18 to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb153, label %bb154

bb152:                                            ; preds = %bb150
  %len = alloca i64, align 8
  store i64 0, ptr %len, align 8
  %load21 = load i64, ptr %safe_end, align 8
  %load22 = load i64, ptr %start, align 8
  %sub = sub i64 %load21, %load22
  store i64 %sub, ptr %len, align 8
  %dest = alloca ptr, align 8
  store ptr null, ptr %dest, align 8
  %load23 = load i64, ptr %len, align 8
  %add = add i64 %load23, 1
  %call24 = call ptr @malloc(i64 %add)
  store ptr %call24, ptr %dest, align 8
  %src_offset = alloca ptr, align 8
  store ptr null, ptr %src_offset, align 8
  %load25 = load ptr, ptr %self, align 8
  %ptrtoint = ptrtoint ptr %load25 to i64
  %load26 = load i64, ptr %start, align 8
  %add27 = add i64 %ptrtoint, %load26
  %inttoptr = inttoptr i64 %add27 to ptr
  store ptr %inttoptr, ptr %src_offset, align 8
  %load28 = load ptr, ptr %dest, align 8
  %load29 = load ptr, ptr %src_offset, align 8
  %load30 = load i64, ptr %len, align 8
  %call31 = call ptr @strncpy(ptr %load28, ptr %load29, i64 %load30)
  %load32 = load ptr, ptr %dest, align 8
  %ptr2int33 = ptrtoint ptr %load32 to i64
  %cmpne34 = icmp ne i64 %ptr2int33, 0
  %zext35 = zext i1 %cmpne34 to i64
  %cond_true36 = icmp ne i64 %zext35, 0
  br i1 %cond_true36, label %bb155, label %bb156

bb153:                                            ; preds = %bb151
  %gep = getelementptr i8, ptr %load16, i64 0
  store i8 0, ptr %gep, align 1
  %load37 = load ptr, ptr %empty, align 8
  ret ptr %load37

bb154:                                            ; preds = %bb151
  call void @panic(ptr @global_str.3)
  unreachable

bb155:                                            ; preds = %bb152
  %load38 = load i64, ptr %len, align 8
  %gep39 = getelementptr i8, ptr %load32, i64 %load38
  store i8 0, ptr %gep39, align 1
  %load40 = load ptr, ptr %dest, align 8
  ret ptr %load40

bb156:                                            ; preds = %bb152
  call void @panic(ptr @global_str.3)
  unreachable
}

define ptr @string_split(ptr %0, ptr %1) {
bb157:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %delim = alloca ptr, align 8
  store ptr %1, ptr %delim, align 8
  %result = alloca ptr, align 8
  store ptr null, ptr %result, align 8
  %struct_alloc = call ptr @malloc(i64 40)
  store i64 1, ptr %struct_alloc, align 8
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 24, ptr %meta_field, align 8
  %data_ptr = getelementptr i64, ptr %struct_alloc, i64 2
  %call = call ptr @malloc(i64 128)
  %gep = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 0
  store ptr %call, ptr %gep, align 8
  %gep1 = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 1
  store i64 0, ptr %gep1, align 8
  %gep2 = getelementptr %Vec_str, ptr %data_ptr, i64 0, i32 2
  store i64 10, ptr %gep2, align 8
  store ptr %data_ptr, ptr %result, align 8
  %s_len = alloca i64, align 8
  store i64 0, ptr %s_len, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb158, label %bb159

bb158:                                            ; preds = %bb157
  %self3 = alloca ptr, align 8
  store ptr %load, ptr %self3, align 8
  %load4 = load ptr, ptr %self3, align 8
  %call5 = call i64 @strlen(ptr %load4)
  store i64 %call5, ptr %s_len, align 8
  %d_len = alloca i64, align 8
  store i64 0, ptr %d_len, align 8
  %load6 = load ptr, ptr %delim, align 8
  %ptr2int7 = ptrtoint ptr %load6 to i64
  %cmpne8 = icmp ne i64 %ptr2int7, 0
  %zext9 = zext i1 %cmpne8 to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb160, label %bb161

bb159:                                            ; preds = %bb157
  call void @panic(ptr @global_str.5)
  unreachable

bb160:                                            ; preds = %bb158
  %self11 = alloca ptr, align 8
  store ptr %load6, ptr %self11, align 8
  %load12 = load ptr, ptr %self11, align 8
  %call13 = call i64 @strlen(ptr %load12)
  store i64 %call13, ptr %d_len, align 8
  %load14 = load i64, ptr %d_len, align 8
  %cmpeq = icmp eq i64 %load14, 0
  %zext15 = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext15 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true16 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true16, label %bb162, label %bb163

bb161:                                            ; preds = %bb158
  call void @panic(ptr @global_str.5)
  unreachable

bb162:                                            ; preds = %bb160
  %load17 = load ptr, ptr %self, align 8
  %load18 = load ptr, ptr %result, align 8
  %ptr2int19 = ptrtoint ptr %load18 to i64
  %cmpne20 = icmp ne i64 %ptr2int19, 0
  %zext21 = zext i1 %cmpne20 to i64
  %cond_true22 = icmp ne i64 %zext21, 0
  br i1 %cond_true22, label %bb164, label %bb165

bb163:                                            ; preds = %bb160
  %start = alloca i64, align 8
  store i64 0, ptr %start, align 8
  store i64 0, ptr %start, align 8
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  br label %bb168

bb164:                                            ; preds = %bb162
  call void @Vec_str_push(ptr %load18, ptr %load17)
  %load23 = load ptr, ptr %result, align 8
  %is_not_null = icmp ne ptr %load23, null
  br i1 %is_not_null, label %arc.retain.do, label %arc.retain.cont

bb165:                                            ; preds = %bb162
  call void @panic(ptr @global_str.5)
  unreachable

bb166:                                            ; preds = %arc.retain.cont
  %is_not_null29 = icmp ne ptr %load24, null
  br i1 %is_not_null29, label %arc.release.do, label %arc.release.cont

bb167:                                            ; preds = %arc.release.cont, %arc.retain.cont
  ret ptr %load23

bb168:                                            ; preds = %bb185, %bb182, %bb163
  %load33 = load i64, ptr %i, align 8
  %load34 = load i64, ptr %d_len, align 8
  %add = add i64 %load33, %load34
  %load35 = load i64, ptr %s_len, align 8
  %cmple = icmp ule i64 %add, %load35
  %zext36 = zext i1 %cmple to i64
  %cast_trunc37 = trunc i64 %zext36 to i1
  %cast_zext38 = zext i1 %cast_trunc37 to i64
  %cond_true39 = icmp ne i64 %cast_zext38, 0
  br i1 %cond_true39, label %bb169, label %bb170

bb169:                                            ; preds = %bb168
  %match_found = alloca i1, align 1
  store i1 false, ptr %match_found, align 1
  store i1 true, ptr %match_found, align 1
  %j = alloca i64, align 8
  store i64 0, ptr %j, align 8
  store i64 0, ptr %j, align 8
  br label %bb171

bb170:                                            ; preds = %bb168
  %load40 = load i64, ptr %start, align 8
  %load41 = load i64, ptr %s_len, align 8
  %load42 = load ptr, ptr %self, align 8
  %ptr2int43 = ptrtoint ptr %load42 to i64
  %cmpne44 = icmp ne i64 %ptr2int43, 0
  %zext45 = zext i1 %cmpne44 to i64
  %cond_true46 = icmp ne i64 %zext45, 0
  br i1 %cond_true46, label %bb187, label %bb188

bb171:                                            ; preds = %bb179, %bb169
  %load47 = load i64, ptr %j, align 8
  %load48 = load i64, ptr %d_len, align 8
  %cmplt = icmp ult i64 %load47, %load48
  %zext49 = zext i1 %cmplt to i64
  %cast_trunc50 = trunc i64 %zext49 to i1
  %cast_zext51 = zext i1 %cast_trunc50 to i64
  %cond_true52 = icmp ne i64 %cast_zext51, 0
  br i1 %cond_true52, label %bb172, label %bb173

bb172:                                            ; preds = %bb171
  %load53 = load ptr, ptr %self, align 8
  %ptr2int54 = ptrtoint ptr %load53 to i64
  %cmpne55 = icmp ne i64 %ptr2int54, 0
  %zext56 = zext i1 %cmpne55 to i64
  %cond_true57 = icmp ne i64 %zext56, 0
  br i1 %cond_true57, label %bb174, label %bb175

bb173:                                            ; preds = %bb171
  %load58 = load i1, ptr %match_found, align 1
  %load_zext = zext i1 %load58 to i64
  %cast_trunc59 = trunc i64 %load_zext to i1
  %cast_zext60 = zext i1 %cast_trunc59 to i64
  %cond_true61 = icmp ne i64 %cast_zext60, 0
  br i1 %cond_true61, label %bb180, label %bb182

bb174:                                            ; preds = %bb172
  %load62 = load i64, ptr %i, align 8
  %load63 = load i64, ptr %j, align 8
  %add64 = add i64 %load62, %load63
  %gep65 = getelementptr i8, ptr %load53, i64 %add64
  %load66 = load i8, ptr %gep65, align 1
  %load_sext = sext i8 %load66 to i64
  %cast_trunc67 = trunc i64 %load_sext to i8
  %cast_zext68 = zext i8 %cast_trunc67 to i64
  %load69 = load ptr, ptr %delim, align 8
  %ptr2int70 = ptrtoint ptr %load69 to i64
  %cmpne71 = icmp ne i64 %ptr2int70, 0
  %zext72 = zext i1 %cmpne71 to i64
  %cond_true73 = icmp ne i64 %zext72, 0
  br i1 %cond_true73, label %bb176, label %bb177

bb175:                                            ; preds = %bb172
  call void @panic(ptr @global_str.4)
  unreachable

bb176:                                            ; preds = %bb174
  %load74 = load i64, ptr %j, align 8
  %gep75 = getelementptr i8, ptr %load69, i64 %load74
  %load76 = load i8, ptr %gep75, align 1
  %load_sext77 = sext i8 %load76 to i64
  %cast_trunc78 = trunc i64 %load_sext77 to i8
  %cast_zext79 = zext i8 %cast_trunc78 to i64
  %cmpne80 = icmp ne i64 %cast_zext68, %cast_zext79
  %zext81 = zext i1 %cmpne80 to i64
  %cast_trunc82 = trunc i64 %zext81 to i1
  %cast_zext83 = zext i1 %cast_trunc82 to i64
  %cond_true84 = icmp ne i64 %cast_zext83, 0
  br i1 %cond_true84, label %bb178, label %bb179

bb177:                                            ; preds = %bb174
  call void @panic(ptr @global_str.4)
  unreachable

bb178:                                            ; preds = %bb176
  store i1 false, ptr %match_found, align 1
  br label %bb179

bb179:                                            ; preds = %bb178, %bb176
  %load85 = load i64, ptr %j, align 8
  %add86 = add i64 %load85, 1
  store i64 %add86, ptr %j, align 8
  br label %bb171

bb180:                                            ; preds = %bb173
  %load87 = load i64, ptr %start, align 8
  %load88 = load i64, ptr %i, align 8
  %load89 = load ptr, ptr %self, align 8
  %ptr2int90 = ptrtoint ptr %load89 to i64
  %cmpne91 = icmp ne i64 %ptr2int90, 0
  %zext92 = zext i1 %cmpne91 to i64
  %cond_true93 = icmp ne i64 %zext92, 0
  br i1 %cond_true93, label %bb183, label %bb184

bb182:                                            ; preds = %bb173
  %load94 = load i64, ptr %i, align 8
  %add95 = add i64 %load94, 1
  store i64 %add95, ptr %i, align 8
  br label %bb168

bb183:                                            ; preds = %bb180
  %call96 = call ptr @string_substring(ptr %load89, i64 %load87, i64 %load88)
  %load97 = load ptr, ptr %result, align 8
  %ptr2int98 = ptrtoint ptr %load97 to i64
  %cmpne99 = icmp ne i64 %ptr2int98, 0
  %zext100 = zext i1 %cmpne99 to i64
  %cond_true101 = icmp ne i64 %zext100, 0
  br i1 %cond_true101, label %bb185, label %bb186

bb184:                                            ; preds = %bb180
  call void @panic(ptr @global_str.5)
  unreachable

bb185:                                            ; preds = %bb183
  call void @Vec_str_push(ptr %load97, ptr %call96)
  %load102 = load i64, ptr %i, align 8
  %load103 = load i64, ptr %d_len, align 8
  %add104 = add i64 %load102, %load103
  store i64 %add104, ptr %i, align 8
  %load105 = load i64, ptr %i, align 8
  store i64 %load105, ptr %start, align 8
  br label %bb168

bb186:                                            ; preds = %bb183
  call void @panic(ptr @global_str.5)
  unreachable

bb187:                                            ; preds = %bb170
  %call106 = call ptr @string_substring(ptr %load42, i64 %load40, i64 %load41)
  %load107 = load ptr, ptr %result, align 8
  %ptr2int108 = ptrtoint ptr %load107 to i64
  %cmpne109 = icmp ne i64 %ptr2int108, 0
  %zext110 = zext i1 %cmpne109 to i64
  %cond_true111 = icmp ne i64 %zext110, 0
  br i1 %cond_true111, label %bb189, label %bb190

bb188:                                            ; preds = %bb170
  call void @panic(ptr @global_str.5)
  unreachable

bb189:                                            ; preds = %bb187
  call void @Vec_str_push(ptr %load107, ptr %call106)
  %load112 = load ptr, ptr %result, align 8
  %is_not_null113 = icmp ne ptr %load112, null
  br i1 %is_not_null113, label %arc.retain.do114, label %arc.retain.cont115

bb190:                                            ; preds = %bb187
  call void @panic(ptr @global_str.5)
  unreachable

bb191:                                            ; preds = %arc.retain.cont115
  %is_not_null124 = icmp ne ptr %load119, null
  br i1 %is_not_null124, label %arc.release.do125, label %arc.release.cont126

bb192:                                            ; preds = %arc.release.cont126, %arc.retain.cont115
  ret ptr %load112

arc.retain.do:                                    ; preds = %bb164
  %ref_ptr = getelementptr i64, ptr %load23, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 8
  %new_count = add i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 8
  br label %arc.retain.cont

arc.retain.cont:                                  ; preds = %arc.retain.do, %bb164
  %load24 = load ptr, ptr %result, align 8
  %ptr2int25 = ptrtoint ptr %load24 to i64
  %cmpne26 = icmp ne i64 %ptr2int25, 0
  %zext27 = zext i1 %cmpne26 to i64
  %cond_true28 = icmp ne i64 %zext27, 0
  br i1 %cond_true28, label %bb166, label %bb167

arc.release.do:                                   ; preds = %bb166
  %ref_ptr30 = getelementptr i64, ptr %load24, i64 -2
  %current_count31 = load i64, ptr %ref_ptr30, align 8
  %new_count32 = sub i64 %current_count31, 1
  store i64 %new_count32, ptr %ref_ptr30, align 8
  %is_zero = icmp eq i64 %new_count32, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb166
  br label %bb167

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr30)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont

arc.retain.do114:                                 ; preds = %bb189
  %ref_ptr116 = getelementptr i64, ptr %load112, i64 -2
  %current_count117 = load i64, ptr %ref_ptr116, align 8
  %new_count118 = add i64 %current_count117, 1
  store i64 %new_count118, ptr %ref_ptr116, align 8
  br label %arc.retain.cont115

arc.retain.cont115:                               ; preds = %arc.retain.do114, %bb189
  %load119 = load ptr, ptr %result, align 8
  %ptr2int120 = ptrtoint ptr %load119 to i64
  %cmpne121 = icmp ne i64 %ptr2int120, 0
  %zext122 = zext i1 %cmpne121 to i64
  %cond_true123 = icmp ne i64 %zext122, 0
  br i1 %cond_true123, label %bb191, label %bb192

arc.release.do125:                                ; preds = %bb191
  %ref_ptr127 = getelementptr i64, ptr %load119, i64 -2
  %current_count128 = load i64, ptr %ref_ptr127, align 8
  %new_count129 = sub i64 %current_count128, 1
  store i64 %new_count129, ptr %ref_ptr127, align 8
  %is_zero130 = icmp eq i64 %new_count129, 0
  br i1 %is_zero130, label %arc.free131, label %arc.end132

arc.release.cont126:                              ; preds = %arc.end132, %bb191
  br label %bb192

arc.free131:                                      ; preds = %arc.release.do125
  call void @free(ptr %ref_ptr127)
  br label %arc.end132

arc.end132:                                       ; preds = %arc.free131, %arc.release.do125
  br label %arc.release.cont126
}

define ptr @string_concat(ptr %0, ptr %1) {
bb193:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %other = alloca ptr, align 8
  store ptr %1, ptr %other, align 8
  %len1 = alloca i64, align 8
  store i64 0, ptr %len1, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb194, label %bb195

bb194:                                            ; preds = %bb193
  %self1 = alloca ptr, align 8
  store ptr %load, ptr %self1, align 8
  %load2 = load ptr, ptr %self1, align 8
  %call = call i64 @strlen(ptr %load2)
  store i64 %call, ptr %len1, align 8
  %len2 = alloca i64, align 8
  store i64 0, ptr %len2, align 8
  %load3 = load ptr, ptr %other, align 8
  %ptr2int4 = ptrtoint ptr %load3 to i64
  %cmpne5 = icmp ne i64 %ptr2int4, 0
  %zext6 = zext i1 %cmpne5 to i64
  %cond_true7 = icmp ne i64 %zext6, 0
  br i1 %cond_true7, label %bb196, label %bb197

bb195:                                            ; preds = %bb193
  call void @panic(ptr @global_str.5)
  unreachable

bb196:                                            ; preds = %bb194
  %self8 = alloca ptr, align 8
  store ptr %load3, ptr %self8, align 8
  %load9 = load ptr, ptr %self8, align 8
  %call10 = call i64 @strlen(ptr %load9)
  store i64 %call10, ptr %len2, align 8
  %new_str = alloca ptr, align 8
  store ptr null, ptr %new_str, align 8
  %load11 = load i64, ptr %len1, align 8
  %load12 = load i64, ptr %len2, align 8
  %add = add i64 %load11, %load12
  %add13 = add i64 %add, 1
  %call14 = call ptr @malloc(i64 %add13)
  store ptr %call14, ptr %new_str, align 8
  %load15 = load ptr, ptr %new_str, align 8
  %load16 = load ptr, ptr %self, align 8
  %load17 = load i64, ptr %len1, align 8
  %call18 = call ptr @strncpy(ptr %load15, ptr %load16, i64 %load17)
  %offset_ptr = alloca ptr, align 8
  store ptr null, ptr %offset_ptr, align 8
  %load19 = load ptr, ptr %new_str, align 8
  %ptrtoint = ptrtoint ptr %load19 to i64
  %load20 = load i64, ptr %len1, align 8
  %add21 = add i64 %ptrtoint, %load20
  %inttoptr = inttoptr i64 %add21 to ptr
  store ptr %inttoptr, ptr %offset_ptr, align 8
  %load22 = load ptr, ptr %offset_ptr, align 8
  %load23 = load ptr, ptr %other, align 8
  %load24 = load i64, ptr %len2, align 8
  %call25 = call ptr @strncpy(ptr %load22, ptr %load23, i64 %load24)
  %load26 = load ptr, ptr %new_str, align 8
  %ptr2int27 = ptrtoint ptr %load26 to i64
  %cmpne28 = icmp ne i64 %ptr2int27, 0
  %zext29 = zext i1 %cmpne28 to i64
  %cond_true30 = icmp ne i64 %zext29, 0
  br i1 %cond_true30, label %bb198, label %bb199

bb197:                                            ; preds = %bb194
  call void @panic(ptr @global_str.5)
  unreachable

bb198:                                            ; preds = %bb196
  %load31 = load i64, ptr %len1, align 8
  %load32 = load i64, ptr %len2, align 8
  %add33 = add i64 %load31, %load32
  %gep = getelementptr i8, ptr %load26, i64 %add33
  store i8 0, ptr %gep, align 1
  %load34 = load ptr, ptr %new_str, align 8
  ret ptr %load34

bb199:                                            ; preds = %bb196
  call void @panic(ptr @global_str.3)
  unreachable
}

define i64 @main() {
bb200:
  call void @__global_init()
  %point3 = alloca ptr, align 8
  store ptr null, ptr %point3, align 8
  %struct_alloc = call ptr @malloc(i64 28)
  store i64 1, ptr %struct_alloc, align 8
  %meta_field = getelementptr i64, ptr %struct_alloc, i64 1
  store i64 12, ptr %meta_field, align 8
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
  br i1 %cond_true, label %bb201, label %bb202

bb201:                                            ; preds = %bb200
  %call = call float @Point3_f32_add3(ptr %load)
  %vararg_fpext = fpext float %call to double
  %call3 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext)
  %load4 = load ptr, ptr %point3, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb203, label %bb204

bb202:                                            ; preds = %bb200
  call void @panic(ptr @global_str.5)
  unreachable

bb203:                                            ; preds = %bb201
  %__inline_ret = alloca float, align 4
  %self = alloca ptr, align 8
  store ptr %load4, ptr %self, align 8
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb540, label %bb541

bb540:                                            ; preds = %bb203
  %gep14 = getelementptr %Point3_f32, ptr %load9, i64 0, i32 0
  %load15 = load float, ptr %gep14, align 4
  store float %load15, ptr %__inline_ret, align 4
  %load16 = load float, ptr %__inline_ret, align 4
  %vararg_fpext17 = fpext float %load16 to double
  %call18 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext17)
  %load19 = load ptr, ptr %point3, align 8
  %ptr2int20 = ptrtoint ptr %load19 to i64
  %cmpne21 = icmp ne i64 %ptr2int20, 0
  %zext22 = zext i1 %cmpne21 to i64
  %cond_true23 = icmp ne i64 %zext22, 0
  br i1 %cond_true23, label %bb205, label %bb206

bb541:                                            ; preds = %bb203
  call void @panic(ptr @global_str.7)
  unreachable

bb204:                                            ; preds = %bb201
  call void @panic(ptr @global_str.5)
  unreachable

bb205:                                            ; preds = %bb540
  %__inline_ret24 = alloca float, align 4
  %self25 = alloca ptr, align 8
  store ptr %load19, ptr %self25, align 8
  %load26 = load ptr, ptr %self25, align 8
  %ptr2int27 = ptrtoint ptr %load26 to i64
  %cmpne28 = icmp ne i64 %ptr2int27, 0
  %zext29 = zext i1 %cmpne28 to i64
  %cond_true30 = icmp ne i64 %zext29, 0
  br i1 %cond_true30, label %bb544, label %bb545

bb544:                                            ; preds = %bb205
  %gep31 = getelementptr %Point3_f32, ptr %load26, i64 0, i32 1
  %load32 = load float, ptr %gep31, align 4
  store float %load32, ptr %__inline_ret24, align 4
  %load33 = load float, ptr %__inline_ret24, align 4
  %vararg_fpext34 = fpext float %load33 to double
  %call35 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext34)
  %load36 = load ptr, ptr %point3, align 8
  %ptr2int37 = ptrtoint ptr %load36 to i64
  %cmpne38 = icmp ne i64 %ptr2int37, 0
  %zext39 = zext i1 %cmpne38 to i64
  %cond_true40 = icmp ne i64 %zext39, 0
  br i1 %cond_true40, label %bb207, label %bb208

bb545:                                            ; preds = %bb205
  call void @panic(ptr @global_str.7)
  unreachable

bb206:                                            ; preds = %bb540
  call void @panic(ptr @global_str.5)
  unreachable

bb207:                                            ; preds = %bb544
  %__inline_ret41 = alloca float, align 4
  %self42 = alloca ptr, align 8
  store ptr %load36, ptr %self42, align 8
  %load43 = load ptr, ptr %self42, align 8
  %ptr2int44 = ptrtoint ptr %load43 to i64
  %cmpne45 = icmp ne i64 %ptr2int44, 0
  %zext46 = zext i1 %cmpne45 to i64
  %cond_true47 = icmp ne i64 %zext46, 0
  br i1 %cond_true47, label %bb548, label %bb549

bb548:                                            ; preds = %bb207
  %gep48 = getelementptr %Point3_f32, ptr %load43, i64 0, i32 2
  %load49 = load float, ptr %gep48, align 4
  store float %load49, ptr %__inline_ret41, align 4
  %load50 = load float, ptr %__inline_ret41, align 4
  %vararg_fpext51 = fpext float %load50 to double
  %call52 = call i32 (ptr, ...) @printf(ptr @global_str.6, double %vararg_fpext51)
  %call53 = call i32 (ptr, ...) @printf(ptr @global_str.6, double 0x3FF1980000000000)
  %call54 = call i32 (ptr, ...) @printf(ptr @global_str.6, double 0x3FF19999A0000000)
  %call55 = call i32 (ptr, ...) @printf(ptr @global_str.6, double 1.100000e+00)
  %a = alloca i64, align 8
  store i64 0, ptr %a, align 8
  store i64 1, ptr %a, align 8
  %call56 = call i32 (ptr, ...) @printf(ptr @global_str.6, double -6.550400e+04)
  %str1 = alloca ptr, align 8
  store ptr null, ptr %str1, align 8
  store ptr @global_str.8, ptr %str1, align 8
  %load57 = load ptr, ptr %str1, align 8
  %ptr2int58 = ptrtoint ptr %load57 to i64
  %cmpne59 = icmp ne i64 %ptr2int58, 0
  %zext60 = zext i1 %cmpne59 to i64
  %cond_true61 = icmp ne i64 %zext60, 0
  br i1 %cond_true61, label %bb209, label %bb210

bb549:                                            ; preds = %bb207
  call void @panic(ptr @global_str.7)
  unreachable

bb208:                                            ; preds = %bb544
  call void @panic(ptr @global_str.5)
  unreachable

bb209:                                            ; preds = %bb548
  %self62 = alloca ptr, align 8
  store ptr %load57, ptr %self62, align 8
  %load63 = load ptr, ptr %self62, align 8
  %call64 = call i64 @strlen(ptr %load63)
  %call65 = call i32 (ptr, ...) @printf(ptr @global_str.9, i64 %call64)
  %load66 = load ptr, ptr %str1, align 8
  %ptr2int67 = ptrtoint ptr %load66 to i64
  %cmpne68 = icmp ne i64 %ptr2int67, 0
  %zext69 = zext i1 %cmpne68 to i64
  %cond_true70 = icmp ne i64 %zext69, 0
  br i1 %cond_true70, label %bb211, label %bb212

bb210:                                            ; preds = %bb548
  call void @panic(ptr @global_str.5)
  unreachable

bb211:                                            ; preds = %bb209
  %__inline_ret71 = alloca i1, align 1
  %self72 = alloca ptr, align 8
  store ptr %load66, ptr %self72, align 8
  %load73 = load ptr, ptr %self72, align 8
  %ptr2int74 = ptrtoint ptr %load73 to i64
  %cmpne75 = icmp ne i64 %ptr2int74, 0
  %zext76 = zext i1 %cmpne75 to i64
  %cond_true77 = icmp ne i64 %zext76, 0
  br i1 %cond_true77, label %bb552, label %bb553

bb552:                                            ; preds = %bb211
  %call78 = call i64 @string_len(ptr %load73)
  %cmpeq = icmp eq i64 %call78, 0
  %zext79 = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext79 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %trunc = trunc i64 %cast_zext to i1
  store i1 %trunc, ptr %__inline_ret71, align 1
  %load80 = load i1, ptr %__inline_ret71, align 1
  %load_zext = zext i1 %load80 to i64
  %cast_trunc81 = trunc i64 %load_zext to i1
  %cast_zext82 = zext i1 %cast_trunc81 to i64
  %__inline_ret83 = alloca ptr, align 8
  %self84 = alloca i1, align 1
  %trunc85 = trunc i64 %cast_zext82 to i1
  store i1 %trunc85, ptr %self84, align 1
  %load86 = load i1, ptr %self84, align 1
  %load_zext87 = zext i1 %load86 to i64
  %cast_trunc88 = trunc i64 %load_zext87 to i1
  %cast_zext89 = zext i1 %cast_trunc88 to i64
  %cond_true90 = icmp ne i64 %cast_zext89, 0
  br i1 %cond_true90, label %bb556, label %bb557

bb553:                                            ; preds = %bb211
  call void @panic(ptr @global_str.5)
  unreachable

bb556:                                            ; preds = %bb552
  store ptr @global_str.11, ptr %__inline_ret83, align 8
  br label %bb558

bb557:                                            ; preds = %bb552
  store ptr @global_str.12, ptr %__inline_ret83, align 8
  br label %bb558

bb558:                                            ; preds = %bb557, %bb556
  %load91 = load ptr, ptr %__inline_ret83, align 8
  %call92 = call i32 (ptr, ...) @printf(ptr @global_str.10, ptr %load91)
  %load93 = load ptr, ptr %str1, align 8
  %ptr2int94 = ptrtoint ptr %load93 to i64
  %cmpne95 = icmp ne i64 %ptr2int94, 0
  %zext96 = zext i1 %cmpne95 to i64
  %cond_true97 = icmp ne i64 %zext96, 0
  br i1 %cond_true97, label %bb213, label %bb214

bb212:                                            ; preds = %bb209
  call void @panic(ptr @global_str.5)
  unreachable

bb213:                                            ; preds = %bb558
  %call98 = call i1 @string_contains(ptr %load93, ptr @global_str.13)
  %cast_zext99 = zext i1 %call98 to i64
  %__inline_ret100 = alloca ptr, align 8
  %self101 = alloca i1, align 1
  %trunc102 = trunc i64 %cast_zext99 to i1
  store i1 %trunc102, ptr %self101, align 1
  %load103 = load i1, ptr %self101, align 1
  %load_zext104 = zext i1 %load103 to i64
  %cast_trunc105 = trunc i64 %load_zext104 to i1
  %cast_zext106 = zext i1 %cast_trunc105 to i64
  %cond_true107 = icmp ne i64 %cast_zext106, 0
  br i1 %cond_true107, label %bb560, label %bb561

bb560:                                            ; preds = %bb213
  store ptr @global_str.11, ptr %__inline_ret100, align 8
  br label %bb562

bb561:                                            ; preds = %bb213
  store ptr @global_str.12, ptr %__inline_ret100, align 8
  br label %bb562

bb562:                                            ; preds = %bb561, %bb560
  %load108 = load ptr, ptr %__inline_ret100, align 8
  %call109 = call i32 (ptr, ...) @printf(ptr @global_str.10, ptr %load108)
  %load110 = load ptr, ptr %str1, align 8
  %ptr2int111 = ptrtoint ptr %load110 to i64
  %cmpne112 = icmp ne i64 %ptr2int111, 0
  %zext113 = zext i1 %cmpne112 to i64
  %cond_true114 = icmp ne i64 %zext113, 0
  br i1 %cond_true114, label %bb215, label %bb216

bb214:                                            ; preds = %bb558
  call void @panic(ptr @global_str.5)
  unreachable

bb215:                                            ; preds = %bb562
  %call115 = call i1 @string_starts_with(ptr %load110, ptr @global_str.14)
  %cast_zext116 = zext i1 %call115 to i64
  %__inline_ret117 = alloca ptr, align 8
  %self118 = alloca i1, align 1
  %trunc119 = trunc i64 %cast_zext116 to i1
  store i1 %trunc119, ptr %self118, align 1
  %load120 = load i1, ptr %self118, align 1
  %load_zext121 = zext i1 %load120 to i64
  %cast_trunc122 = trunc i64 %load_zext121 to i1
  %cast_zext123 = zext i1 %cast_trunc122 to i64
  %cond_true124 = icmp ne i64 %cast_zext123, 0
  br i1 %cond_true124, label %bb564, label %bb565

bb564:                                            ; preds = %bb215
  store ptr @global_str.11, ptr %__inline_ret117, align 8
  br label %bb566

bb565:                                            ; preds = %bb215
  store ptr @global_str.12, ptr %__inline_ret117, align 8
  br label %bb566

bb566:                                            ; preds = %bb565, %bb564
  %load125 = load ptr, ptr %__inline_ret117, align 8
  %call126 = call i32 (ptr, ...) @printf(ptr @global_str.10, ptr %load125)
  %load127 = load ptr, ptr %str1, align 8
  %ptr2int128 = ptrtoint ptr %load127 to i64
  %cmpne129 = icmp ne i64 %ptr2int128, 0
  %zext130 = zext i1 %cmpne129 to i64
  %cond_true131 = icmp ne i64 %zext130, 0
  br i1 %cond_true131, label %bb217, label %bb218

bb216:                                            ; preds = %bb562
  call void @panic(ptr @global_str.5)
  unreachable

bb217:                                            ; preds = %bb566
  %call132 = call i1 @string_ends_with(ptr %load127, ptr @global_str.15)
  %cast_zext133 = zext i1 %call132 to i64
  %__inline_ret134 = alloca ptr, align 8
  %self135 = alloca i1, align 1
  %trunc136 = trunc i64 %cast_zext133 to i1
  store i1 %trunc136, ptr %self135, align 1
  %load137 = load i1, ptr %self135, align 1
  %load_zext138 = zext i1 %load137 to i64
  %cast_trunc139 = trunc i64 %load_zext138 to i1
  %cast_zext140 = zext i1 %cast_trunc139 to i64
  %cond_true141 = icmp ne i64 %cast_zext140, 0
  br i1 %cond_true141, label %bb568, label %bb569

bb568:                                            ; preds = %bb217
  store ptr @global_str.11, ptr %__inline_ret134, align 8
  br label %bb570

bb569:                                            ; preds = %bb217
  store ptr @global_str.12, ptr %__inline_ret134, align 8
  br label %bb570

bb570:                                            ; preds = %bb569, %bb568
  %load142 = load ptr, ptr %__inline_ret134, align 8
  %call143 = call i32 (ptr, ...) @printf(ptr @global_str.10, ptr %load142)
  %load144 = load ptr, ptr %str1, align 8
  %load145 = load ptr, ptr %str1, align 8
  %ptr2int146 = ptrtoint ptr %load145 to i64
  %cmpne147 = icmp ne i64 %ptr2int146, 0
  %zext148 = zext i1 %cmpne147 to i64
  %cond_true149 = icmp ne i64 %zext148, 0
  br i1 %cond_true149, label %bb219, label %bb220

bb218:                                            ; preds = %bb566
  call void @panic(ptr @global_str.5)
  unreachable

bb219:                                            ; preds = %bb570
  %self150 = alloca ptr, align 8
  store ptr %load145, ptr %self150, align 8
  %other = alloca ptr, align 8
  store ptr %load144, ptr %other, align 8
  %load151 = load ptr, ptr %self150, align 8
  %load152 = load ptr, ptr %other, align 8
  %call153 = call i32 @strcmp(ptr %load151, ptr %load152)
  %cast_sext = sext i32 %call153 to i64
  %cmpeq154 = icmp eq i64 %cast_sext, 0
  %zext155 = zext i1 %cmpeq154 to i64
  %cast_trunc156 = trunc i64 %zext155 to i1
  %cast_zext157 = zext i1 %cast_trunc156 to i64
  %cast_trunc158 = trunc i64 %cast_zext157 to i1
  %cast_zext159 = zext i1 %cast_trunc158 to i64
  %__inline_ret160 = alloca ptr, align 8
  %self161 = alloca i1, align 1
  %trunc162 = trunc i64 %cast_zext159 to i1
  store i1 %trunc162, ptr %self161, align 1
  %load163 = load i1, ptr %self161, align 1
  %load_zext164 = zext i1 %load163 to i64
  %cast_trunc165 = trunc i64 %load_zext164 to i1
  %cast_zext166 = zext i1 %cast_trunc165 to i64
  %cond_true167 = icmp ne i64 %cast_zext166, 0
  br i1 %cond_true167, label %bb572, label %bb573

bb572:                                            ; preds = %bb219
  store ptr @global_str.11, ptr %__inline_ret160, align 8
  br label %bb574

bb573:                                            ; preds = %bb219
  store ptr @global_str.12, ptr %__inline_ret160, align 8
  br label %bb574

bb574:                                            ; preds = %bb573, %bb572
  %load168 = load ptr, ptr %__inline_ret160, align 8
  %call169 = call i32 (ptr, ...) @printf(ptr @global_str.10, ptr %load168)
  %load170 = load ptr, ptr %str1, align 8
  %ptr2int171 = ptrtoint ptr %load170 to i64
  %cmpne172 = icmp ne i64 %ptr2int171, 0
  %zext173 = zext i1 %cmpne172 to i64
  %cond_true174 = icmp ne i64 %zext173, 0
  br i1 %cond_true174, label %bb221, label %bb222

bb220:                                            ; preds = %bb570
  call void @panic(ptr @global_str.5)
  unreachable

bb221:                                            ; preds = %bb574
  %call175 = call ptr @string_substring(ptr %load170, i64 0, i64 3)
  %call176 = call i32 (ptr, ...) @printf(ptr @global_str.10, ptr %call175)
  %n1 = alloca i8, align 1
  store i8 0, ptr %n1, align 1
  store i8 -2, ptr %n1, align 1
  %n2 = alloca i8, align 1
  store i8 0, ptr %n2, align 1
  store i8 1, ptr %n2, align 1
  %load177 = load i8, ptr %n1, align 1
  %load_zext178 = zext i8 %load177 to i64
  %cast_trunc179 = trunc i64 %load_zext178 to i8
  %cast_zext180 = zext i8 %cast_trunc179 to i64
  %load181 = load i8, ptr %n2, align 1
  %load_zext182 = zext i8 %load181 to i64
  %cast_trunc183 = trunc i64 %load_zext182 to i8
  %cast_zext184 = zext i8 %cast_trunc183 to i64
  %add = add i64 %cast_zext180, %cast_zext184
  %cast_trunc185 = trunc i64 %add to i8
  %cast_zext186 = zext i8 %cast_trunc185 to i64
  %call187 = call i32 (ptr, ...) @printf(ptr @global_str.9, i64 %cast_zext186)
  %load188 = load ptr, ptr %str1, align 8
  %load189 = load ptr, ptr %str1, align 8
  %ptr2int190 = ptrtoint ptr %load189 to i64
  %cmpne191 = icmp ne i64 %ptr2int190, 0
  %zext192 = zext i1 %cmpne191 to i64
  %cond_true193 = icmp ne i64 %zext192, 0
  br i1 %cond_true193, label %bb223, label %bb224

bb222:                                            ; preds = %bb574
  call void @panic(ptr @global_str.5)
  unreachable

bb223:                                            ; preds = %bb221
  %call194 = call ptr @string_concat(ptr %load189, ptr %load188)
  %call195 = call i32 (ptr, ...) @printf(ptr @global_str.10, ptr %call194)
  %call196 = call i32 (ptr, ...) @printf(ptr @global_str.16)
  %stdin = alloca ptr, align 8
  store ptr null, ptr %stdin, align 8
  %call197 = call ptr @read_line(i64 128)
  store ptr %call197, ptr %stdin, align 8
  %fruits = alloca ptr, align 8
  store ptr null, ptr %fruits, align 8
  %load198 = load ptr, ptr %stdin, align 8
  %ptr2int199 = ptrtoint ptr %load198 to i64
  %cmpne200 = icmp ne i64 %ptr2int199, 0
  %zext201 = zext i1 %cmpne200 to i64
  %cond_true202 = icmp ne i64 %zext201, 0
  br i1 %cond_true202, label %bb225, label %bb226

bb224:                                            ; preds = %bb221
  call void @panic(ptr @global_str.5)
  unreachable

bb225:                                            ; preds = %bb223
  %call203 = call ptr @string_split(ptr %load198, ptr @global_str.17)
  store ptr %call203, ptr %fruits, align 8
  %call204 = call i32 (ptr, ...) @printf(ptr @global_str.18)
  %i = alloca i64, align 8
  store i64 0, ptr %i, align 8
  store i64 0, ptr %i, align 8
  br label %bb227

bb226:                                            ; preds = %bb223
  call void @panic(ptr @global_str.5)
  unreachable

bb227:                                            ; preds = %bb232, %bb225
  %load205 = load i64, ptr %i, align 8
  %load206 = load ptr, ptr %fruits, align 8
  %ptr2int207 = ptrtoint ptr %load206 to i64
  %cmpne208 = icmp ne i64 %ptr2int207, 0
  %zext209 = zext i1 %cmpne208 to i64
  %cond_true210 = icmp ne i64 %zext209, 0
  br i1 %cond_true210, label %bb230, label %bb231

bb228:                                            ; preds = %bb576
  %load211 = load i64, ptr %i, align 8
  %load212 = load ptr, ptr %fruits, align 8
  %ptr2int213 = ptrtoint ptr %load212 to i64
  %cmpne214 = icmp ne i64 %ptr2int213, 0
  %zext215 = zext i1 %cmpne214 to i64
  %cond_true216 = icmp ne i64 %zext215, 0
  br i1 %cond_true216, label %bb232, label %bb233

bb229:                                            ; preds = %bb576
  %load217 = load ptr, ptr %fruits, align 8
  %ptr2int218 = ptrtoint ptr %load217 to i64
  %cmpne219 = icmp ne i64 %ptr2int218, 0
  %zext220 = zext i1 %cmpne219 to i64
  %cond_true221 = icmp ne i64 %zext220, 0
  br i1 %cond_true221, label %bb234, label %bb235

bb230:                                            ; preds = %bb227
  %__inline_ret222 = alloca i64, align 8
  %self223 = alloca ptr, align 8
  store ptr %load206, ptr %self223, align 8
  %load224 = load ptr, ptr %self223, align 8
  %ptr2int225 = ptrtoint ptr %load224 to i64
  %cmpne226 = icmp ne i64 %ptr2int225, 0
  %zext227 = zext i1 %cmpne226 to i64
  %cond_true228 = icmp ne i64 %zext227, 0
  br i1 %cond_true228, label %bb576, label %bb577

bb576:                                            ; preds = %bb230
  %gep229 = getelementptr %Vec_str, ptr %load224, i64 0, i32 1
  %load230 = load i64, ptr %gep229, align 8
  store i64 %load230, ptr %__inline_ret222, align 8
  %load231 = load i64, ptr %__inline_ret222, align 8
  %cmplt = icmp ult i64 %load205, %load231
  %zext232 = zext i1 %cmplt to i64
  %cast_trunc233 = trunc i64 %zext232 to i1
  %cast_zext234 = zext i1 %cast_trunc233 to i64
  %cond_true235 = icmp ne i64 %cast_zext234, 0
  br i1 %cond_true235, label %bb228, label %bb229

bb577:                                            ; preds = %bb230
  call void @panic(ptr @global_str.7)
  unreachable

bb231:                                            ; preds = %bb227
  call void @panic(ptr @global_str.5)
  unreachable

bb232:                                            ; preds = %bb228
  %call236 = call ptr @Vec_str_get(ptr %load212, i64 %load211)
  %call237 = call i32 (ptr, ...) @printf(ptr @global_str.19, ptr %call236)
  %load238 = load i64, ptr %i, align 8
  %add239 = add i64 %load238, 1
  store i64 %add239, ptr %i, align 8
  br label %bb227

bb233:                                            ; preds = %bb228
  call void @panic(ptr @global_str.5)
  unreachable

bb234:                                            ; preds = %bb229
  call void @Vec_str_drop(ptr %load217)
  %load240 = load ptr, ptr %stdin, align 8
  call void @free(ptr %load240)
  store ptr null, ptr %stdin, align 8
  %load241 = load ptr, ptr %fruits, align 8
  %ptr2int242 = ptrtoint ptr %load241 to i64
  %cmpne243 = icmp ne i64 %ptr2int242, 0
  %zext244 = zext i1 %cmpne243 to i64
  %cond_true245 = icmp ne i64 %zext244, 0
  br i1 %cond_true245, label %bb236, label %bb237

bb235:                                            ; preds = %bb229
  call void @panic(ptr @global_str.5)
  unreachable

bb236:                                            ; preds = %bb234
  %is_not_null = icmp ne ptr %load241, null
  br i1 %is_not_null, label %arc.release.do, label %arc.release.cont

bb237:                                            ; preds = %arc.release.cont, %bb234
  %load246 = load ptr, ptr %point3, align 8
  %ptr2int247 = ptrtoint ptr %load246 to i64
  %cmpne248 = icmp ne i64 %ptr2int247, 0
  %zext249 = zext i1 %cmpne248 to i64
  %cond_true250 = icmp ne i64 %zext249, 0
  br i1 %cond_true250, label %bb238, label %bb239

bb238:                                            ; preds = %bb237
  %is_not_null251 = icmp ne ptr %load246, null
  br i1 %is_not_null251, label %arc.release.do252, label %arc.release.cont253

bb239:                                            ; preds = %arc.release.cont253, %bb237
  ret i64 0

arc.release.do:                                   ; preds = %bb236
  %ref_ptr = getelementptr i64, ptr %load241, i64 -2
  %current_count = load i64, ptr %ref_ptr, align 8
  %new_count = sub i64 %current_count, 1
  store i64 %new_count, ptr %ref_ptr, align 8
  %is_zero = icmp eq i64 %new_count, 0
  br i1 %is_zero, label %arc.free, label %arc.end

arc.release.cont:                                 ; preds = %arc.end, %bb236
  br label %bb237

arc.free:                                         ; preds = %arc.release.do
  call void @free(ptr %ref_ptr)
  br label %arc.end

arc.end:                                          ; preds = %arc.free, %arc.release.do
  br label %arc.release.cont

arc.release.do252:                                ; preds = %bb238
  %ref_ptr254 = getelementptr i64, ptr %load246, i64 -2
  %current_count255 = load i64, ptr %ref_ptr254, align 8
  %new_count256 = sub i64 %current_count255, 1
  store i64 %new_count256, ptr %ref_ptr254, align 8
  %is_zero257 = icmp eq i64 %new_count256, 0
  br i1 %is_zero257, label %arc.free258, label %arc.end259

arc.release.cont253:                              ; preds = %arc.end259, %bb238
  br label %bb239

arc.free258:                                      ; preds = %arc.release.do252
  call void @free(ptr %ref_ptr254)
  br label %arc.end259

arc.end259:                                       ; preds = %arc.free258, %arc.release.do252
  br label %arc.release.cont253
}

define void @Vec_str_reserve(ptr %0, i64 %1) {
bb277:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %new_cap = alloca i64, align 8
  store i64 %1, ptr %new_cap, align 8
  %load = load i64, ptr %new_cap, align 8
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb278, label %bb279

bb278:                                            ; preds = %bb277
  %gep = getelementptr %Vec_str, ptr %load1, i64 0, i32 2
  %load2 = load i64, ptr %gep, align 8
  %cmple = icmp ule i64 %load, %load2
  %zext3 = zext i1 %cmple to i64
  %cast_trunc = trunc i64 %zext3 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true4 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true4, label %bb280, label %bb281

bb279:                                            ; preds = %bb277
  call void @panic(ptr @global_str.7)
  unreachable

bb280:                                            ; preds = %bb278
  ret void

bb281:                                            ; preds = %bb278
  %size_in_bytes = alloca i64, align 8
  store i64 0, ptr %size_in_bytes, align 8
  %load5 = load i64, ptr %new_cap, align 8
  %mul = mul i64 %load5, 8
  store i64 %mul, ptr %size_in_bytes, align 8
  %load6 = load ptr, ptr %self, align 8
  %ptr2int7 = ptrtoint ptr %load6 to i64
  %cmpne8 = icmp ne i64 %ptr2int7, 0
  %zext9 = zext i1 %cmpne8 to i64
  %cond_true10 = icmp ne i64 %zext9, 0
  br i1 %cond_true10, label %bb282, label %bb283

bb282:                                            ; preds = %bb281
  %gep11 = getelementptr %Vec_str, ptr %load6, i64 0, i32 2
  %load12 = load i64, ptr %gep11, align 8
  %cmpeq = icmp eq i64 %load12, 0
  %zext13 = zext i1 %cmpeq to i64
  %cast_trunc14 = trunc i64 %zext13 to i1
  %cast_zext15 = zext i1 %cast_trunc14 to i64
  %cond_true16 = icmp ne i64 %cast_zext15, 0
  br i1 %cond_true16, label %bb284, label %bb286

bb283:                                            ; preds = %bb281
  call void @panic(ptr @global_str.7)
  unreachable

bb284:                                            ; preds = %bb282
  %load17 = load ptr, ptr %self, align 8
  %ptr2int18 = ptrtoint ptr %load17 to i64
  %cmpne19 = icmp ne i64 %ptr2int18, 0
  %zext20 = zext i1 %cmpne19 to i64
  %cond_true21 = icmp ne i64 %zext20, 0
  br i1 %cond_true21, label %bb287, label %bb288

bb285:                                            ; preds = %bb291, %bb287
  %load22 = load ptr, ptr %self, align 8
  %ptr2int23 = ptrtoint ptr %load22 to i64
  %cmpne24 = icmp ne i64 %ptr2int23, 0
  %zext25 = zext i1 %cmpne24 to i64
  %cond_true26 = icmp ne i64 %zext25, 0
  br i1 %cond_true26, label %bb293, label %bb294

bb286:                                            ; preds = %bb282
  %load27 = load ptr, ptr %self, align 8
  %ptr2int28 = ptrtoint ptr %load27 to i64
  %cmpne29 = icmp ne i64 %ptr2int28, 0
  %zext30 = zext i1 %cmpne29 to i64
  %cond_true31 = icmp ne i64 %zext30, 0
  br i1 %cond_true31, label %bb289, label %bb290

bb287:                                            ; preds = %bb284
  %load32 = load i64, ptr %size_in_bytes, align 8
  %call = call ptr @malloc(i64 %load32)
  %gep33 = getelementptr %Vec_str, ptr %load17, i64 0, i32 0
  store ptr %call, ptr %gep33, align 8
  br label %bb285

bb288:                                            ; preds = %bb284
  call void @panic(ptr @global_str.20)
  unreachable

bb289:                                            ; preds = %bb286
  %load34 = load ptr, ptr %self, align 8
  %ptr2int35 = ptrtoint ptr %load34 to i64
  %cmpne36 = icmp ne i64 %ptr2int35, 0
  %zext37 = zext i1 %cmpne36 to i64
  %cond_true38 = icmp ne i64 %zext37, 0
  br i1 %cond_true38, label %bb291, label %bb292

bb290:                                            ; preds = %bb286
  call void @panic(ptr @global_str.20)
  unreachable

bb291:                                            ; preds = %bb289
  %gep39 = getelementptr %Vec_str, ptr %load34, i64 0, i32 0
  %load40 = load ptr, ptr %gep39, align 8
  %load41 = load i64, ptr %size_in_bytes, align 8
  %call42 = call ptr @realloc(ptr %load40, i64 %load41)
  %gep43 = getelementptr %Vec_str, ptr %load27, i64 0, i32 0
  store ptr %call42, ptr %gep43, align 8
  br label %bb285

bb292:                                            ; preds = %bb289
  call void @panic(ptr @global_str.7)
  unreachable

bb293:                                            ; preds = %bb285
  %gep44 = getelementptr %Vec_str, ptr %load22, i64 0, i32 0
  %load45 = load ptr, ptr %gep44, align 8
  %ptr2int46 = ptrtoint ptr %load45 to i64
  %cmpeq47 = icmp eq i64 %ptr2int46, 0
  %zext48 = zext i1 %cmpeq47 to i64
  %cast_trunc49 = trunc i64 %zext48 to i1
  %cast_zext50 = zext i1 %cast_trunc49 to i64
  %cond_true51 = icmp ne i64 %cast_zext50, 0
  br i1 %cond_true51, label %bb295, label %bb296

bb294:                                            ; preds = %bb285
  call void @panic(ptr @global_str.7)
  unreachable

bb295:                                            ; preds = %bb293
  call void @panic(ptr @global_str.21)
  br label %bb296

bb296:                                            ; preds = %bb295, %bb293
  %load52 = load ptr, ptr %self, align 8
  %ptr2int53 = ptrtoint ptr %load52 to i64
  %cmpne54 = icmp ne i64 %ptr2int53, 0
  %zext55 = zext i1 %cmpne54 to i64
  %cond_true56 = icmp ne i64 %zext55, 0
  br i1 %cond_true56, label %bb297, label %bb298

bb297:                                            ; preds = %bb296
  %load57 = load i64, ptr %new_cap, align 8
  %gep58 = getelementptr %Vec_str, ptr %load52, i64 0, i32 2
  store i64 %load57, ptr %gep58, align 8
  ret void

bb298:                                            ; preds = %bb296
  call void @panic(ptr @global_str.20)
  unreachable
}

define void @Vec_str_drop(ptr %0) {
bb299:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb300, label %bb301

bb300:                                            ; preds = %bb299
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 2
  %load1 = load i64, ptr %gep, align 8
  %cmpgt = icmp ugt i64 %load1, 0
  %zext2 = zext i1 %cmpgt to i64
  %cast_trunc = trunc i64 %zext2 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true3 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true3, label %bb302, label %bb303

bb301:                                            ; preds = %bb299
  call void @panic(ptr @global_str.7)
  unreachable

bb302:                                            ; preds = %bb300
  %load4 = load ptr, ptr %self, align 8
  %ptr2int5 = ptrtoint ptr %load4 to i64
  %cmpne6 = icmp ne i64 %ptr2int5, 0
  %zext7 = zext i1 %cmpne6 to i64
  %cond_true8 = icmp ne i64 %zext7, 0
  br i1 %cond_true8, label %bb304, label %bb305

bb303:                                            ; preds = %bb308, %bb300
  ret void

bb304:                                            ; preds = %bb302
  %gep9 = getelementptr %Vec_str, ptr %load4, i64 0, i32 0
  %load10 = load ptr, ptr %gep9, align 8
  call void @free(ptr %load10)
  %load11 = load ptr, ptr %self, align 8
  %gep12 = getelementptr i64, ptr %load11, i64 0
  store i64 0, ptr %gep12, align 8
  %load13 = load ptr, ptr %self, align 8
  %ptr2int14 = ptrtoint ptr %load13 to i64
  %cmpne15 = icmp ne i64 %ptr2int14, 0
  %zext16 = zext i1 %cmpne15 to i64
  %cond_true17 = icmp ne i64 %zext16, 0
  br i1 %cond_true17, label %bb306, label %bb307

bb305:                                            ; preds = %bb302
  call void @panic(ptr @global_str.7)
  unreachable

bb306:                                            ; preds = %bb304
  %gep18 = getelementptr %Vec_str, ptr %load13, i64 0, i32 2
  store i64 0, ptr %gep18, align 8
  %load19 = load ptr, ptr %self, align 8
  %ptr2int20 = ptrtoint ptr %load19 to i64
  %cmpne21 = icmp ne i64 %ptr2int20, 0
  %zext22 = zext i1 %cmpne21 to i64
  %cond_true23 = icmp ne i64 %zext22, 0
  br i1 %cond_true23, label %bb308, label %bb309

bb307:                                            ; preds = %bb304
  call void @panic(ptr @global_str.20)
  unreachable

bb308:                                            ; preds = %bb306
  %gep24 = getelementptr %Vec_str, ptr %load19, i64 0, i32 1
  store i64 0, ptr %gep24, align 8
  br label %bb303

bb309:                                            ; preds = %bb306
  call void @panic(ptr @global_str.20)
  unreachable
}

define void @Vec_str_push(ptr %0, ptr %1) {
bb310:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %item = alloca ptr, align 8
  store ptr %1, ptr %item, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb311, label %bb312

bb311:                                            ; preds = %bb310
  %gep = getelementptr %Vec_str, ptr %load, i64 0, i32 1
  %load1 = load i64, ptr %gep, align 8
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb313, label %bb314

bb312:                                            ; preds = %bb310
  call void @panic(ptr @global_str.7)
  unreachable

bb313:                                            ; preds = %bb311
  %gep7 = getelementptr %Vec_str, ptr %load2, i64 0, i32 2
  %load8 = load i64, ptr %gep7, align 8
  %cmpeq = icmp eq i64 %load1, %load8
  %zext9 = zext i1 %cmpeq to i64
  %cast_trunc = trunc i64 %zext9 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true10 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true10, label %bb315, label %bb316

bb314:                                            ; preds = %bb311
  call void @panic(ptr @global_str.7)
  unreachable

bb315:                                            ; preds = %bb313
  %next_cap = alloca i64, align 8
  store i64 0, ptr %next_cap, align 8
  store i64 4, ptr %next_cap, align 8
  %load11 = load ptr, ptr %self, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb317, label %bb318

bb316:                                            ; preds = %bb323, %bb313
  %load16 = load ptr, ptr %self, align 8
  %ptr2int17 = ptrtoint ptr %load16 to i64
  %cmpne18 = icmp ne i64 %ptr2int17, 0
  %zext19 = zext i1 %cmpne18 to i64
  %cond_true20 = icmp ne i64 %zext19, 0
  br i1 %cond_true20, label %bb325, label %bb326

bb317:                                            ; preds = %bb315
  %gep21 = getelementptr %Vec_str, ptr %load11, i64 0, i32 2
  %load22 = load i64, ptr %gep21, align 8
  %cmpgt = icmp ugt i64 %load22, 0
  %zext23 = zext i1 %cmpgt to i64
  %cast_trunc24 = trunc i64 %zext23 to i1
  %cast_zext25 = zext i1 %cast_trunc24 to i64
  %cond_true26 = icmp ne i64 %cast_zext25, 0
  br i1 %cond_true26, label %bb319, label %bb320

bb318:                                            ; preds = %bb315
  call void @panic(ptr @global_str.7)
  unreachable

bb319:                                            ; preds = %bb317
  %load27 = load ptr, ptr %self, align 8
  %ptr2int28 = ptrtoint ptr %load27 to i64
  %cmpne29 = icmp ne i64 %ptr2int28, 0
  %zext30 = zext i1 %cmpne29 to i64
  %cond_true31 = icmp ne i64 %zext30, 0
  br i1 %cond_true31, label %bb321, label %bb322

bb320:                                            ; preds = %bb321, %bb317
  %load32 = load i64, ptr %next_cap, align 8
  %load33 = load ptr, ptr %self, align 8
  %ptr2int34 = ptrtoint ptr %load33 to i64
  %cmpne35 = icmp ne i64 %ptr2int34, 0
  %zext36 = zext i1 %cmpne35 to i64
  %cond_true37 = icmp ne i64 %zext36, 0
  br i1 %cond_true37, label %bb323, label %bb324

bb321:                                            ; preds = %bb319
  %gep38 = getelementptr %Vec_str, ptr %load27, i64 0, i32 2
  %load39 = load i64, ptr %gep38, align 8
  %mul = mul i64 %load39, 2
  store i64 %mul, ptr %next_cap, align 8
  br label %bb320

bb322:                                            ; preds = %bb319
  call void @panic(ptr @global_str.7)
  unreachable

bb323:                                            ; preds = %bb320
  call void @Vec_str_reserve(ptr %load33, i64 %load32)
  br label %bb316

bb324:                                            ; preds = %bb320
  call void @panic(ptr @global_str.5)
  unreachable

bb325:                                            ; preds = %bb316
  %gep40 = getelementptr %Vec_str, ptr %load16, i64 0, i32 0
  %load41 = load ptr, ptr %gep40, align 8
  %ptr2int42 = ptrtoint ptr %load41 to i64
  %cmpne43 = icmp ne i64 %ptr2int42, 0
  %zext44 = zext i1 %cmpne43 to i64
  %cond_true45 = icmp ne i64 %zext44, 0
  br i1 %cond_true45, label %bb327, label %bb328

bb326:                                            ; preds = %bb316
  call void @panic(ptr @global_str.7)
  unreachable

bb327:                                            ; preds = %bb325
  %load46 = load ptr, ptr %self, align 8
  %ptr2int47 = ptrtoint ptr %load46 to i64
  %cmpne48 = icmp ne i64 %ptr2int47, 0
  %zext49 = zext i1 %cmpne48 to i64
  %cond_true50 = icmp ne i64 %zext49, 0
  br i1 %cond_true50, label %bb329, label %bb330

bb328:                                            ; preds = %bb325
  call void @panic(ptr @global_str.3)
  unreachable

bb329:                                            ; preds = %bb327
  %gep51 = getelementptr %Vec_str, ptr %load46, i64 0, i32 1
  %load52 = load i64, ptr %gep51, align 8
  %load53 = load ptr, ptr %item, align 8
  %gep54 = getelementptr ptr, ptr %load41, i64 %load52
  store ptr %load53, ptr %gep54, align 8
  %load55 = load ptr, ptr %self, align 8
  %ptr2int56 = ptrtoint ptr %load55 to i64
  %cmpne57 = icmp ne i64 %ptr2int56, 0
  %zext58 = zext i1 %cmpne57 to i64
  %cond_true59 = icmp ne i64 %zext58, 0
  br i1 %cond_true59, label %bb331, label %bb332

bb330:                                            ; preds = %bb327
  call void @panic(ptr @global_str.7)
  unreachable

bb331:                                            ; preds = %bb329
  %load60 = load ptr, ptr %self, align 8
  %ptr2int61 = ptrtoint ptr %load60 to i64
  %cmpne62 = icmp ne i64 %ptr2int61, 0
  %zext63 = zext i1 %cmpne62 to i64
  %cond_true64 = icmp ne i64 %zext63, 0
  br i1 %cond_true64, label %bb333, label %bb334

bb332:                                            ; preds = %bb329
  call void @panic(ptr @global_str.20)
  unreachable

bb333:                                            ; preds = %bb331
  %gep65 = getelementptr %Vec_str, ptr %load60, i64 0, i32 1
  %load66 = load i64, ptr %gep65, align 8
  %add = add i64 %load66, 1
  %gep67 = getelementptr %Vec_str, ptr %load55, i64 0, i32 1
  store i64 %add, ptr %gep67, align 8
  ret void

bb334:                                            ; preds = %bb331
  call void @panic(ptr @global_str.7)
  unreachable
}

define ptr @Vec_str_get(ptr %0, i64 %1) {
bb446:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %index = alloca i64, align 8
  store i64 %1, ptr %index, align 8
  %load = load i64, ptr %index, align 8
  %load1 = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load1 to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb447, label %bb448

bb447:                                            ; preds = %bb446
  %gep = getelementptr %Vec_str, ptr %load1, i64 0, i32 1
  %load2 = load i64, ptr %gep, align 8
  %cmpge = icmp uge i64 %load, %load2
  %zext3 = zext i1 %cmpge to i64
  %cast_trunc = trunc i64 %zext3 to i1
  %cast_zext = zext i1 %cast_trunc to i64
  %cond_true4 = icmp ne i64 %cast_zext, 0
  br i1 %cond_true4, label %bb449, label %bb450

bb448:                                            ; preds = %bb446
  call void @panic(ptr @global_str.7)
  unreachable

bb449:                                            ; preds = %bb447
  call void @panic(ptr @global_str.22)
  br label %bb450

bb450:                                            ; preds = %bb449, %bb447
  %load5 = load ptr, ptr %self, align 8
  %ptr2int6 = ptrtoint ptr %load5 to i64
  %cmpne7 = icmp ne i64 %ptr2int6, 0
  %zext8 = zext i1 %cmpne7 to i64
  %cond_true9 = icmp ne i64 %zext8, 0
  br i1 %cond_true9, label %bb451, label %bb452

bb451:                                            ; preds = %bb450
  %gep10 = getelementptr %Vec_str, ptr %load5, i64 0, i32 0
  %load11 = load ptr, ptr %gep10, align 8
  %ptr2int12 = ptrtoint ptr %load11 to i64
  %cmpne13 = icmp ne i64 %ptr2int12, 0
  %zext14 = zext i1 %cmpne13 to i64
  %cond_true15 = icmp ne i64 %zext14, 0
  br i1 %cond_true15, label %bb453, label %bb454

bb452:                                            ; preds = %bb450
  call void @panic(ptr @global_str.7)
  unreachable

bb453:                                            ; preds = %bb451
  %load16 = load i64, ptr %index, align 8
  %gep17 = getelementptr ptr, ptr %load11, i64 %load16
  %load18 = load ptr, ptr %gep17, align 8
  ret ptr %load18

bb454:                                            ; preds = %bb451
  call void @panic(ptr @global_str.4)
  unreachable
}

define float @Point3_f32_add3(ptr %0) {
bb497:
  %self = alloca ptr, align 8
  store ptr %0, ptr %self, align 8
  %load = load ptr, ptr %self, align 8
  %ptr2int = ptrtoint ptr %load to i64
  %cmpne = icmp ne i64 %ptr2int, 0
  %zext = zext i1 %cmpne to i64
  %cond_true = icmp ne i64 %zext, 0
  br i1 %cond_true, label %bb498, label %bb499

bb498:                                            ; preds = %bb497
  %gep = getelementptr %Point3_f32, ptr %load, i64 0, i32 0
  %load1 = load float, ptr %gep, align 4
  %load2 = load ptr, ptr %self, align 8
  %ptr2int3 = ptrtoint ptr %load2 to i64
  %cmpne4 = icmp ne i64 %ptr2int3, 0
  %zext5 = zext i1 %cmpne4 to i64
  %cond_true6 = icmp ne i64 %zext5, 0
  br i1 %cond_true6, label %bb500, label %bb501

bb499:                                            ; preds = %bb497
  call void @panic(ptr @global_str.7)
  unreachable

bb500:                                            ; preds = %bb498
  %gep7 = getelementptr %Point3_f32, ptr %load2, i64 0, i32 1
  %load8 = load float, ptr %gep7, align 4
  %fadd = fadd float %load1, %load8
  %load9 = load ptr, ptr %self, align 8
  %ptr2int10 = ptrtoint ptr %load9 to i64
  %cmpne11 = icmp ne i64 %ptr2int10, 0
  %zext12 = zext i1 %cmpne11 to i64
  %cond_true13 = icmp ne i64 %zext12, 0
  br i1 %cond_true13, label %bb502, label %bb503

bb501:                                            ; preds = %bb498
  call void @panic(ptr @global_str.7)
  unreachable

bb502:                                            ; preds = %bb500
  %gep14 = getelementptr %Point3_f32, ptr %load9, i64 0, i32 2
  %load15 = load float, ptr %gep14, align 4
  %fadd16 = fadd float %fadd, %load15
  ret float %fadd16

bb503:                                            ; preds = %bb500
  call void @panic(ptr @global_str.7)
  unreachable
}

define void @__global_init() {
bb0:
  ret void
}
