import java.lang.classfile.*;
import java.lang.constant.ClassDesc;
import java.lang.constant.MethodTypeDesc;
import java.nio.file.*;

/** Hand-lowers three verifier-valid whole-construct catch clause bodies whose `pop`s are no
 *  discard the 17a criterion states (CF-17c, negatives of `recover-enclosing-catch-call-bodies`).
 *  The TWR rows mirror javac 23 --release 8 byte for byte (body row, close's own self-protection
 *  row); the last row is the compiler's whole-construct `catch`, and its handler body is a
 *  `log.append("E")` discard damaged one way per method:
 *
 *  - popWrongValue:  `invoke; dup; pop; pop` — the first `pop` reads the copy, not the call's
 *        result; the discard is of a value the `dup` made, a shape the criterion refuses;
 *  - popSecondReader: `invoke; dup; astore_1; pop` — the stored copy reads the value beside the
 *        `pop`: the result is consumed by a local store AND discarded, twice-used;
 *  - popAfterCast:   `invoke; checkcast; pop` — an instruction stands between the call and the
 *        `pop`, so the adjacency the identity needs is broken.
 *
 *  Every handler stays one straight block whose entry store binds the clause parameter and whose
 *  only exit is a value `return`, so the whole-construct clause reading fails on the discard
 *  criterion alone and the method keeps the mainline `jre_guard_unexplained_row` refusal. The
 *  bodies never run (the `try` body cannot raise), so the class runs clean; it is the *recovery*
 *  that refuses.
 */
public class N17cGen {
    static final ClassDesc SELF = ClassDesc.of("N17c");
    static final ClassDesc OBJECT = ClassDesc.ofDescriptor("Ljava/lang/Object;");
    static final ClassDesc THROWABLE = ClassDesc.of("java.lang.Throwable");
    static final ClassDesc STRING = ClassDesc.of("java.lang.String");
    static final ClassDesc SB = ClassDesc.of("java.lang.StringBuilder");
    static final ClassDesc CLOSEABLE = ClassDesc.of("java.lang.AutoCloseable");
    static final ClassDesc ISE = ClassDesc.of("java.lang.IllegalStateException");
    static final MethodTypeDesc MT_VOID = MethodTypeDesc.ofDescriptor("()V");
    static final MethodTypeDesc MT_STRING = MethodTypeDesc.ofDescriptor("()Ljava/lang/String;");
    static final MethodTypeDesc MT_SUPPRESS = MethodTypeDesc.ofDescriptor("(Ljava/lang/Throwable;)V");
    static final MethodTypeDesc MT_PRINT = MethodTypeDesc.ofDescriptor("(Ljava/lang/String;)V");
    static final MethodTypeDesc MT_APPEND = MethodTypeDesc.ofDescriptor("(Ljava/lang/String;)Ljava/lang/StringBuilder;");
    static final String[] NAMES = {
        "popWrongValue", "popSecondReader", "popAfterCast",
    };

    public static void main(String[] args) throws Exception {
        var cf = ClassFile.of();
        byte[] bytes = cf.build(SELF, cb -> {
            cb.withFlags(ClassFile.ACC_PUBLIC | ClassFile.ACC_SUPER)
              .withSuperclass(OBJECT)
              .withInterfaceSymbols(CLOSEABLE)
              .withField("LOG", SB, fb -> { });
            cb.withMethodBody("<init>", MT_VOID, ClassFile.ACC_PUBLIC, code -> code
                    .aload(0)
                    .invokespecial(OBJECT, "<init>", MT_VOID)
                    .return_());
            cb.withMethodBody("close", MT_VOID, ClassFile.ACC_PUBLIC, code -> code.return_());
            cb.withMethodBody("give", MT_VOID, ClassFile.ACC_PRIVATE | ClassFile.ACC_STATIC,
                    code -> code.return_());
            for (String name : NAMES) {
                cb.withMethod(name, MT_STRING, ClassFile.ACC_PUBLIC | ClassFile.ACC_STATIC,
                        mb -> mb.withCode(code -> lower(code, name)));
            }
            cb.withMethodBody("main", MethodTypeDesc.ofDescriptor("([Ljava/lang/String;)V"),
                    ClassFile.ACC_PUBLIC | ClassFile.ACC_STATIC, code -> {
                        for (String name : NAMES) {
                            code.getstatic(ClassDesc.of("java.lang.System"), "out",
                                    ClassDesc.of("java.io.PrintStream"))
                                .invokestatic(SELF, name, MT_STRING)
                                .invokevirtual(ClassDesc.of("java.io.PrintStream"), "println", MT_PRINT);
                        }
                        code.return_();
                    });
        });
        Files.write(Path.of("N17c.class"), bytes);
        System.out.println("written " + bytes.length);
    }

    /** Lowers one method: `try (N17c r = new N17c()) { give(); } catch (ISE e) { log.append("E"); … }
     *  return "done";`. */
    static void lower(CodeBuilder code, String name) {
        Label wholeStart = code.newLabel();
        Label bodyStart = code.newLabel();
        Label bodyEnd = code.newLabel();
        Label handler = code.newLabel();
        Label handlerCloseStart = code.newLabel();
        Label handlerCloseEnd = code.newLabel();
        Label handler2 = code.newLabel();
        Label suppress = code.newLabel();
        Label normalExit = code.newLabel();
        Label namedHandler = code.newLabel();
        Label join = code.newLabel();

        code.labelBinding(wholeStart);
        code.new_(SELF)
            .dup()
            .invokespecial(SELF, "<init>", MT_VOID)
            .astore(0);
        code.labelBinding(bodyStart);
        code.invokestatic(SELF, "give", MT_VOID);
        code.labelBinding(bodyEnd);

        // normal close, then the run carries on past the whole construct
        code.aload(0).invokevirtual(SELF, "close", MT_VOID).goto_(normalExit);

        // exceptional close: store the exception, close the resource, run on to the rethrow
        code.labelBinding(handler);
        code.astore(1);
        code.labelBinding(handlerCloseStart);
        code.aload(0).invokevirtual(SELF, "close", MT_VOID);
        code.labelBinding(handlerCloseEnd);
        code.goto_(suppress);
        // suppression: store the closer's exception, add it to the primary, rethrow the primary
        code.labelBinding(handler2);
        code.astore(2).aload(1).aload(2)
            .invokevirtual(THROWABLE, "addSuppressed", MT_SUPPRESS);
        code.labelBinding(suppress);
        code.aload(1).athrow();
        code.labelBinding(normalExit);
        code.goto_(join);

        // The named handler: the clause parameter's binding store, then `LOG.append("E")` as a
        // discard damaged per the variant, then one value return.
        code.labelBinding(namedHandler);
        code.astore(0)
            .getstatic(SELF, "LOG", SB)
            .ldc("E")
            .invokevirtual(SB, "append", MT_APPEND);
        switch (name) {
            case "popWrongValue" -> code.dup().pop().pop();
            case "popSecondReader" -> code.dup().astore(1).pop();
            case "popAfterCast" -> code.checkcast(SB).pop();
            default -> throw new AssertionError(name);
        }
        code.ldc("done").areturn();
        code.labelBinding(join);
        code.ldc("done").areturn();

        // rows: the body raises into the exceptional close; the exceptional close's own close
        // raises into the suppression; the whole-construct row the clause reading tries to take.
        code.exceptionCatch(bodyStart, bodyEnd, handler, THROWABLE);
        code.exceptionCatch(handlerCloseStart, handlerCloseEnd, handler2, THROWABLE);
        code.exceptionCatch(wholeStart, normalExit, namedHandler, ISE);
    }
}
