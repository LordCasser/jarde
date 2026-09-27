import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

import jdk.internal.org.objectweb.asm.ClassReader;
import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.Opcodes;
import jdk.internal.org.objectweb.asm.tree.AbstractInsnNode;
import jdk.internal.org.objectweb.asm.tree.ClassNode;
import jdk.internal.org.objectweb.asm.tree.FieldInsnNode;
import jdk.internal.org.objectweb.asm.tree.FieldNode;
import jdk.internal.org.objectweb.asm.tree.InsnList;
import jdk.internal.org.objectweb.asm.tree.JumpInsnNode;
import jdk.internal.org.objectweb.asm.tree.LabelNode;
import jdk.internal.org.objectweb.asm.tree.MethodInsnNode;
import jdk.internal.org.objectweb.asm.tree.MethodNode;
import jdk.internal.org.objectweb.asm.tree.TryCatchBlockNode;
import jdk.internal.org.objectweb.asm.tree.VarInsnNode;

/** Emits verifier-valid one-fact-at-a-time near misses from the frozen positive class. */
public final class MutateTest14 {
    private static final String TARGET = "jadx/tests/integration/trycatch/TestTryCatchFinally14$TestCls";
    private static final String TCLS = "jadx/tests/integration/trycatch/TestTryCatchFinally14$TestCls$TCls";
    private static final String DESC = "Ljadx/tests/integration/trycatch/TestTryCatchFinally14$TestCls$TCls;";

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("usage: MutateTest14 INPUT_CLASS OUTPUT_DIR");
        }
        byte[] original = Files.readAllBytes(Path.of(args[0]));
        Path out = Path.of(args[1]);
        Files.createDirectories(out);
        for (String mutant : List.of("field", "call", "predicate", "self-protected", "external-entry", "throwable")) {
            ClassNode node = read(original);
            MethodNode method = node.methods.stream()
                    .filter(item -> item.name.equals("test") && item.desc.equals("()V"))
                    .findFirst().orElseThrow();
            switch (mutant) {
                case "field" -> {
                    node.fields.add(new FieldNode(Opcodes.ACC_PRIVATE, "alternate", DESC, null, null));
                    changeField(method);
                }
                case "call" -> changeCall(method);
                case "predicate" -> changePredicate(method);
                case "self-protected" -> widenToNormalCleanup(method);
                case "external-entry" -> addExternalEntry(method);
                case "throwable" -> rewriteThrowable(method);
                default -> throw new AssertionError(mutant);
            }
            ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
            node.accept(writer);
            Files.write(out.resolve(mutant + ".class"), writer.toByteArray());
        }
    }

    private static ClassNode read(byte[] bytes) {
        ClassNode node = new ClassNode(Opcodes.ASM8);
        new ClassReader(bytes).accept(node, 0);
        if (!node.name.equals(TARGET)) {
            throw new IllegalArgumentException("unexpected target class: " + node.name);
        }
        return node;
    }

    private static List<AbstractInsnNode> real(InsnList instructions) {
        List<AbstractInsnNode> result = new ArrayList<>();
        for (AbstractInsnNode instruction = instructions.getFirst(); instruction != null; instruction = instruction.getNext()) {
            if (instruction.getOpcode() >= 0) {
                result.add(instruction);
            }
        }
        return result;
    }

    private static void changeField(MethodNode method) {
        List<FieldInsnNode> fields = real(method.instructions).stream()
                .filter(FieldInsnNode.class::isInstance).map(FieldInsnNode.class::cast)
                .filter(field -> field.getOpcode() == Opcodes.GETFIELD && field.owner.equals(TARGET)
                        && field.name.equals("t") && field.desc.equals(DESC)).toList();
        if (fields.size() != 6) {
            throw new IllegalStateException("expected six target-field reads, found " + fields.size());
        }
        fields.get(2).name = "alternate";
    }

    private static void changeCall(MethodNode method) {
        List<MethodInsnNode> calls = real(method.instructions).stream()
                .filter(MethodInsnNode.class::isInstance).map(MethodInsnNode.class::cast)
                .filter(call -> call.owner.equals(TCLS) && call.name.equals("doFinally") && call.desc.equals("()V"))
                .toList();
        if (calls.size() != 2) {
            throw new IllegalStateException("expected two doFinally calls, found " + calls.size());
        }
        calls.get(0).name = "doFinallyAlt";
    }

    private static void changePredicate(MethodNode method) {
        List<JumpInsnNode> branches = real(method.instructions).stream()
                .filter(JumpInsnNode.class::isInstance).map(JumpInsnNode.class::cast)
                .filter(branch -> branch.getOpcode() == Opcodes.IFNULL).toList();
        if (branches.size() != 3) {
            throw new IllegalStateException("expected three IFNULL branches, found " + branches.size());
        }
        branches.get(1).setOpcode(Opcodes.IFNONNULL);
    }

    private static void widenToNormalCleanup(MethodNode method) {
        if (method.tryCatchBlocks.size() != 1) {
            throw new IllegalStateException("expected one exception row");
        }
        JumpInsnNode normalJoin = real(method.instructions).stream()
                .filter(JumpInsnNode.class::isInstance).map(JumpInsnNode.class::cast)
                .filter(branch -> branch.getOpcode() == Opcodes.GOTO).findFirst().orElseThrow();
        LabelNode end = new LabelNode();
        method.instructions.insertBefore(normalJoin, end);
        TryCatchBlockNode row = method.tryCatchBlocks.get(0);
        row.end = end;
    }

    private static void addExternalEntry(MethodNode method) {
        List<JumpInsnNode> branches = real(method.instructions).stream()
                .filter(JumpInsnNode.class::isInstance).map(JumpInsnNode.class::cast)
                .filter(branch -> branch.getOpcode() == Opcodes.IFNULL).toList();
        List<MethodInsnNode> finallyCalls = real(method.instructions).stream()
                .filter(MethodInsnNode.class::isInstance).map(MethodInsnNode.class::cast)
                .filter(call -> call.owner.equals(TCLS) && call.name.equals("doFinally") && call.desc.equals("()V"))
                .toList();
        if (branches.size() != 3 || finallyCalls.size() != 2) {
            throw new IllegalStateException("unexpected conditional cleanup shape");
        }
        LabelNode entry = new LabelNode();
        method.instructions.insertBefore(finallyCalls.get(0).getPrevious().getPrevious(), entry);
        branches.get(0).label = entry;
    }

    private static void rewriteThrowable(MethodNode method) {
        List<AbstractInsnNode> instructions = real(method.instructions);
        for (int i = instructions.size() - 2; i >= 0; i--) {
            AbstractInsnNode instruction = instructions.get(i);
            if (instruction.getOpcode() == Opcodes.ATHROW && instructions.get(i - 1) instanceof VarInsnNode load
                    && load.getOpcode() == Opcodes.ALOAD && load.var == 1) {
                method.instructions.set(load, new jdk.internal.org.objectweb.asm.tree.InsnNode(Opcodes.ACONST_NULL));
                return;
            }
        }
        throw new IllegalStateException("saved Throwable rethrow not found");
    }
}
