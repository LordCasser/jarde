import java.nio.file.Files;
import java.nio.file.Paths;
import jdk.internal.org.objectweb.asm.ClassReader;
import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.Opcodes;
import jdk.internal.org.objectweb.asm.tree.*;

public class PatchNeighbor {
    public static void main(String[] args) throws Exception {
        ClassNode node = new ClassNode();
        new ClassReader(Files.readAllBytes(Paths.get(args[0]))).accept(node, 0);
        MethodNode method = null;
        for (MethodNode candidate : node.methods) {
            if (candidate.name.equals("handled")) method = candidate;
        }
        if (method == null) throw new IllegalStateException("missing handled");
        MethodInsnNode toString = null;
        TypeInsnNode argument = null;
        MethodInsnNode argumentInit = null;
        for (AbstractInsnNode insn = method.instructions.getFirst(); insn != null; insn = insn.getNext()) {
            if (insn instanceof MethodInsnNode) {
                MethodInsnNode call = (MethodInsnNode) insn;
                if (call.owner.equals("java/lang/StringBuilder") && call.name.equals("toString")) toString = call;
                if (call.owner.equals("java/lang/IllegalArgumentException") && call.name.equals("<init>")) argumentInit = call;
            }
            if (insn instanceof TypeInsnNode) {
                TypeInsnNode type = (TypeInsnNode) insn;
                if (type.desc.equals("java/lang/IllegalArgumentException")) argument = type;
            }
        }
        if (toString == null || argument == null || argumentInit == null) throw new IllegalStateException("missing target");
        switch (args[2]) {
            case "non-concat":
                method.instructions.set(toString, new MethodInsnNode(Opcodes.INVOKESTATIC, "java/lang/String", "valueOf", "(Ljava/lang/Object;)Ljava/lang/String;", false));
                break;
            case "extra-consumer": {
                MethodNode observe = new MethodNode(Opcodes.ACC_PRIVATE | Opcodes.ACC_STATIC, "observe", "(Ljava/lang/String;)V", null, null);
                observe.instructions.add(new InsnNode(Opcodes.RETURN));
                node.methods.add(observe);
                InsnList extra = new InsnList();
                extra.add(new InsnNode(Opcodes.DUP));
                extra.add(new MethodInsnNode(Opcodes.INVOKESTATIC, node.name, "observe", "(Ljava/lang/String;)V", false));
                method.instructions.insert(toString, extra);
                break;
            }
            case "range-shrunk": {
                LabelNode end = new LabelNode();
                method.instructions.insertBefore(toString, end);
                method.tryCatchBlocks.get(2).end = end;
                break;
            }
            case "exceptional-message":
                argument.desc = "ThrowingArgument";
                argumentInit.owner = "ThrowingArgument";
                break;
            default: throw new IllegalArgumentException(args[2]);
        }
        ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        node.accept(writer);
        Files.write(Paths.get(args[1]), writer.toByteArray());
    }
}
