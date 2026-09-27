import java.nio.file.Files;
import java.nio.file.Paths;
import jdk.internal.org.objectweb.asm.ClassReader;
import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.Opcodes;
import jdk.internal.org.objectweb.asm.tree.*;

public class Test14Neighbor {
    public static void main(String[] args) throws Exception {
        if (args.length != 3) throw new IllegalArgumentException("MODE INPUT OUTPUT");
        ClassNode node = new ClassNode();
        new ClassReader(Files.readAllBytes(Paths.get(args[1]))).accept(node, 0);
        MethodNode method = null;
        for (MethodNode candidate : node.methods) {
            if (candidate.name.equals("test") && candidate.desc.equals("()V")) method = candidate;
        }
        if (method == null) throw new IllegalStateException("missing test");
        switch (args[0]) {
            case "slot0": {
                AbstractInsnNode first = method.instructions.getFirst();
                while (first != null && first.getOpcode() < 0) first = first.getNext();
                if (first == null || first.getOpcode() != Opcodes.ALOAD) throw new IllegalStateException("unexpected first instruction");
                InsnList rewrite = new InsnList();
                rewrite.add(new VarInsnNode(Opcodes.ALOAD, 0));
                rewrite.add(new VarInsnNode(Opcodes.ASTORE, 0));
                method.instructions.insertBefore(first, rewrite);
                break;
            }
            case "second-field": {
                int fields = 0;
                boolean changed = false;
                for (AbstractInsnNode insn = method.instructions.getFirst(); insn != null; insn = insn.getNext()) {
                    if (insn.getOpcode() == Opcodes.GETFIELD && ++fields == 4) {
                        FieldInsnNode field = (FieldInsnNode) insn;
                        node.fields.add(new FieldNode(Opcodes.ACC_PRIVATE, "tAlt", field.desc, null, null));
                        field.name = "tAlt";
                        changed = true;
                    }
                }
                if (!changed) throw new IllegalStateException("missing second normal field read");
                break;
            }
            case "handler-call": {
                int calls = 0;
                boolean changed = false;
                for (AbstractInsnNode insn = method.instructions.getFirst(); insn != null; insn = insn.getNext()) {
                    if (insn.getOpcode() == Opcodes.INVOKEVIRTUAL && ++calls == 3) {
                        MethodInsnNode call = (MethodInsnNode) insn;
                        if (!call.name.equals("doFinally") || !call.desc.equals("()V")) throw new IllegalStateException("unexpected handler call");
                        call.name = "doFinallyAlt";
                        changed = true;
                    }
                }
                if (!changed) throw new IllegalStateException("missing handler call");
                break;
            }
            default: throw new IllegalArgumentException(args[0]);
        }
        ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        node.accept(writer);
        Files.write(Paths.get(args[2]), writer.toByteArray());
    }
}
