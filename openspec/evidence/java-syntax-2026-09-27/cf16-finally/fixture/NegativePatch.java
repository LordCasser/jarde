import java.nio.file.Files;
import java.nio.file.Paths;
import jdk.internal.org.objectweb.asm.*;
public final class NegativePatch {
  public static void main(String[] args) throws Exception {
    ClassReader reader = new ClassReader(Files.readAllBytes(Paths.get(args[0])));
    ClassWriter writer = new ClassWriter(reader, 0);
    reader.accept(new ClassVisitor(Opcodes.ASM7, writer) {
      @Override public MethodVisitor visitMethod(int access, String name, String desc, String signature, String[] exceptions) {
        MethodVisitor delegate = super.visitMethod(access, name, desc, signature, exceptions);
        if (!name.equals("test") || !desc.equals("(Ljava/lang/Object;)Z")) return delegate;
        return new MethodVisitor(Opcodes.ASM7, delegate) {
          int cleanupTrueConstants;
          @Override public void visitInsn(int opcode) {
            if (opcode == Opcodes.ICONST_1 && ++cleanupTrueConstants == 2) opcode = Opcodes.ICONST_0;
            super.visitInsn(opcode);
          }
        };
      }
    }, 0);
    Files.write(Paths.get(args[1]), writer.toByteArray());
  }
}
