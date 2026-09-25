import java.nio.file.Files;
import java.nio.file.Paths;

import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.Handle;
import jdk.internal.org.objectweb.asm.MethodVisitor;
import jdk.internal.org.objectweb.asm.Opcodes;
import jdk.internal.org.objectweb.asm.Type;

public final class GenerateUnknownReference implements Opcodes {
    public static void main(String[] args) throws Exception {
        ClassWriter writer = new ClassWriter(0);
        writer.visit(
                V1_8,
                ACC_PUBLIC | ACC_FINAL | ACC_SUPER,
                "UnknownReferenceProbe",
                null,
                "java/lang/Object",
                null);
        writer.visitField(ACC_PUBLIC | ACC_STATIC, "calls", "I", null, null).visitEnd();

        addFactory(writer, "sameType", "(LUnknownLeft;)Ljava/lang/Integer;");
        addFactory(writer, "unknownRelation", "(LUnknownRight;)Ljava/lang/Integer;");
        addConsumer(writer);

        writer.visitEnd();
        Files.write(Paths.get(args[0]), writer.toByteArray());
    }

    private static void addFactory(ClassWriter writer, String name, String instantiatedType) {
        MethodVisitor method = writer.visitMethod(
                ACC_PUBLIC | ACC_STATIC,
                name,
                "()Ljava/util/function/Function;",
                null,
                null);
        method.visitCode();
        Handle bootstrap = new Handle(
                H_INVOKESTATIC,
                "java/lang/invoke/LambdaMetafactory",
                "metafactory",
                "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;"
                        + "Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodType;"
                        + "Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)"
                        + "Ljava/lang/invoke/CallSite;",
                false);
        Handle implementation = new Handle(
                H_INVOKESTATIC,
                "UnknownReferenceProbe",
                "consume",
                "(LUnknownLeft;)Ljava/lang/Integer;",
                false);
        method.visitInvokeDynamicInsn(
                "apply",
                "()Ljava/util/function/Function;",
                bootstrap,
                Type.getType("(Ljava/lang/Object;)Ljava/lang/Object;"),
                implementation,
                Type.getType(instantiatedType));
        method.visitInsn(ARETURN);
        method.visitMaxs(1, 0);
        method.visitEnd();
    }

    private static void addConsumer(ClassWriter writer) {
        MethodVisitor method = writer.visitMethod(
                ACC_PUBLIC | ACC_STATIC,
                "consume",
                "(LUnknownLeft;)Ljava/lang/Integer;",
                null,
                null);
        method.visitCode();
        method.visitFieldInsn(GETSTATIC, "UnknownReferenceProbe", "calls", "I");
        method.visitInsn(ICONST_1);
        method.visitInsn(IADD);
        method.visitFieldInsn(PUTSTATIC, "UnknownReferenceProbe", "calls", "I");
        method.visitIntInsn(BIPUSH, 73);
        method.visitMethodInsn(
                INVOKESTATIC,
                "java/lang/Integer",
                "valueOf",
                "(I)Ljava/lang/Integer;",
                false);
        method.visitInsn(ARETURN);
        method.visitMaxs(2, 1);
        method.visitEnd();
    }
}
