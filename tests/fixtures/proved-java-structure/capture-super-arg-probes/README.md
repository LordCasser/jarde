# Super-argument probes: the shapes javac never writes

Both probes are **hand-made bytecode**, frozen as class files by `freeze.py`; no Java source
produces them. javac hands the *parameter* to the superclass constructor and hoists a call out of
the constructor altogether, so the shapes a reorder's argument reading is about only exist as
bytes.

## `read-arg/` — the argument is read from the field the group moves

```
aload_0; aload_1; putfield ReadArg$1.val$s;
aload_0; aload_0; getfield ReadArg$1.val$s;
invokespecial ReadArg$Base.<init>(Ljava/lang/String;)V; return
```

The shape is **not verifiable**: JVMS 4.10.1.9 lets `uninitializedThis` be used for a `putfield`
of the current class and for the `invokespecial` that initializes it, and a `getfield` on it is
refused (`Type uninitializedThis (current frame, stack[1]) is not assignable to 'ReadArg$1'`).
`freeze.py` asserts that refusal on both runtimes it can find, and the presentation refuses the
whole constructor (`ir_frame_deferred`), which is the state
`tests/double_brace_capture.rs` pins as *current fact*: it changes the day the frame pass starts
carrying the shape, and the pin's classification has to be revisited with it.

## `call-arg/` — the argument is a call the artifact holds no body for

```
aload_0; aload_1; putfield CallArg$1.val$s;
aload_0; invokestatic CallArg$Helper.compute()Ljava/lang/String;
invokespecial CallArg$Base.<init>(Ljava/lang/String;)V; return
```

This one verifies and prints `c` under `java -Xverify:all` on both runtimes. The class declares
nothing but that constructor, so the only refusal the reorder can meet is the argument walk's:
the value handed to the superclass constructor comes from a body this run does not hold, and the
presentation keeps the constructor in the byte order (`this.val$s = arg1;` before the call) —
which `tests/double_brace_capture.rs` pins, and which flips the day that walk admits a call.

    python3 tests/fixtures/proved-java-structure/capture-super-arg-probes/freeze.py
