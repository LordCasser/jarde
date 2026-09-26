import java.nio.file.Files;
import java.nio.file.Path;
import jdk.internal.org.objectweb.asm.ClassReader;
import jdk.internal.org.objectweb.asm.ClassVisitor;
import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.MethodVisitor;
import jdk.internal.org.objectweb.asm.Opcodes;

/** Builds verifier-valid Java 8 proof inputs from the frozen two-join class. */
public final class GenerateBridgeControls {
    private GenerateBridgeControls() {}

    public static void main(String[] args) throws Exception {
        byte[] original = Files.readAllBytes(Path.of(args[0]));
        write(original, Path.of(args[1]));
    }

    private static void write(byte[] original, Path target) throws Exception {
        ClassReader reader = new ClassReader(original);
        ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        reader.accept(new ClassVisitor(Opcodes.ASM8, writer) {
            @Override
            public MethodVisitor visitMethod(int access, String name, String descriptor,
                    String signature, String[] exceptions) {
                MethodVisitor base = super.visitMethod(access, name, descriptor, signature, exceptions);
                if (!name.equals("choose")) return base;
                return new MethodVisitor(Opcodes.ASM8, base) {
                    private boolean bridgeConstant;

                    @Override
                    public void visitInsn(int opcode) {
                        if (opcode == Opcodes.ICONST_3 && !bridgeConstant) {
                            bridgeConstant = true;
                        }
                        if (bridgeConstant && opcode == Opcodes.IADD) {
                            super.visitInsn(Opcodes.IXOR);
                            bridgeConstant = false;
                            return;
                        }
                        super.visitInsn(opcode);
                    }
                };
            }
        }, 0);
        Files.createDirectories(target.getParent());
        Files.write(target, writer.toByteArray());
    }
}
