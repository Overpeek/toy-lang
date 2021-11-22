; ModuleID = 'repl'
source_filename = "repl"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

@"format str" = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

; Function Attrs: noreturn
define void @main() local_unnamed_addr #0 {
entry:
  br label %loop

loop:                                             ; preds = %loop, %entry
  %f0.0 = phi i128 [ 0, %entry ], [ %"sum f0 f1", %loop ]
  %f1.0 = phi i128 [ 1, %entry ], [ %"sum f1 f0", %loop ]
  %"call printf0" = tail call i32 @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @"format str", i64 0, i64 0), i128 %f0.0)
  %"call printf1" = tail call i32 @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @"format str", i64 0, i64 0), i128 %f1.0)
  %"sum f0 f1" = add i128 %f1.0, %f0.0
  %"sum f1 f0" = add i128 %"sum f0 f1", %f1.0
  br label %loop
}

declare i32 @printf(i8*, i128) local_unnamed_addr

attributes #0 = { noreturn }
