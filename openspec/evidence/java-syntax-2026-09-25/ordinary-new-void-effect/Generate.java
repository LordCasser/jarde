import java.nio.file.Files;
import java.nio.file.Path;
import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.MethodVisitor;
import static jdk.internal.org.objectweb.asm.Opcodes.*;

public final class Generate {
    public static void main(String[] args) throws Exception {
        ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        writer.visit(V1_8, ACC_PUBLIC | ACC_SUPER, "VoidBetween", null, "java/lang/Object", null);
        MethodVisitor method = writer.visitMethod(ACC_PUBLIC | ACC_STATIC, "make", "()LTarget;", null, null);
        method.visitCode();
        method.visitTypeInsn(NEW, "Target");
        method.visitInsn(DUP);
        method.visitMethodInsn(INVOKESTATIC, "Side", "effect", "()V", false);
        method.visitInsn(ICONST_1);
        method.visitMethodInsn(INVOKESPECIAL, "Target", "<init>", "(I)V", false);
        method.visitInsn(ARETURN);
        method.visitMaxs(0, 0);
        method.visitEnd();
        writer.visitEnd();
        Files.write(Path.of(args[0]), writer.toByteArray());
    }
}
