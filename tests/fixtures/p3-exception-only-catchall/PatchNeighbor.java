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
            if (candidate.name.equals("escaping")) method = candidate;
        }
        if (method == null) throw new IllegalStateException("missing escaping");
        TryCatchBlockNode row = method.tryCatchBlocks.get(0);
        AbstractInsnNode firstThrow = null, lastThrow = null, store = null, load = null;
        for (AbstractInsnNode insn = method.instructions.getFirst(); insn != null; insn = insn.getNext()) {
            if (insn.getOpcode() == Opcodes.ATHROW) {
                if (firstThrow == null) firstThrow = insn;
                lastThrow = insn;
            }
            if (insn.getOpcode() == Opcodes.ASTORE) store = insn;
            if (insn.getOpcode() == Opcodes.ALOAD) load = insn;
        }
        switch (args[2]) {
            case "normal-exit":
                InsnNode pop = new InsnNode(Opcodes.POP);
                method.instructions.set(firstThrow, pop);
                method.instructions.insert(pop, new InsnNode(Opcodes.RETURN));
                break;
            case "range-shrunk": {
                AbstractInsnNode dup = firstThrow;
                while (dup.getPrevious() != null && dup.getOpcode() != Opcodes.DUP) dup = dup.getPrevious();
                LabelNode start = new LabelNode();
                method.instructions.insertBefore(dup, start);
                row.start = start;
                break;
            }
            case "range-expanded": {
                AbstractInsnNode firstGet = store.getNext();
                while (firstGet != null && firstGet.getOpcode() != Opcodes.GETSTATIC) firstGet = firstGet.getNext();
                LabelNode end = new LabelNode();
                method.instructions.insert(firstGet, end);
                row.end = end;
                break;
            }
            case "competing-row":
                method.tryCatchBlocks.add(0, new TryCatchBlockNode(row.start, row.end, row.handler, "java/lang/RuntimeException"));
                break;
            case "external-entry": {
                LabelNode original = new LabelNode();
                LabelNode external = new LabelNode();
                method.instructions.insertBefore(method.instructions.getFirst(), original);
                InsnList guard = new InsnList();
                guard.add(new FieldInsnNode(Opcodes.GETSTATIC, node.name, "cleanupCount", "I"));
                guard.add(new JumpInsnNode(Opcodes.IFNE, external));
                guard.add(new JumpInsnNode(Opcodes.GOTO, original));
                guard.add(external);
                guard.add(new InsnNode(Opcodes.ACONST_NULL));
                guard.add(new JumpInsnNode(Opcodes.GOTO, row.handler));
                method.instructions.insertBefore(original, guard);
                break;
            }
            case "wrong-rethrow":
                method.instructions.set(load, new InsnNode(Opcodes.ACONST_NULL));
                break;
            case "branch-cleanup": {
                LabelNode skip = new LabelNode();
                InsnList branch = new InsnList();
                branch.add(new FieldInsnNode(Opcodes.GETSTATIC, node.name, "cleanupCount", "I"));
                branch.add(new JumpInsnNode(Opcodes.IFEQ, skip));
                method.instructions.insert(store, branch);
                method.instructions.insertBefore(load, skip);
                break;
            }
            default: throw new IllegalArgumentException(args[2]);
        }
        ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        node.accept(writer);
        Files.write(Paths.get(args[1]), writer.toByteArray());
    }
}
