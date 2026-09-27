import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Collections;
import jdk.internal.org.objectweb.asm.ClassReader;
import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.Opcodes;
import jdk.internal.org.objectweb.asm.tree.AbstractInsnNode;
import jdk.internal.org.objectweb.asm.tree.ClassNode;
import jdk.internal.org.objectweb.asm.tree.IincInsnNode;
import jdk.internal.org.objectweb.asm.tree.InsnNode;
import jdk.internal.org.objectweb.asm.tree.JumpInsnNode;
import jdk.internal.org.objectweb.asm.tree.LabelNode;
import jdk.internal.org.objectweb.asm.tree.MethodInsnNode;
import jdk.internal.org.objectweb.asm.tree.MethodNode;
import jdk.internal.org.objectweb.asm.tree.TryCatchBlockNode;

/** Rebuild the verifier-valid CF-18 negative classes from the frozen original class. */
public final class NegativeFixtures {
    private static MethodNode run(ClassNode node) {
        for (MethodNode method : node.methods) {
            if (method.name.equals("run") && method.desc.equals("()I")) return method;
        }
        throw new IllegalStateException("run()I missing");
    }

    private static LabelNode before(MethodNode method, AbstractInsnNode instruction) {
        LabelNode label = new LabelNode();
        method.instructions.insertBefore(instruction, label);
        return label;
    }

    private static JumpInsnNode gotoNumber(MethodNode method, int number) {
        int found = 0;
        for (AbstractInsnNode instruction : method.instructions) {
            if (instruction instanceof JumpInsnNode && instruction.getOpcode() == Opcodes.GOTO) {
                if (++found == number) return (JumpInsnNode) instruction;
            }
        }
        throw new IllegalStateException("GOTO " + number + " missing");
    }

    private static void change(String kind, MethodNode method) {
        if (method.tryCatchBlocks.size() != 4) throw new IllegalStateException("table changed");
        TryCatchBlockNode outerMiddle = method.tryCatchBlocks.get(2);
        switch (kind) {
            case "wrong-type":
                outerMiddle.type = "java/lang/NumberFormatException";
                return;
            case "wrong-order":
                Collections.swap(method.tryCatchBlocks, 0, 2);
                return;
            case "missing-protection":
                for (AbstractInsnNode instruction : method.instructions) {
                    if (instruction instanceof MethodInsnNode
                            && ((MethodInsnNode) instruction).name.equals("work")) {
                        outerMiddle.start = before(method, instruction.getNext()); // BCI 33: iadd
                        return;
                    }
                }
                throw new IllegalStateException("work call missing");
            case "extra-throw-site":
                // BCI 25 was a three-byte GOTO outside the first outer row. A three-byte,
                // stack-neutral invocation at the same position now can throw in the gap.
                method.instructions.set(gotoNumber(method, 1),
                    new MethodInsnNode(Opcodes.INVOKESTATIC, "java/lang/System", "gc", "()V", false));
                return;
            case "other-loop-point":
                for (AbstractInsnNode instruction : method.instructions) {
                    if (instruction.getOpcode() == Opcodes.IINC
                            && ((jdk.internal.org.objectweb.asm.tree.IincInsnNode) instruction).incr == 5) {
                        gotoNumber(method, 3).label = before(method, instruction); // 52 -> 66
                        return;
                    }
                }
                throw new IllegalStateException("BCI 66 update missing");
            case "ordinary-handler-entry":
                // BCI 35's normal GOTO now enters the inner handler with a verifier-compatible
                // null exception reference on the stack.
                JumpInsnNode normal = gotoNumber(method, 2);
                method.instructions.insertBefore(normal, new InsnNode(Opcodes.ACONST_NULL));
                normal.label = method.tryCatchBlocks.get(0).handler;
                return;
            case "handler-index-write":
                // Keep all exception rows and normal targets, but alter the loop-carried
                // induction value only on exceptional entry to the outer handler.
                AbstractInsnNode store = method.tryCatchBlocks.get(1).handler.getNext();
                method.instructions.insert(store, new IincInsnNode(1, 1));
                return;
            case "crossing-range":
                for (AbstractInsnNode instruction : method.instructions) {
                    if (instruction.getOpcode() == Opcodes.IFNE) {
                        method.tryCatchBlocks.get(3).start = before(method, instruction.getPrevious()); // BCI 48
                        return;
                    }
                }
                throw new IllegalStateException("BCI 48 branch missing");
            default:
                throw new IllegalArgumentException(kind);
        }
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) throw new IllegalArgumentException("input.class output-directory");
        byte[] source = Files.readAllBytes(Path.of(args[0]));
        for (String kind : new String[] {"wrong-type", "wrong-order", "missing-protection",
                "extra-throw-site", "other-loop-point", "ordinary-handler-entry",
                "handler-index-write", "crossing-range"}) {
            ClassNode node = new ClassNode();
            new ClassReader(source).accept(node, 0);
            change(kind, run(node));
            ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
            node.accept(writer);
            Path target = Path.of(args[1], kind, "ExceptionRegionsAudit.class");
            Files.createDirectories(target.getParent());
            Files.write(target, writer.toByteArray());
        }
    }
}
