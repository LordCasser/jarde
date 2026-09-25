// jarde: presentation of `ThrowAudit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ThrowAudit extends java.lang.Object {
    public ThrowAudit() {
        // @method <init>()V
        // @declaration a constructor of `ThrowAudit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void nullValue() {
        // jarde: not recovered: the recovery run for `nullValue()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nullValue()V
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the instruction at BCI 1 belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it
    }

    public static void parameter(java.lang.RuntimeException arg0) {
        // jarde: not recovered: the recovery run for `parameter(Ljava/lang/RuntimeException;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method parameter(Ljava/lang/RuntimeException;)V
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the instruction at BCI 1 belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it
    }

    public static void allocation() {
        // jarde: not recovered: the recovery run for `allocation()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method allocation()V
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6
        // the value at BCI 6 comes from an Duplicate at BCI 3, which produces no expression this subset writes
        // @bytecode 9
        // the instruction at BCI 9 belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it
    }

    public static void call() {
        // @method call()V
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ThrowEffects.problem();
        // @bytecode 3
        // the instruction at BCI 3 belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it
    }

    public static void cast(java.lang.Object arg0) {
        // jarde: not recovered: the recovery run for `cast(Ljava/lang/Object;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method cast(Ljava/lang/Object;)V
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the cast at BCI 1 is not consumed by a statement this run can write, so its runtime check remains quoted
        // @bytecode 4
        // the instruction at BCI 4 belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it
    }

    public static void checked(java.io.IOException arg0) throws java.io.IOException {
        // jarde: not recovered: the recovery run for `checked(Ljava/io/IOException;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method checked(Ljava/io/IOException;)V
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the instruction at BCI 1 belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it
    }

    public static java.lang.RuntimeException caught(java.lang.RuntimeException arg0) {
        // @method caught(Ljava/lang/RuntimeException;)Ljava/lang/RuntimeException;
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            // @bytecode 1
            // the instruction at BCI 1 belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it
        } catch (java.lang.RuntimeException local1) {
            return local1;
        }
    }

    public static void withFinally(java.lang.RuntimeException arg0) {
        // jarde: not recovered: the recovery run for `withFinally(Ljava/lang/RuntimeException;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method withFinally(Ljava/lang/RuntimeException;)V
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // BCI 2: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; merging the copies into one `finally` restates the source only if they are provably equal, which this build does not prove
        // @bytecode 2
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [2]
    }

    public static void synchronizedBody(java.lang.Object arg0, java.lang.RuntimeException arg1) {
        // jarde: not recovered: the recovery run for `synchronizedBody(Ljava/lang/Object;Ljava/lang/RuntimeException;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method synchronizedBody(Ljava/lang/Object;Ljava/lang/RuntimeException;)V
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // BCI 3: the monitor is not entered once and left on every path out of the region: a `synchronized` statement would drop the exit the bytecode performs
        // @bytecode 6
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [6]
    }

    public static void conditional(boolean arg0, java.lang.RuntimeException arg1, java.lang.RuntimeException arg2) {
        // @method conditional(ZLjava/lang/RuntimeException;Ljava/lang/RuntimeException;)V
        // @declaration a static method of `ThrowAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0) {
            // @bytecode 5
            // the instruction at BCI 5 belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it
        } else {
            // @bytecode 7
            // the instruction at BCI 7 belongs to no guarded shape this run proved: a monitor or a bare `throw` is written only where a rule claimed the statement around it
        }
    }
}
