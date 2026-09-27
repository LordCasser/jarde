import java.nio.file.*;
import jdk.internal.org.objectweb.asm.*;
import jdk.internal.org.objectweb.asm.tree.*;
public class MidEntryPatch {
  public static void main(String[] args) throws Exception {
    ClassNode node = new ClassNode();
    new ClassReader(Files.readAllBytes(Paths.get(args[0]))).accept(node, 0);
    for (MethodNode method : node.methods) {
      if (!method.name.equals("handled")) continue;
      int copies = 0;
      for (AbstractInsnNode insn = method.instructions.getFirst(); insn != null; insn = insn.getNext()) {
        if (insn.getOpcode() != Opcodes.GETSTATIC) continue;
        if (++copies != 1) continue;
        AbstractInsnNode next = insn.getNext();
        LabelNode entry = new LabelNode();
        method.instructions.insertBefore(next, entry);
        method.instructions.insert(insn, new JumpInsnNode(Opcodes.GOTO, entry));
        break;
      }
    }
    ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
    node.accept(writer);
    Files.write(Paths.get(args[1]), writer.toByteArray());
  }
}
