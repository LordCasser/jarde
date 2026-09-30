import java.lang.classfile.*;
import java.lang.constant.ClassDesc;
import java.lang.constant.MethodTypeDesc;
import java.nio.file.*;

/** Hand-lowers six verifier-valid surroundings of one complete TWR lowering, each failing exactly
 *  one guard of the whole-construct catch (CF-17b). The TWR rows mirror javac 23 --release 8 byte
 *  for byte (body row, close's own self-protection row, unchecked close); the last row is the
 *  compiler's whole-construct `catch`, damaged one way per method:
 *
 *  - catchAllSurround: the whole-construct row names no class at all (catch-all) — the `finally`
 *        semantics another certificate owns, never a named clause;
 *  - partialRow:       the row covers the initialisation and the body but stops before the cleanup
 *        chain ends, so it is no whole-construct row;
 *  - overlapHandler:   the row's handler is the lowering's own primary handler entry;
 *  - branchingHandler: the row's handler body branches (two `areturn` arms);
 *  - doubleCatch:      two whole-construct rows over one range, two handlers — a double `catch`;
 *  - multiCatch:       two whole-construct rows over one range, one handler, two classes — the
 *        multi-catch `catch (A | B e)` the slice does not state.
 */
public class N17bGen {
    static final ClassDesc SELF = ClassDesc.of("N17b");
    static final ClassDesc OBJECT = ClassDesc.ofDescriptor("Ljava/lang/Object;");
    static final ClassDesc THROWABLE = ClassDesc.of("java.lang.Throwable");
    static final ClassDesc STRING = ClassDesc.of("java.lang.String");
    static final ClassDesc CLOSEABLE = ClassDesc.of("java.lang.AutoCloseable");
    static final ClassDesc ISE = ClassDesc.of("java.lang.IllegalStateException");
    static final ClassDesc RUNTIME = ClassDesc.of("java.lang.RuntimeException");
    static final MethodTypeDesc MT_VOID = MethodTypeDesc.ofDescriptor("()V");
    static final MethodTypeDesc MT_STRING = MethodTypeDesc.ofDescriptor("()Ljava/lang/String;");
    static final MethodTypeDesc MT_SUPPRESS = MethodTypeDesc.ofDescriptor("(Ljava/lang/Throwable;)V");
    static final MethodTypeDesc MT_PRINT = MethodTypeDesc.ofDescriptor("(Ljava/lang/String;)V");
    static final String[] NAMES = {
        "catchAllSurround", "partialRow", "overlapHandler",
        "branchingHandler", "doubleCatch", "multiCatch",
    };

    public static void main(String[] args) throws Exception {
        var cf = ClassFile.of();
        byte[] bytes = cf.build(SELF, cb -> {
            cb.withFlags(ClassFile.ACC_PUBLIC | ClassFile.ACC_SUPER)
              .withSuperclass(OBJECT)
              .withInterfaceSymbols(CLOSEABLE)
              .withField("FLAG", ClassDesc.ofDescriptor("Z"), fb -> { });
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
        Files.write(Path.of("N17b.class"), bytes);
        System.out.println("written " + bytes.length);
    }

    /** Lowers one method: `try (N17b r = new N17b()) { give(); } … return "done";`. */
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
        Label namedHandler2 = code.newLabel();
        Label branchTarget = code.newLabel();
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

        // The named handler(s), then the join. The whole-construct row(s) are bound at the end.
        switch (name) {
            case "catchAllSurround", "partialRow", "overlapHandler" -> {
                code.labelBinding(namedHandler);
                code.astore(0).ldc("caught").areturn();
            }
            case "branchingHandler" -> {
                code.labelBinding(namedHandler);
                code.astore(0);
                code.getstatic(SELF, "FLAG", ClassDesc.ofDescriptor("Z"));
                code.ifne(branchTarget);
                code.ldc("caught").areturn();
                code.labelBinding(branchTarget);
                code.ldc("branch").areturn();
            }
            case "doubleCatch" -> {
                code.labelBinding(namedHandler);
                code.astore(0).ldc("one").areturn();
                code.labelBinding(namedHandler2);
                code.astore(0).ldc("two").areturn();
            }
            case "multiCatch" -> {
                code.labelBinding(namedHandler);
                code.astore(0).ldc("either").areturn();
            }
            default -> throw new AssertionError(name);
        }
        code.labelBinding(join);
        code.ldc("done").areturn();

        // rows: the body raises into the exceptional close; the exceptional close's own close
        // raises into the suppression; the whole-construct row(s) per the variant.
        code.exceptionCatch(bodyStart, bodyEnd, handler, THROWABLE);
        code.exceptionCatch(handlerCloseStart, handlerCloseEnd, handler2, THROWABLE);
        switch (name) {
            case "catchAllSurround" ->
                // the catch-all a compiler winds around a construct is `finally` semantics
                code.exceptionCatchAll(wholeStart, normalExit, namedHandler);
            case "partialRow" ->
                // covers the initialisation and the body, stops before the cleanup ends
                code.exceptionCatch(wholeStart, bodyEnd, namedHandler, ISE);
            case "overlapHandler" ->
                // a named row over the body whose handler is the lowering's own primary handler:
                // the handler block sits inside the claim, so it is no clause of the statement's
                // own and the row stays unexplained
                code.exceptionCatch(bodyStart, bodyEnd, handler, ISE);
            case "branchingHandler" ->
                code.exceptionCatch(wholeStart, normalExit, namedHandler, ISE);
            case "doubleCatch" -> {
                code.exceptionCatch(wholeStart, normalExit, namedHandler, ISE);
                code.exceptionCatch(wholeStart, normalExit, namedHandler2, RUNTIME);
            }
            case "multiCatch" -> {
                code.exceptionCatch(wholeStart, normalExit, namedHandler, ISE);
                code.exceptionCatch(wholeStart, normalExit, namedHandler, RUNTIME);
            }
            default -> throw new AssertionError(name);
        }
    }
}
