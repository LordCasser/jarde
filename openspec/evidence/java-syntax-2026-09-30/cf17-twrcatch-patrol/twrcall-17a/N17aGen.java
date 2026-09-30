import java.lang.classfile.*;
import java.lang.constant.ClassDesc;
import java.lang.constant.MethodTypeDesc;
import java.nio.file.*;

/** Hand-lowers three verifier-valid TWR bodies whose `pop` fails exactly one criterion conjunct:
 *
 *  - popWrongValue   (N2): `invoke; dup; pop; pop`  — the first pop reads the copy's value, not the
 *        call's own result; the second pop reads the call's result but does not follow the call.
 *  - popAfterCast    (N3): `invoke; checkcast; pop` — an instruction stands between the call and
 *        the pop, so no non-void call is adjacent to the pop.
 *  - popSecondReader (N4): `invoke; dup; astore_3; pop` — the duplicated copy is stored, so the
 *        discarded expression's value has a reader beside the pop.
 *  - pop2Discard     (N5): `invokestatic; pop2` — javac's own discard of a two-slot result, a
 *        shape the subset's single-slot `pop` statement does not carry.
 *
 *  The TWR lowering mirrors javac 23 --release 8 exactly (row pair, close, suppression, rethrow).
 */
public class N17aGen {
    static final ClassDesc SELF = ClassDesc.of("N17a");
    static final ClassDesc OBJECT = ClassDesc.ofDescriptor("Ljava/lang/Object;");
    static final ClassDesc SYSTEM = ClassDesc.of("java.lang.System");
    static final ClassDesc PRINTSTREAM = ClassDesc.of("java.io.PrintStream");
    static final ClassDesc THROWABLE = ClassDesc.of("java.lang.Throwable");
    static final ClassDesc STRING = ClassDesc.of("java.lang.String");
    static final ClassDesc CLOSEABLE = ClassDesc.of("java.lang.AutoCloseable");
    static final MethodTypeDesc MT_VOID = MethodTypeDesc.ofDescriptor("()V");
    static final MethodTypeDesc MT_STRING = MethodTypeDesc.ofDescriptor("()Ljava/lang/String;");
    static final MethodTypeDesc MT_SUPPRESS = MethodTypeDesc.ofDescriptor("(Ljava/lang/Throwable;)V");
    static final MethodTypeDesc MT_PRINT =
            MethodTypeDesc.ofDescriptor("(Ljava/lang/String;)V");
    static final String[] NAMES = {"popWrongValue", "popAfterCast", "popSecondReader", "pop2Discard"};
    static final ClassDesc SYSTEMTIME = ClassDesc.of("java.lang.System");

    public static void main(String[] args) throws Exception {
        var cf = ClassFile.of();
        byte[] bytes = cf.build(SELF, cb -> {
            cb.withFlags(ClassFile.ACC_PUBLIC | ClassFile.ACC_SUPER)
              .withSuperclass(OBJECT)
              .withInterfaceSymbols(CLOSEABLE);
            cb.withMethodBody("<init>", MT_VOID, ClassFile.ACC_PUBLIC, code -> code
                    .aload(0)
                    .invokespecial(OBJECT, "<init>", MT_VOID)
                    .return_());
            cb.withMethodBody("close", MT_VOID, ClassFile.ACC_PUBLIC, code -> code
                    .return_());
            for (String name : NAMES) {
                cb.withMethod(name, MT_STRING, ClassFile.ACC_PUBLIC | ClassFile.ACC_STATIC,
                        mb -> mb.withCode(code -> lower(code, name)));
            }
            cb.withMethodBody("main", MethodTypeDesc.ofDescriptor("([Ljava/lang/String;)V"),
                    ClassFile.ACC_PUBLIC | ClassFile.ACC_STATIC, code -> {
                        for (String name : NAMES) {
                            code.getstatic(SYSTEM, "out", PRINTSTREAM)
                                .invokestatic(SELF, name, MT_STRING)
                                .invokevirtual(PRINTSTREAM, "println", MT_PRINT);
                        }
                        code.return_();
                    });
        });
        Files.write(Path.of("N17a.class"), bytes);
        System.out.println("written " + bytes.length);
    }

    /** Lowers one method: `try (N17a r = new N17a()) { BODY } return "done";`. */
    static void lower(CodeBuilder code, String name) {
        code.new_(SELF)
            .dup()
            .invokespecial(SELF, "<init>", MT_VOID)
            .astore(0);
        Label bodyStart = code.newLabel();
        Label bodyEnd = code.newLabel();
        Label handler = code.newLabel();
        Label handlerCloseStart = code.newLabel();
        Label handlerCloseEnd = code.newLabel();
        Label handler2 = code.newLabel();
        Label suppress = code.newLabel();
        Label join = code.newLabel();

        code.labelBinding(bodyStart);
        switch (name) {
            case "popWrongValue" -> code.aload(0).invokevirtual(OBJECT, "toString", MT_STRING).dup().pop().pop();
            case "popAfterCast" -> code.aload(0).invokevirtual(OBJECT, "toString", MT_STRING).checkcast(STRING).pop();
            case "popSecondReader" -> code.aload(0).invokevirtual(OBJECT, "toString", MT_STRING).dup().astore(3).pop();
            // javac's own two-slot discard: `System.currentTimeMillis();` as a statement
            case "pop2Discard" -> code.invokestatic(SYSTEMTIME, "currentTimeMillis",
                    MethodTypeDesc.ofDescriptor("()J")).pop2();
            default -> throw new AssertionError(name);
        }
        code.labelBinding(bodyEnd);

        // normal close, then the run carries on to the return
        code.aload(0).invokevirtual(SELF, "close", MT_VOID).goto_(join);

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
        code.labelBinding(join);
        code.ldc("done").areturn();

        // rows: the body raises into the exceptional close; the exceptional close's own close
        // raises into the suppression. The ranges are javac's: the body only, and the handler's
        // own close sequence.
        code.exceptionCatch(bodyStart, bodyEnd, handler, THROWABLE);
        code.exceptionCatch(handlerCloseStart, handlerCloseEnd, handler2, THROWABLE);
    }
}
