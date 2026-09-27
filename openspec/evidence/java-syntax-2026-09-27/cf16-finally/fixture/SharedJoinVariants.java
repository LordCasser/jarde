import java.nio.file.Files;
import java.nio.file.Paths;
import jdk.internal.org.objectweb.asm.ClassReader;
import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.Opcodes;
import jdk.internal.org.objectweb.asm.tree.AbstractInsnNode;
import jdk.internal.org.objectweb.asm.tree.ClassNode;
import jdk.internal.org.objectweb.asm.tree.JumpInsnNode;
import jdk.internal.org.objectweb.asm.tree.LabelNode;
import jdk.internal.org.objectweb.asm.tree.MethodNode;

/** Verifier-valid boundary probes for the fixed shared-join method. */
public final class SharedJoinVariants {
  public static void main(String[] args) throws Exception {
    ClassNode node = new ClassNode(Opcodes.ASM7);
    new ClassReader(Files.readAllBytes(Paths.get(args[1]))).accept(node, 0);
    MethodNode test = null;
    for (MethodNode method : node.methods) {
      if (method.name.equals("test") && method.desc.equals("(Ljava/lang/Object;)Z")) {
        test = method;
      }
    }
    if (test == null || test.tryCatchBlocks.size() != 3) throw new IllegalStateException("fixture changed");
    if (args[0].equals("row")) {
      AbstractInsnNode pop = null;
      for (AbstractInsnNode instruction : test.instructions) {
        if (instruction.getOpcode() == Opcodes.POP) { pop = instruction; break; }
      }
      if (pop == null) throw new IllegalStateException("missing BCI 9 pop");
      LabelNode end = new LabelNode();
      test.instructions.insertBefore(pop, end);
      test.tryCatchBlocks.get(1).end = end;
    } else if (args[0].equals("join")) {
      AbstractInsnNode pop = null;
      JumpInsnNode secondGoto = null;
      int gotos = 0;
      for (AbstractInsnNode instruction : test.instructions) {
        if (instruction.getOpcode() == Opcodes.POP) pop = instruction;
        if (instruction.getOpcode() == Opcodes.GOTO && ++gotos == 2) secondGoto = (JumpInsnNode) instruction;
      }
      if (pop == null || secondGoto == null) throw new IllegalStateException("missing completion edge");
      LabelNode firstCleanup = new LabelNode();
      test.instructions.insert(pop, firstCleanup);
      secondGoto.label = firstCleanup;
    } else {
      throw new IllegalArgumentException(args[0]);
    }
    ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
    node.accept(writer);
    Files.write(Paths.get(args[2]), writer.toByteArray());
  }
}
