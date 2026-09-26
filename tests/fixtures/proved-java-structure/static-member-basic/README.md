# DT-02 static member, with a top-level `$` control

`javac --release 8 -g:none -d . StaticMemberBasic.java 'Named$Top.java'` produces the frozen classes in this directory. The runtime output is `9:4`. `StaticMemberBasic$Leaf` has matching `InnerClasses` member metadata, while `Named$Top` is an unrelated top-level class whose dollar sign is part of its actual name. `class-sha256.txt` and `javap.txt` freeze the compiler output. The original/JADX/Jarde source comparison is in `openspec/evidence/java-syntax-2026-09-27/static-member-basic/`.
