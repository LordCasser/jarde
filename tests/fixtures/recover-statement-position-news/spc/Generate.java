// Generates `SPC.class`, the **statement-position** twin of the frozen `ordinary-new-void-effect`
// counterexample: `new Target; dup; Side.effect()V; iconst_1; Target.<init>(I)V; pop; return`.
//
// Ordinary Java sources cannot write an independent call between an allocation's copy and its
// constructor, so this class is assembled rather than compiled — the same way the frozen
// counterexample itself was made (`openspec/evidence/java-syntax-2026-09-25/ordinary-new-void-effect/Generate.java`).
// What it pins is the *order* of this change's own refusals: the interleaved call is refused by the
// `Invoke ∉ argument_dependencies` branch before the statement-position reader is ever consulted,
// so the site keeps `jre_new_interleaved_effect` naming BCI 4 even though its argument (`iconst_1`)
// and its `pop` would otherwise satisfy the statement position.
//
// Build (from this directory, with the support family compiled into `out/`):
//
//   javac -d out Side.java Target.java Trace.java SPCRunner.java
//   javac --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED -d out Generate.java
//   java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED \
//       -cp out Generate SPC.class
import java.nio.file.Files;
import java.nio.file.Path;
import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.MethodVisitor;
import static jdk.internal.org.objectweb.asm.Opcodes.*;

public final class Generate {
    public static void main(String[] args) throws Exception {
        ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        writer.visit(V1_8, ACC_PUBLIC | ACC_SUPER, "SPC", null, "java/lang/Object", null);
        MethodVisitor method = writer.visitMethod(ACC_PUBLIC | ACC_STATIC, "run", "()V", null, null);
        method.visitCode();
        method.visitTypeInsn(NEW, "Target");
        method.visitInsn(DUP);
        method.visitMethodInsn(INVOKESTATIC, "Side", "effect", "()V", false);
        method.visitInsn(ICONST_1);
        method.visitMethodInsn(INVOKESPECIAL, "Target", "<init>", "(I)V", false);
        method.visitInsn(POP);
        method.visitInsn(RETURN);
        method.visitMaxs(0, 0);
        method.visitEnd();
        writer.visitEnd();
        Files.write(Path.of(args[0]), writer.toByteArray());
    }
}
